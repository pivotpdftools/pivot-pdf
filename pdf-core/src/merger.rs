use std::collections::{HashMap, HashSet};
use std::io;
use std::path::Path;

use crate::reader::{PdfReadError, PdfReader};

// ── Public types ───────────────────────────────────────────────────────────────

/// Options controlling PDF merge behaviour.
pub struct MergeOptions {
    /// Flatten interactive form fields into static page content before merging.
    ///
    /// **Not yet implemented.** Returns `PdfMergeError::NotSupported` if `true`.
    pub flatten_forms: bool,
}

impl Default for MergeOptions {
    fn default() -> Self {
        Self {
            flatten_forms: false,
        }
    }
}

/// Errors that can occur during a PDF merge operation.
#[derive(Debug)]
pub enum PdfMergeError {
    /// A feature is requested that is not yet implemented.
    NotSupported,
    /// An error occurred while reading one of the source PDFs.
    ReadError(PdfReadError),
    /// An I/O error occurred while writing the output file.
    Io(String),
}

impl std::fmt::Display for PdfMergeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdfMergeError::NotSupported => write!(f, "operation not yet supported"),
            PdfMergeError::ReadError(e) => write!(f, "read error: {}", e),
            PdfMergeError::Io(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for PdfMergeError {}

impl From<io::Error> for PdfMergeError {
    fn from(e: io::Error) -> Self {
        PdfMergeError::Io(e.to_string())
    }
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Merge one or more PDF files into a single output PDF.
///
/// Pages from each source are appended in order. All pages from `inputs[0]`
/// come first, then all pages from `inputs[1]`, and so on.
///
/// # Limitations
/// - Sources must use traditional xref tables (xref streams / PDF 1.5+ not supported).
/// - `options.flatten_forms = true` returns `PdfMergeError::NotSupported`.
pub fn merge_pdfs<P: AsRef<Path>>(
    inputs: &[P],
    output: P,
    options: MergeOptions,
) -> Result<(), PdfMergeError> {
    if options.flatten_forms {
        return Err(PdfMergeError::NotSupported);
    }

    // ── Phase 1: read sources, collect object closures, assign new IDs ────────

    struct SourceData {
        reader: PdfReader,
        page_obj_nums: Vec<u32>,
        closure: HashSet<u32>,
        remap: HashMap<u32, u32>,
    }

    let mut next_id: u32 = 1;
    let mut sources: Vec<SourceData> = Vec::new();

    for path in inputs {
        let reader = PdfReader::open(path).map_err(PdfMergeError::ReadError)?;
        let page_obj_nums = reader
            .page_object_numbers()
            .map_err(PdfMergeError::ReadError)?;
        let closure = reader
            .collect_closure(&page_obj_nums)
            .map_err(PdfMergeError::ReadError)?;

        // Assign output IDs in a deterministic order.
        let mut sorted_objs: Vec<u32> = closure.iter().copied().collect();
        sorted_objs.sort_unstable();

        let mut remap = HashMap::new();
        for obj_num in sorted_objs {
            remap.insert(obj_num, next_id);
            next_id += 1;
        }

        sources.push(SourceData {
            reader,
            page_obj_nums,
            closure,
            remap,
        });
    }

    // Reserve IDs for the new merged Pages tree and Catalog.
    let pages_id = next_id;
    next_id += 1;
    let catalog_id = next_id;
    let max_id = catalog_id;

    // ── Phase 2: build the output PDF ─────────────────────────────────────────

    let mut out = OutputBuilder::new();
    out.write_header();

    // Copy and renumber all objects from each source.
    for source in &sources {
        let mut by_new_id: Vec<(u32, u32)> = source
            .closure
            .iter()
            .map(|&orig| (source.remap[&orig], orig))
            .collect();
        by_new_id.sort_unstable_by_key(|(new_id, _)| *new_id);

        for (new_id, orig_num) in by_new_id {
            let raw = source
                .reader
                .raw_object_bytes(orig_num)
                .map_err(PdfMergeError::ReadError)?;

            let renumbered = renumber_object_bytes(raw, &source.remap);
            out.write_raw_object(new_id, &renumbered);
        }
    }

    // Write the merged Pages tree — lists all source pages in document order.
    let all_page_ids: Vec<u32> = sources
        .iter()
        .flat_map(|s| s.page_obj_nums.iter().map(|n| s.remap[n]))
        .collect();

    let total_pages = all_page_ids.len();
    let kids: String = all_page_ids
        .iter()
        .map(|id| format!("{} 0 R", id))
        .collect::<Vec<_>>()
        .join(" ");

    out.write_object_str(
        pages_id,
        &format!("<< /Type /Pages /Count {} /Kids [{}] >>", total_pages, kids),
    );

    // Write the Catalog.
    out.write_object_str(
        catalog_id,
        &format!("<< /Type /Catalog /Pages {} 0 R >>", pages_id),
    );

    out.write_xref_and_trailer(max_id, catalog_id);

    std::fs::write(output, out.into_bytes())?;

    Ok(())
}

// ── Object renumbering ─────────────────────────────────────────────────────────

/// Rewrite all indirect references and the object header in `bytes` using `remap`.
///
/// Scans for `N G R` (indirect reference) and `N G obj` (object header) patterns
/// at word boundaries. When `N` appears in `remap`, replaces it with the mapped
/// value and resets the generation number to 0.
///
/// Stream bodies (between `stream` and `endstream`) are copied verbatim to avoid
/// corrupting binary-compressed content.
fn renumber_object_bytes(bytes: &[u8], remap: &HashMap<u32, u32>) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len() + 16);
    let mut i = 0;

    while i < bytes.len() {
        // Copy stream bodies verbatim — they may contain compressed binary data
        // that could accidentally match reference patterns.
        let at_boundary = i == 0 || is_pdf_delim(bytes[i - 1]);
        if at_boundary && bytes[i..].starts_with(b"stream") {
            // Verify it's the keyword, not e.g. "streaming".
            let after = i + 6;
            if after >= bytes.len() || is_pdf_delim(bytes[after]) {
                // Locate the stream body start (after the mandatory line ending).
                let body_start = skip_stream_newline(&bytes[after..])
                    .map(|n| after + n)
                    .unwrap_or(after);

                // Find "endstream" from the body start.
                let endstream_pos = bytes[body_start..]
                    .windows(9)
                    .position(|w| w == b"endstream")
                    .map(|p| body_start + p)
                    .unwrap_or(bytes.len());

                // Copy "stream\n...binary content...endstream" verbatim.
                out.extend_from_slice(&bytes[i..endstream_pos + 9]);
                i = endstream_pos + 9;
                continue;
            }
        }

        // At word boundaries, try to match and renumber `N G R` or `N G obj`.
        if at_boundary && i < bytes.len() && bytes[i].is_ascii_digit() {
            if let Some((n, consumed, keyword)) = parse_ngr(&bytes[i..]) {
                let new_n = remap.get(&n).copied().unwrap_or(n);
                out.extend_from_slice(format!("{} 0 {}", new_n, keyword).as_bytes());
                i += consumed;
                continue;
            }
        }

        out.push(bytes[i]);
        i += 1;
    }

    out
}

