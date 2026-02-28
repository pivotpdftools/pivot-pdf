use std::collections::HashMap;
use std::io;
use std::path::Path;

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors that can occur when reading a PDF file.
#[derive(Debug, PartialEq)]
pub enum PdfReadError {
    /// The bytes do not start with a valid `%PDF-` header.
    NotAPdf,
    /// The `startxref` keyword or its offset could not be found.
    StartxrefNotFound,
    /// The cross-reference table is missing or could not be parsed.
    MalformedXref,
    /// The trailer dictionary is missing or malformed.
    MalformedTrailer,
    /// The PDF uses a cross-reference stream (PDF 1.5+), which is not yet supported.
    XrefStreamNotSupported,
    /// An object reference could not be resolved (offset out of range or malformed).
    UnresolvableObject(u32),
    /// The page tree structure is invalid (missing /Count or /Pages).
    MalformedPageTree,
    /// An I/O error occurred while opening a file.
    Io(String),
}

impl std::fmt::Display for PdfReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdfReadError::NotAPdf => write!(f, "not a PDF file"),
            PdfReadError::StartxrefNotFound => write!(f, "startxref not found"),
            PdfReadError::MalformedXref => write!(f, "malformed or missing xref table"),
            PdfReadError::MalformedTrailer => write!(f, "malformed or missing trailer"),
            PdfReadError::XrefStreamNotSupported => {
                write!(
                    f,
                    "cross-reference streams (PDF 1.5+) are not yet supported"
                )
            }
            PdfReadError::UnresolvableObject(n) => write!(f, "cannot resolve object {}", n),
            PdfReadError::MalformedPageTree => write!(f, "malformed page tree"),
            PdfReadError::Io(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for PdfReadError {}

impl From<io::Error> for PdfReadError {
    fn from(e: io::Error) -> Self {
        PdfReadError::Io(e.to_string())
    }
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Reads an existing PDF file.
///
/// `PdfReader` parses the PDF's cross-reference table and trailer to locate
/// and resolve objects. The raw bytes and xref offset map are retained so that
/// future enhancements (editing, field extraction, merging) can resolve
/// arbitrary objects without re-reading the file.
///
/// # Limitations
/// PDF 1.5+ cross-reference streams are not supported. Files that use them
/// return `PdfReadError::XrefStreamNotSupported`.
pub struct PdfReader {
    /// Retained for future object resolution (editing, field extraction, merging).
    #[allow(dead_code)]
    data: Vec<u8>,
    /// Maps each object number to its byte offset in `data`.
    /// Retained for future object resolution.
    #[allow(dead_code)]
    xref: HashMap<u32, usize>,
    version: String,
    page_count: usize,
}

impl PdfReader {
    /// Open a PDF from a file path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, PdfReadError> {
        let data = std::fs::read(path.as_ref())?;
        Self::from_bytes(data)
    }

    /// Parse a PDF from raw bytes.
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, PdfReadError> {
        let version = parse_version(&data)?;
        let xref_offset = find_startxref(&data)?;
        let (xref, root_ref) = parse_xref_and_trailer(&data, xref_offset)?;
        let page_count = resolve_page_count(&data, &xref, root_ref)?;

        Ok(PdfReader {
            data,
            xref,
            version,
            page_count,
        })
    }

    /// Number of pages in the document.
    pub fn page_count(&self) -> usize {
        self.page_count
    }

    /// PDF version string (e.g. `"1.7"`).
    pub fn pdf_version(&self) -> &str {
        &self.version
    }
}

// ── Internal parsing ───────────────────────────────────────────────────────────

/// Extract the PDF version from the `%PDF-x.y` header.
fn parse_version(data: &[u8]) -> Result<String, PdfReadError> {
    if data.len() < 8 || !data.starts_with(b"%PDF-") {
        return Err(PdfReadError::NotAPdf);
    }
    // Version is the characters after "%PDF-" up to the first whitespace.
    let rest = &data[5..];
    let end = rest
        .iter()
        .position(|&b| b == b'\n' || b == b'\r' || b == b' ')
        .unwrap_or(rest.len());
    let version = std::str::from_utf8(&rest[..end])
        .map(|s| s.to_string())
        .map_err(|_| PdfReadError::NotAPdf)?;
    Ok(version)
}

/// Scan backward from the end of the file to find the `startxref` offset.
///
/// The PDF spec places `startxref\n{offset}\n%%EOF` near the end of the file.
/// We search within the last 1024 bytes to handle comments or trailing whitespace.
fn find_startxref(data: &[u8]) -> Result<usize, PdfReadError> {
    let search_start = data.len().saturating_sub(1024);
    let tail = &data[search_start..];

    // Search backward for the keyword "startxref"
    let keyword = b"startxref";
    let pos = tail
        .windows(keyword.len())
        .rposition(|w| w == keyword)
        .ok_or(PdfReadError::StartxrefNotFound)?;

    // The offset integer follows on the next line.
    let after = &tail[pos + keyword.len()..];
    let offset_str = skip_whitespace_to_token(after).ok_or(PdfReadError::StartxrefNotFound)?;
    let offset: usize = offset_str
        .parse()
        .map_err(|_| PdfReadError::StartxrefNotFound)?;

    if offset >= data.len() {
        return Err(PdfReadError::StartxrefNotFound);
    }

    Ok(offset)
}

/// Parse the xref table starting at `xref_offset` and the following trailer.
///
/// Returns `(object_offset_map, root_object_number)`.
fn parse_xref_and_trailer(
    data: &[u8],
    xref_offset: usize,
) -> Result<(HashMap<u32, usize>, u32), PdfReadError> {
    if xref_offset >= data.len() {
        return Err(PdfReadError::MalformedXref);
    }

    let section = &data[xref_offset..];

    // Check for cross-reference stream (PDF 1.5+): starts with "N 0 obj" not "xref"
    let trimmed = skip_ascii_whitespace(section);
    if !trimmed.starts_with(b"xref") {
        return Err(PdfReadError::XrefStreamNotSupported);
    }

    let xref = parse_xref_table(section)?;
    let root = parse_trailer_root(data, xref_offset)?;

    Ok((xref, root))
}

/// Parse the traditional xref table.
///
/// Each subsection has a header line `{first_obj} {count}` followed by
/// 20-byte fixed-width entries: `{offset:010} {gen:05} {n|f}\r\n`.
fn parse_xref_table(section: &[u8]) -> Result<HashMap<u32, usize>, PdfReadError> {
    let mut map = HashMap::new();

    // Skip "xref\n"
    let rest = skip_ascii_whitespace(consume_token(section, b"xref")?);

    let mut cursor = rest;
    loop {
        let trimmed = skip_ascii_whitespace(cursor);
        // Stop at "trailer" or end of section
        if trimmed.is_empty() || trimmed.starts_with(b"trailer") {
            break;
        }

        // Subsection header: "{first_obj} {count}"
        let (first_obj_str, after_first) =
            next_token(trimmed).ok_or(PdfReadError::MalformedXref)?;
        let first_obj: u32 = first_obj_str
            .parse()
            .map_err(|_| PdfReadError::MalformedXref)?;

        let after_first = skip_ascii_whitespace(after_first);
        let (count_str, after_count) =
            next_token(after_first).ok_or(PdfReadError::MalformedXref)?;
        let count: usize = count_str.parse().map_err(|_| PdfReadError::MalformedXref)?;

        // Each entry is exactly 20 bytes: "oooooooooo ggggg n/f\r\n"
        let entries_start = skip_line(after_count);
        let entry_size = 20;
        let entries_bytes = entries_start.len();

        if entries_bytes < count * entry_size {
            return Err(PdfReadError::MalformedXref);
        }

        for i in 0..count {
            let entry = &entries_start[i * entry_size..(i + 1) * entry_size];
            // Offset: first 10 bytes
            let offset_bytes = &entry[..10];
            // Status: byte 17 ('n' = in-use, 'f' = free)
            let status = entry[17];

            if status == b'n' {
                let offset_str =
                    std::str::from_utf8(offset_bytes).map_err(|_| PdfReadError::MalformedXref)?;
                let offset: usize = offset_str
                    .parse()
                    .map_err(|_| PdfReadError::MalformedXref)?;
                let obj_num = first_obj + i as u32;
                if obj_num > 0 {
                    map.insert(obj_num, offset);
                }
            }
        }

        cursor = &entries_start[count * entry_size..];
    }

    Ok(map)
}

/// Extract the `/Root` object number from the trailer dictionary.
fn parse_trailer_root(data: &[u8], xref_offset: usize) -> Result<u32, PdfReadError> {
    // Find "trailer" after the xref table
    let section = &data[xref_offset..];
    let pos = section
        .windows(7)
        .position(|w| w == b"trailer")
        .ok_or(PdfReadError::MalformedTrailer)?;

    let after_trailer = skip_ascii_whitespace(&section[pos + 7..]);

    // Parse the trailer dictionary to find /Root
    let dict = parse_dict_bytes(after_trailer).ok_or(PdfReadError::MalformedTrailer)?;

    let root_ref = dict.get("Root").ok_or(PdfReadError::MalformedTrailer)?;
    // Root value is a reference: "N M R" — we only need N
    let obj_num: u32 = root_ref
        .parse()
        .map_err(|_| PdfReadError::MalformedTrailer)?;
    Ok(obj_num)
}

/// Follow the catalog → pages chain to read the `/Count` value.
fn resolve_page_count(
    data: &[u8],
    xref: &HashMap<u32, usize>,
    catalog_obj_num: u32,
) -> Result<usize, PdfReadError> {
    // Resolve catalog object → get /Pages reference
    let catalog_dict = resolve_dict(data, xref, catalog_obj_num)?;

    let pages_ref = catalog_dict
        .get("Pages")
        .ok_or(PdfReadError::MalformedPageTree)?;
    let pages_obj_num: u32 = pages_ref
        .parse()
        .map_err(|_| PdfReadError::MalformedPageTree)?;

    // Resolve pages object → read /Count
    let pages_dict = resolve_dict(data, xref, pages_obj_num)?;

    let count_str = pages_dict
        .get("Count")
        .ok_or(PdfReadError::MalformedPageTree)?;
    let count: usize = count_str
        .parse()
        .map_err(|_| PdfReadError::MalformedPageTree)?;

    Ok(count)
}

/// Resolve an indirect object by number, parse its body as a dictionary,
/// and return a flat `name → first-token-of-value` map.
fn resolve_dict(
    data: &[u8],
    xref: &HashMap<u32, usize>,
    obj_num: u32,
) -> Result<HashMap<String, String>, PdfReadError> {
    let offset = xref
        .get(&obj_num)
        .copied()
        .ok_or(PdfReadError::UnresolvableObject(obj_num))?;

    if offset >= data.len() {
        return Err(PdfReadError::UnresolvableObject(obj_num));
    }

    let slice = &data[offset..];

    // Skip "N G obj" header
    let after_header = skip_obj_header(slice).ok_or(PdfReadError::UnresolvableObject(obj_num))?;
    let after_ws = skip_ascii_whitespace(after_header);

    parse_dict_bytes(after_ws).ok_or(PdfReadError::UnresolvableObject(obj_num))
}

// ── Token / byte utilities ─────────────────────────────────────────────────────

/// Parse `<<...>>` dictionary bytes into a flat `key → first-token-of-value` map.
///
/// Values that are indirect references (`N G R`) are stored as just the object
/// number string. Nested dictionaries and arrays are skipped.
fn parse_dict_bytes(data: &[u8]) -> Option<HashMap<String, String>> {
    let data = skip_ascii_whitespace(data);
    if !data.starts_with(b"<<") {
        return None;
    }

    let mut map = HashMap::new();
    let mut cursor = &data[2..];

    loop {
        cursor = skip_ascii_whitespace(cursor);

        if cursor.starts_with(b">>") {
            break;
        }

        // Expect a name key: /KeyName
        if !cursor.starts_with(b"/") {
            // Skip unknown token
            let (_, rest) = next_token(cursor)?;
            cursor = rest;
            continue;
        }

        let (key, after_key) = next_token(&cursor[1..])?;
        cursor = skip_ascii_whitespace(after_key);

        // Read the value — we only need the first token (object number for refs)
        if cursor.starts_with(b"<<") {
            // Nested dict: skip to matching >>
            cursor = skip_nested_dict(cursor)?;
        } else if cursor.starts_with(b"[") {
            // Array: skip to ]
            cursor = skip_array(cursor)?;
        } else if cursor.starts_with(b"(") {
            // Literal string: skip to closing )
            cursor = skip_literal_string(cursor)?;
        } else {
            let (val, rest) = next_token(cursor)?;
            cursor = skip_ascii_whitespace(rest);

            // If it looks like an indirect reference (val=N, next="G R"), capture just N
            if let Some((gen_str, after_gen)) = next_token(cursor) {
                let after_gen_ws = skip_ascii_whitespace(after_gen);
                if let Some((r_str, after_r)) = next_token(after_gen_ws) {
                    if r_str == "R"
                        && val.chars().all(|c| c.is_ascii_digit())
                        && gen_str.chars().all(|c| c.is_ascii_digit())
                    {
                        map.insert(key.to_string(), val.to_string());
                        cursor = after_r;
                        continue;
                    }
                }
                // Not a reference: store the raw value token
                map.insert(key.to_string(), val.to_string());
            } else {
                map.insert(key.to_string(), val.to_string());
            }
        }
    }

    Some(map)
}

/// Skip over a `<<...>>` block (with nested dicts), returning bytes after `>>`.
fn skip_nested_dict(data: &[u8]) -> Option<&[u8]> {
    debug_assert!(data.starts_with(b"<<"));
    let mut depth = 0usize;
    let mut i = 0;
    while i < data.len() {
        if data[i..].starts_with(b"<<") {
            depth += 1;
            i += 2;
        } else if data[i..].starts_with(b">>") {
            if depth == 0 {
                return Some(&data[i + 2..]);
            }
            depth -= 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    None
}

/// Skip over a `[...]` array, returning bytes after `]`.
fn skip_array(data: &[u8]) -> Option<&[u8]> {
    debug_assert!(data.starts_with(b"["));
    let pos = data.iter().position(|&b| b == b']')?;
    Some(&data[pos + 1..])
}

/// Skip over a `(...)` literal string (handles backslash escapes), returning bytes after `)`.
fn skip_literal_string(data: &[u8]) -> Option<&[u8]> {
    debug_assert!(data.starts_with(b"("));
    let mut i = 1;
    let mut depth = 1i32;
    while i < data.len() {
        match data[i] {
            b'\\' => i += 2,
            b'(' => {
                depth += 1;
                i += 1;
            }
            b')' => {
                depth -= 1;
                i += 1;
                if depth == 0 {
                    return Some(&data[i..]);
                }
            }
            _ => i += 1,
        }
    }
    None
}

/// Skip "N G obj" indirect object header, returning bytes after "obj".
fn skip_obj_header(data: &[u8]) -> Option<&[u8]> {
    let (_, rest) = next_token(data)?; // object number
    let rest = skip_ascii_whitespace(rest);
    let (_, rest) = next_token(rest)?; // generation number
    let rest = skip_ascii_whitespace(rest);
    let (keyword, rest) = next_token(rest)?; // "obj"
    if keyword != "obj" {
        return None;
    }
    Some(rest)
}

/// Return a sub-slice starting at the first non-whitespace byte.
fn skip_ascii_whitespace(data: &[u8]) -> &[u8] {
    let pos = data
        .iter()
        .position(|&b| !b.is_ascii_whitespace())
        .unwrap_or(data.len());
    &data[pos..]
}

/// Skip to the end of the current line (past `\n` or `\r\n`).
fn skip_line(data: &[u8]) -> &[u8] {
    let pos = data
        .iter()
        .position(|&b| b == b'\n')
        .unwrap_or(data.len().saturating_sub(1));
    if pos + 1 < data.len() {
        &data[pos + 1..]
    } else {
        &data[data.len()..]
    }
}

/// Consume a literal byte sequence at the start of `data`, returning the remainder.
fn consume_token<'a>(data: &'a [u8], token: &[u8]) -> Result<&'a [u8], PdfReadError> {
    let trimmed = skip_ascii_whitespace(data);
    if trimmed.starts_with(token) {
        Ok(&trimmed[token.len()..])
    } else {
        Err(PdfReadError::MalformedXref)
    }
}

/// Read the next whitespace-delimited token from `data`.
/// Returns `(token_str, remaining_bytes)` or `None` if at end.
fn next_token(data: &[u8]) -> Option<(&str, &[u8])> {
    let data = skip_ascii_whitespace(data);
    if data.is_empty() {
        return None;
    }
    let end = data
        .iter()
        .position(|&b| b.is_ascii_whitespace() || b == b'<' || b == b'>')
        .unwrap_or(data.len());
    if end == 0 {
        // Single delimiter character
        let token = std::str::from_utf8(&data[..1]).ok()?;
        return Some((token, &data[1..]));
    }
    let token = std::str::from_utf8(&data[..end]).ok()?;
    Some((token, &data[end..]))
}

/// Find the first non-whitespace token in `data` and parse it as a string.
fn skip_whitespace_to_token(data: &[u8]) -> Option<&str> {
    let (tok, _) = next_token(data)?;
    Some(tok)
}
