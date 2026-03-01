---
layout: default
title: PDF Read
parent: Features
---

# PDF Read

## Purpose

`PdfReader` allows opening an existing PDF file and inspecting its basic properties. This is the foundation for future features such as field extraction, form filling, and PDF merging.

The initial implementation supports the most common use case: **counting the number of pages** in a PDF and reading its version string.

## How It Works

PDF files are structured as:

1. **Header** — `%PDF-x.y` version declaration
2. **Body** — indirect objects (pages, fonts, images, etc.)
3. **Cross-reference table** — maps object numbers to byte offsets
4. **Trailer** — dictionary with `/Root` and `/Size`, followed by `startxref` and `%%EOF`

`PdfReader` parses these in reverse order (the recommended approach per the PDF spec):

1. Scan backward from the end of the file for `startxref` to get the xref table offset
2. Parse the xref table to build an `object number → byte offset` map
3. Parse the trailer dictionary to find the `/Root` (catalog) reference
4. Resolve the catalog object → follow `/Pages` reference
5. Resolve the pages object → read `/Count`

The raw bytes and xref map are retained on the `PdfReader` struct, ready for future object resolution.

## API

### Rust

```rust
use pdf_core::{PdfReadError, PdfReader};

// From a file
let reader = PdfReader::open("document.pdf")?;

// From bytes (e.g. from a network response or in-memory buffer)
let bytes: Vec<u8> = std::fs::read("document.pdf")?;
let reader = PdfReader::from_bytes(bytes)?;

// Inspect
println!("Pages: {}", reader.page_count());     // e.g. 42
println!("Version: {}", reader.pdf_version());  // e.g. "1.7"
```

### PHP

```php
// From a file
$reader = PdfReader::open("document.pdf");

// From bytes
$bytes = file_get_contents("document.pdf");
$reader = PdfReader::fromBytes($bytes);

// Inspect
echo $reader->pageCount();   // e.g. 42
echo $reader->pdfVersion();  // e.g. "1.7"
```

## Error Handling

`PdfReader::from_bytes()` and `PdfReader::open()` return `Result<PdfReader, PdfReadError>`.

| Error variant              | Meaning                                                                 |
|---------------------------|-------------------------------------------------------------------------|
| `NotAPdf`                 | The data does not begin with `%PDF-`                                    |
| `StartxrefNotFound`       | The `startxref` keyword is missing from the last 1024 bytes             |
| `MalformedXref`           | The xref table cannot be parsed                                         |
| `MalformedTrailer`        | The trailer dictionary is missing or lacks `/Root`                      |
| `XrefStreamNotSupported`  | The PDF uses a cross-reference stream (PDF 1.5+) — see Limitations      |
| `UnresolvableObject(n)`   | Object `n` referenced in the xref map cannot be parsed                  |
| `MalformedPageTree`       | The catalog or pages object is missing required entries                  |
| `Io(msg)`                 | A file I/O error occurred                                               |

## Design Decisions

### Reverse-parse approach

The PDF spec recommends starting from the end of the file because appended incremental updates push new xref tables and trailers toward the end. Starting from `startxref` ensures the most recent xref table is used.

### Retain raw bytes and xref map

`PdfReader` holds `data: Vec<u8>` and `xref: HashMap<u32, usize>` even though they are not currently exposed publicly. This is intentional: future issues for field extraction, annotation reading, or page merging will need to resolve arbitrary objects without re-reading the file.

### Flat dictionary parsing

The minimal dictionary parser extracts only `name → first-token` pairs. For indirect references (`N G R`), only the object number `N` is stored. This is sufficient for following the Catalog → Pages → Count chain. Nested dictionaries and arrays are skipped without error.

### No dependency on external crates

The parser is implemented in pure Rust with no additional dependencies. A full-featured PDF parsing crate (e.g., `lopdf`) was considered but would significantly increase the dependency footprint. The focused implementation is adequate for the current and anticipated near-term requirements.

## Limitations

- **Cross-reference streams (PDF 1.5+)**: PDFs that use xref streams instead of the traditional xref table are not supported. These files return `PdfReadError::XrefStreamNotSupported`. Many PDFs from Adobe Acrobat and LibreOffice use this format. Support is planned as a future issue.
- **Encrypted PDFs**: Not supported. Parsing an encrypted PDF will likely fail with `MalformedPageTree` or similar.
- **Incremental updates**: Only the most recent xref table (at `startxref`) is used. Earlier versions of an incrementally updated PDF are ignored, which is the correct behavior for reading the current document state.

## Internal Infrastructure (pub(crate))

Issue 27 added three `pub(crate)` methods used by the PDF Merge feature (Issue 28).
These are not part of the public API.

### `page_object_numbers() -> Result<Vec<u32>, PdfReadError>`

Walks the page tree (Catalog → Pages → Kids, recursively) and returns the object
number of every leaf `/Page` node in document order.

### `collect_closure(roots: &[u32]) -> Result<HashSet<u32>, PdfReadError>`

BFS from a seed set of object numbers through all indirect references (`N G R`)
reachable from those objects. Returns the complete transitive closure — every object
a page depends on (content streams, fonts, images, resource dicts, etc.).

### `raw_object_bytes(obj_num: u32) -> Result<&[u8], PdfReadError>`

Returns a zero-copy slice of the raw bytes for a single object, from `N G obj`
through (and including) `endobj`. Used by `collect_closure` and the merge function
to scan for references and to copy object data verbatim.

### Implementation notes

- **`skip_nested_dict` depth tracking**: The parser uses an `i32` depth counter that
  decrements-then-checks (not checks-then-decrements), so the outer closing `>>`
  is consumed correctly without accidentally consuming the enclosing structure's `>>`.
- **`resolve_kids` is bounded**: The search for `/Kids` is restricted to the current
  object's bytes (up to its `endobj`) to prevent matching `/Kids` from a later object
  in the file.
- **`extract_indirect_refs` tokenizer**: Scans for whitespace/delimiter-separated
  tokens and looks for `N G R` triplets. May include false positives from binary
  streams, which is harmless for closure computation (extra objects in the closure
  are simply orphaned during merge).

## History

- **Issue 26**: Initial implementation — `PdfReader::open()`, `PdfReader::from_bytes()`, `page_count()`, `pdf_version()`. PHP bindings via `PdfReader::open()` and `PdfReader::fromBytes()`.
- **Issue 27**: Added `pub(crate)` infrastructure for PDF merging: `page_object_numbers()`, `collect_closure()`, `raw_object_bytes()`. Fixed `skip_nested_dict` depth-tracking bug (was checking depth before decrementing, causing the enclosing dict's closing `>>` to be consumed). Added bounded search in `resolve_kids`.