/// Returns the number of bytes consumed by the line ending after `stream`.
/// PDF spec requires either `\n` or `\r\n`.
fn skip_stream_newline(data: &[u8]) -> Option<usize> {
    match data.first()? {
        b'\n' => Some(1),
        b'\r' => {
            if data.get(1) == Some(&b'\n') {
                Some(2)
            } else {
                Some(1)
            }
        }
        _ => None,
    }
}

/// Attempt to parse `N G R` or `N G obj` at the start of `data`.
///
/// Returns `(object_number, bytes_consumed, keyword)` on success.
fn parse_ngr(data: &[u8]) -> Option<(u32, usize, &'static str)> {
    let mut i = 0;

    // Parse N (object number — one or more ASCII digits).
    let n_start = i;
    while i < data.len() && data[i].is_ascii_digit() {
        i += 1;
    }
    if i == n_start {
        return None;
    }
    let n: u32 = std::str::from_utf8(&data[n_start..i]).ok()?.parse().ok()?;

    // Require whitespace between N and G.
    if i >= data.len() || !data[i].is_ascii_whitespace() {
        return None;
    }
    while i < data.len() && data[i].is_ascii_whitespace() {
        i += 1;
    }

    // Parse G (generation number — one or more ASCII digits).
    let g_start = i;
    while i < data.len() && data[i].is_ascii_digit() {
        i += 1;
    }
    if i == g_start {
        return None;
    }

    // Require whitespace between G and the keyword.
    if i >= data.len() || !data[i].is_ascii_whitespace() {
        return None;
    }
    while i < data.len() && data[i].is_ascii_whitespace() {
        i += 1;
    }

    // Match keyword at a word boundary.
    if data[i..].starts_with(b"R") {
        let after = i + 1;
        if after >= data.len() || is_pdf_delim(data[after]) {
            return Some((n, after, "R"));
        }
    } else if data[i..].starts_with(b"obj") {
        let after = i + 3;
        if after >= data.len() || is_pdf_delim(data[after]) {
            return Some((n, after, "obj"));
        }
    }

    None
}

