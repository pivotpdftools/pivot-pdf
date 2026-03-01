---
layout: default
title: PDF Merge
parent: Features
---

# PDF Merge

## Purpose

`merge_pdfs` combines two or more existing PDF files into a single output file.
Pages from each source are appended in document order: all pages from the first
source, then all pages from the second, and so on.

This is useful for assembling multi-part documents — for example, combining a
cover page, a body report, and an appendix that were generated separately.

## How It Works

The merge operation relies on the `pub(crate)` infrastructure added in Issue 27:

1. **Open each source** with `PdfReader`.
2. **Walk the page tree** (`page_object_numbers`) to get the leaf page object
   numbers in document order.
3. **Collect each page's object closure** (`collect_closure`) — a BFS from the
   page node through all indirect references, gathering every object the page
   depends on: content streams, resource dictionaries, fonts, images, etc.
4. **Assign new output IDs** from a global counter starting at 1. All objects
   from all sources are assigned unique IDs, preventing any conflicts.
5. **Build a per-source remapping table** (`source_obj_num → output_obj_num`).
6. **Copy and renumber** each object: extract its raw bytes with
   `raw_object_bytes`, then scan for `N G R` (indirect reference) and `N G obj`
   (object header) patterns and substitute using the remapping table. Stream
   bodies are copied verbatim to preserve compressed binary content.
7. **Write a new Pages tree** listing the remapped page object numbers in order,
   then a new **Catalog** pointing to that Pages tree.
8. **Write the xref table and trailer**, referencing the new Catalog as `/Root`.

### Why copy raw bytes rather than re-parsing?

Re-serialising objects would require a full PDF object model. Copying raw bytes
and rewriting only the integer tokens that appear in reference patterns is far
simpler and avoids introducing a dependency on a full PDF parsing library. The
only tokens that must change are object numbers; all other content (stream
operators, name dictionaries, encoding tables) is copied byte-for-byte.

### Object closure and orphaned nodes

`collect_closure` is seeded with the leaf page objects. Because each page
references its parent Pages node (`/Parent N G R`), the source's Pages tree
nodes are included in the closure and copied to the output. These copied nodes
are not referenced by the new merged Catalog and are effectively orphaned — they
waste a small amount of space but do not affect correctness for any PDF operation
that follows the standard Catalog → Pages → Kids traversal.

## API

### Rust

```rust
use pdf_core::{merge_pdfs, MergeOptions, PdfMergeError};

merge_pdfs(
    &["report.pdf", "appendix.pdf"],
    "combined.pdf",
    MergeOptions::default(),
)?;
```

### PHP

```php
$opts = new MergeOptions(); // flattenForms defaults to false
merge_pdfs(['report.pdf', 'appendix.pdf'], 'combined.pdf', $opts);
```

## MergeOptions

| Field          | PHP property   | Default | Description                                               |
|----------------|----------------|---------|-----------------------------------------------------------|
| `flatten_forms`| `flattenForms` | `false` | Flatten interactive form fields. **Not yet implemented.** |

Setting `flatten_forms = true` returns `PdfMergeError::NotSupported` (Rust) or
throws an exception (PHP). Full support is deferred until form field reading and
writing are implemented.

## Error Handling

`merge_pdfs` returns `Result<(), PdfMergeError>`.

| Variant                      | Meaning                                                    |
|------------------------------|------------------------------------------------------------|
| `NotSupported`               | An unsupported option was requested (e.g. `flatten_forms`) |
| `ReadError(PdfReadError)`    | A source PDF could not be read or parsed                   |
| `Io(String)`                 | The output file could not be written                       |

## Limitations

- **Cross-reference streams (PDF 1.5+)**: Sources using xref streams return
  `PdfMergeError::ReadError(PdfReadError::XrefStreamNotSupported)`. This
  affects many PDFs from Adobe Acrobat and LibreOffice.
- **Encrypted PDFs**: Not supported.
- **Page range selection**: All pages from each source are included. Selecting a
  subset of pages is a future issue.
- **Form field merging**: `flatten_forms = true` is not yet implemented.

## Examples

- **Rust**: `examples/rust/generate_merge.rs` — merges `rust-tables.pdf` and
  `rust-invoice.pdf`, prints the merged page count.
- **PHP**: `examples/php/generate_merge.php` — mirrors the Rust example.

```bash
# Rust
cargo run --example generate_tables -p pdf-examples
cargo run --example generate_invoice -p pdf-examples
cargo run --example generate_merge -p pdf-examples

# PHP
php -d extension=target/release/libpdf_php.so examples/php/generate_tables.php
php -d extension=target/release/libpdf_php.so examples/php/generate_invoice.php
php -d extension=target/release/libpdf_php.so examples/php/generate_merge.php
```

## Design Decisions

### Global sequential ID assignment

All source objects are renumbered from a single global counter. This guarantees
no two objects from different sources share an ID in the output, without needing
to scan for conflicts.

### Stream bodies are not scanned for references

Binary-compressed stream content may accidentally contain byte sequences that
look like `N G R`. Scanning stream content for references would corrupt it.
The renumber pass detects the `stream` keyword (at a word boundary) and copies
everything up to `endstream` verbatim. This is safe because real indirect
references can only appear in the object's dictionary, not inside the stream
body.

### Generation numbers reset to 0

All output objects use generation number 0. PDFs with objects at generation > 0
(the result of incremental updates that delete and reuse object numbers) are rare
in practice, and resetting to 0 is always valid for a freshly written PDF.

## History

- **Issue 28**: Initial implementation — `merge_pdfs`, `MergeOptions`,
  `PdfMergeError`. PHP bindings via `merge_pdfs()` function and `MergeOptions`
  class. Depends on `pub(crate)` infrastructure from Issue 27.