/// True for ASCII whitespace or PDF delimiter characters.
fn is_pdf_delim(b: u8) -> bool {
    b.is_ascii_whitespace() || matches!(b, b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'/' | b'%')
}

// ── Output builder ─────────────────────────────────────────────────────────────

struct OutputBuilder {
    buf: Vec<u8>,
    offsets: HashMap<u32, usize>,
}

impl OutputBuilder {
    fn new() -> Self {
        OutputBuilder {
            buf: Vec::new(),
            offsets: HashMap::new(),
        }
    }

    fn write_header(&mut self) {
        self.buf.extend_from_slice(b"%PDF-1.7\n");
        // Binary comment: 4 high bytes signal binary content to transfer tools.
        self.buf.extend_from_slice(b"%\xe2\xe3\xcf\xd3\n");
    }

    /// Append a pre-renumbered object (bytes already contain the correct `N 0 obj…endobj`).
    fn write_raw_object(&mut self, new_id: u32, bytes: &[u8]) {
        self.offsets.insert(new_id, self.buf.len());
        self.buf.extend_from_slice(bytes);
        // Ensure there is a trailing newline between objects.
        if !self.buf.ends_with(b"\n") {
            self.buf.push(b'\n');
        }
    }

    /// Write a new object whose body is supplied as a string (dictionary literal).
    fn write_object_str(&mut self, id: u32, body: &str) {
        self.offsets.insert(id, self.buf.len());
        self.buf
            .extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", id, body).as_bytes());
    }

    /// Write the xref table and trailer, then `startxref` and `%%EOF`.
    fn write_xref_and_trailer(&mut self, max_id: u32, catalog_id: u32) {
        let xref_offset = self.buf.len();
        let total = max_id + 1; // objects 0..=max_id

        self.buf.extend_from_slice(b"xref\n");
        self.buf
            .extend_from_slice(format!("0 {}\n", total).as_bytes());

        // Object 0: free-list head.
        self.buf.extend_from_slice(b"0000000000 65535 f\r\n");

        for obj_id in 1..=max_id {
            let offset = self.offsets.get(&obj_id).copied().unwrap_or(0);
            self.buf
                .extend_from_slice(format!("{:010} 00000 n\r\n", offset).as_bytes());
        }

        self.buf.extend_from_slice(b"trailer\n");
        self.buf.extend_from_slice(
            format!("<< /Size {} /Root {} 0 R >>\n", total, catalog_id).as_bytes(),
        );
        self.buf.extend_from_slice(b"startxref\n");
        self.buf
            .extend_from_slice(format!("{}\n", xref_offset).as_bytes());
        self.buf.extend_from_slice(b"%%EOF\n");
    }

    fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
}

// ── Internal unit tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocumentOptions, PdfDocument};

    fn make_pdf(n: usize) -> Vec<u8> {
        let mut doc = PdfDocument::new(Vec::new(), DocumentOptions::default()).unwrap();
        for _ in 0..n {
            doc.begin_page(612.0, 792.0);
            doc.end_page().unwrap();
        }
        doc.end_document().unwrap()
    }

    // ── renumber_object_bytes ─────────────────────────────────────────────────

    #[test]
    fn renumber_replaces_obj_header() {
        let input = b"5 0 obj\n<< /Type /Page >>\nendobj";
        let mut remap = HashMap::new();
        remap.insert(5u32, 3u32);
        let out = renumber_object_bytes(input, &remap);
        assert!(
            out.starts_with(b"3 0 obj"),
            "header not renumbered: {:?}",
            &out[..20]
        );
    }

    #[test]
    fn renumber_replaces_indirect_references() {
        let input = b"5 0 obj\n<< /Parent 2 0 R /Contents 6 0 R >>\nendobj";
        let mut remap = HashMap::new();
        remap.insert(5u32, 10u32);
        remap.insert(2u32, 20u32);
        remap.insert(6u32, 60u32);
        let out = renumber_object_bytes(input, &remap);
        let s = std::str::from_utf8(&out).unwrap();
        assert!(s.contains("20 0 R"), "Parent ref not renumbered: {}", s);
        assert!(s.contains("60 0 R"), "Contents ref not renumbered: {}", s);
    }

    #[test]
    fn renumber_does_not_corrupt_stream_body() {
        // A stream body with bytes that look like "2 0 R" must not be rewritten.
        let stream_body = b"2 0 R this looks like a ref but is compressed content";
        let input = {
            let mut v = b"7 0 obj\n<< /Length 51 >>\nstream\n".to_vec();
            v.extend_from_slice(stream_body);
            v.extend_from_slice(b"\nendstream\nendobj");
            v
        };
        let mut remap = HashMap::new();
        remap.insert(7u32, 1u32);
        remap.insert(2u32, 99u32);
        let out = renumber_object_bytes(&input, &remap);
        let s = std::str::from_utf8(&out).unwrap();
        // The stream body must be preserved verbatim.
        assert!(
            s.contains("2 0 R this looks like"),
            "stream body was incorrectly renumbered: {}",
            s
        );
    }

    #[test]
    fn renumber_preserves_unmapped_refs() {
        // References whose object number is not in remap should pass through unchanged.
        let input = b"5 0 obj\n<< /Font 99 0 R >>\nendobj";
        let mut remap = HashMap::new();
        remap.insert(5u32, 1u32);
        let out = renumber_object_bytes(input, &remap);
        let s = std::str::from_utf8(&out).unwrap();
        assert!(s.contains("99 0 R"), "unmapped ref was changed: {}", s);
    }

    // ── parse_ngr ─────────────────────────────────────────────────────────────

    #[test]
    fn parse_ngr_matches_reference() {
        let (n, _, kw) = parse_ngr(b"5 0 R ").unwrap();
        assert_eq!(n, 5);
        assert_eq!(kw, "R");
    }

    #[test]
    fn parse_ngr_matches_obj_header() {
        let (n, _, kw) = parse_ngr(b"10 0 obj\n").unwrap();
        assert_eq!(n, 10);
        assert_eq!(kw, "obj");
    }

    #[test]
    fn parse_ngr_rejects_partial_match() {
        // "5 0 Refer" — "R" not at a word boundary
        assert!(parse_ngr(b"5 0 Refer").is_none());
    }

    // ── merge_pdfs round-trip (internal) ──────────────────────────────────────

    #[test]
    fn merge_two_pdfs_round_trip() {
        let a_bytes = make_pdf(1);
        let b_bytes = make_pdf(2);

        let dir = std::env::temp_dir();
        let a_path = dir.join("merge_internal_a.pdf");
        let b_path = dir.join("merge_internal_b.pdf");
        let out_path = dir.join("merge_internal_out.pdf");

        std::fs::write(&a_path, &a_bytes).unwrap();
        std::fs::write(&b_path, &b_bytes).unwrap();

        merge_pdfs(&[&a_path, &b_path], &out_path, MergeOptions::default()).unwrap();

        let reader = crate::reader::PdfReader::open(&out_path).unwrap();
        assert_eq!(reader.page_count(), 3);
    }
}
