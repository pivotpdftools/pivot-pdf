# Pivot PDF — Roadmap

Pivot PDF is a PDF library targeting SaaS and web applications. The primary focus is **generating** PDF documents efficiently — reports, contracts, invoices, and similar output. The library will also add support for **reading** existing PDFs, enabling field extraction, data parsing, and merging multiple documents into one.

This roadmap outlines what has been implemented, what is planned, and what is intentionally out of scope.

---

## Feature Matrix

### Core Document

| Feature | Status | Notes |
|---------|--------|-------|
| Create PDF 1.7 | ✅ Implemented | |
| Text placement (`place_text`) | ✅ Implemented | Fixed position with font/size |
| TextFlow (word wrap + reflow) | ✅ Implemented | Multi-page, mixed font styles |
| FlateDecode compression | ✅ Implemented | ~50–80% size reduction |
| Page editing (post-write overlay) | ✅ Implemented | Used for "Page X of Y" |
| Coordinate origin (top-left / bottom-left) | ✅ Implemented | Transparent transform via `DocumentOptions` |

### Fonts

| Feature | Status | Notes |
|---------|--------|-------|
| 14 standard built-in fonts | ✅ Implemented | Helvetica, Times, Courier, Symbol, ZapfDingbats families |
| TrueType font embedding | ✅ Implemented | Full `.ttf` with Unicode via Type0/CIDFont |
| Font subsetting | 🔲 Planned | Reduce embedded font size from ~1–20 MB to ~5–50 KB |
| OpenType / variable fonts | 🔲 Future | Depends on demand |

### Text

| Feature | Status | Notes |
|---------|--------|-------|
| Word wrap | ✅ Implemented | Breaks on whitespace |
| Word break (long words) | ✅ Implemented | Force-break at character boundary; optional hyphen |
| Mixed font styles in one flow | ✅ Implemented | |
| Right-to-left text (RTL) | 🔲 Future | Arabic, Hebrew — complex, low priority for now |
| Vertical text | 🔲 Future | Japanese/CJK — complex, low priority for now |
| Multi-column text | 🔲 Future | |

### Graphics

| Feature | Status | Notes |
|---------|--------|-------|
| Line paths (moveto, lineto, stroke) | ✅ Implemented | |
| Rectangles | ✅ Implemented | |
| Fill and fill-stroke | ✅ Implemented | |
| Color (RGB, gray) | ✅ Implemented | |
| Line width | ✅ Implemented | |
| Bezier curves | ✅ Implemented | `curve_to` — PDF `c` operator with origin transform |
| Arcs and circles | ✅ Implemented | `arc` and `circle` — approximated with cubic Bezier segments |
| Gradients (shading) | 🔲 Future | Complex — PDF shading patterns |
| Patterns and hatching | 🔲 Future | |

### Images

| Feature | Status | Notes |
|---------|--------|-------|
| JPEG images | ✅ Implemented | |
| PNG images (with alpha) | ✅ Implemented | |
| SVG images | 🔲 Future | Requires SVG rendering/rasterization |
| WebP, AVIF | 🔲 Future | Low demand currently |

### Layout

| Feature | Status | Notes |
|---------|--------|-------|
| Tables (streaming, row-by-row) | ✅ Implemented | Per-cell styles, overflow modes, borders, backgrounds, text alignment, column span |
| Table cell word break | ✅ Implemented | Force-break at character boundary; optional hyphen |
| Headers and footers (built-in) | 🔲 Planned | Repeated content registered once, applied each page |
| Multi-column layout | 🔲 Future | |

### Document Features

| Feature | Status | Notes |
|---------|--------|-------|
| Hyperlinks | 🔲 Planned | URI annotations — common in reports |
| Bookmarks / outline / TOC | 🔲 Planned | Navigation in long documents |
| PDF/A compliance | 🔲 Planned | Regulatory requirement — needs font embedding, metadata, colorspace conformance |
| Forms and interactive fields | 🔲 Planned | AcroForm text fields implemented; checkboxes, dropdowns future |
| Encryption / password protection | 🔲 Future | |
| Digital signatures | 🔲 Future | |
| Barcodes / QR codes | 🔲 Future | Could be implemented as an image or native vectors |

### PDF Reading and Manipulation

| Feature | Status | Notes |
|---------|--------|-------|
| Read / parse PDF | ✅ Implemented | Basic: page count + version. xref streams (PDF 1.5+) planned |
| Extract form fields | 🔲 Future | Depends on read/parse |
| Merge multiple PDFs | ✅ Implemented | Traditional xref only; xref streams (PDF 1.5+) not yet supported |
| Split PDF | 🔲 Future | Depends on read/parse |
| OCR | ❌ Out of scope | Requires a full OCR pipeline; use a dedicated tool |
| Multimedia (audio/video) | ❌ Out of scope | Not relevant to the target use case |
| JavaScript | ❌ Out of scope | Security concern; unsuitable for server-side generation |

---

## Priority Tiers

Priorities are informed by the core use case: **server-side PDF generation for SaaS/web applications**.

### Tier 1 — Next Up

These directly address known gaps in the core generation loop:

1. ~~**Bezier curves and arcs**~~ — ✅ Implemented in Issue 36.
2. **Font subsetting** — Embedded TrueType fonts can be 1–20 MB. Subsetting cuts this to 5–50 KB, which matters for any document with embedded fonts.
3. **Hyperlinks** — URI annotations are commonly needed in generated reports and invoices.

### Tier 2 — Near Term

4. **Bookmarks / outline** — Important for long multi-section documents (contracts, manuals).
5. **Headers and footers** — Common pattern; currently users must implement this manually via `open_page`.
6. **PDF/A compliance** — Required for legal/archival use cases. Primarily a metadata and font-embedding constraint.
7. **More language bindings** — See Language Binding Roadmap below.

### Tier 3 — Future

8. **Forms and interactive fields** — Needed for fillable PDFs (onboarding forms, applications).
9. **Encryption** — Required when PDFs contain sensitive data.
10. **Digital signatures** — Required for legally binding e-documents.
11. **Multi-column text** — Useful for newsletters, academic papers.
12. **Gradients and shading** — Useful for polished reports.
13. **Read / parse PDF** — Foundation for field extraction and merging. This is a significant undertaking and planned as a future phase after the creation features are mature.
14. **Extract form fields** — Depends on read/parse.
15. **Merge multiple PDFs** — Implemented in Issue 28.

---

## Language Binding Roadmap

| Language | Status | Notes |
|----------|--------|-------|
| Rust | ✅ Available | Native — this is the core library |
| PHP (Linux) | ✅ Available | Release artifacts built via CI for PHP 8.3–8.5 |
| PHP (macOS) | ✅ Available | Release artifacts built via CI for PHP 8.3–8.5 |
| PHP (Windows) | 🔲 Planned | `ext-php-rs` requires nightly Rust + MSVC link-time symbol resolution; CI not yet stable — tracked separately |
| CLI | 🔲 Planned | `pdf-cli` workspace member exists; needs a full command-line interface |
| Python | 🔲 Planned | `PyO3` for Rust/Python bindings |
| Go | 🔲 Planned | CGO binding or pure Go wrapper |
| C# | 🔲 Planned | Interop via native Rust shared library |
| Java | 🔲 Future | JNI or JNA |
| Node.js / WASM | 🔲 Future | `wasm-bindgen` — good fit for browser-side generation |

Priority order: **PHP (Windows) → CLI → Python → Go → C# → Java → Node.js**.

PHP Windows support is prioritized because PHP developers on Windows targeting Linux deployments need a local build path. Python is next due to its widespread use in data pipelines and report generation.

---

## Examples

### Current Examples

| Example | Rust | PHP |
|---------|------|-----|
| Basic text placement | ✅ | ✅ |
| TextFlow (multi-page reflow) | ✅ | ✅ |
| Line graphics | ✅ | ✅ |
| Images (JPEG + PNG) | ✅ | ✅ |
| Tables (streaming) | ✅ | ✅ |
| TrueType fonts | ✅ | ✅ |
| Page numbers (edit page) | ✅ | ✅ |
| Large PDF from database (Sakila) | ✅ | ✅ |
| Fake invoice | ✅ | ✅ |
| Curves and arcs | ✅ | ✅ |

### Planned Examples

| Example | Purpose |
|---------|---------|
| Letter / cover letter | Demonstrates mixed text blocks, fonts, and spacing for a professional document |
| Report with charts | Demonstrates tables, graphics, and layout working together |

### Committing Example Output PDFs

Example output PDFs are currently `.gitignore`d. There is value in committing a reference set to the repository — it allows visual regression testing and gives new contributors something to compare against. This is tracked as a separate decision; the approach would be to add a curated set of reference PDFs to a `examples/reference/` directory that is committed.

---

## Performance and Benchmarking

The library is designed for low memory and CPU usage. However, no formal benchmark suite exists yet.

Planned benchmark scenarios:
- **Throughput**: Documents per second at various page counts (10, 100, 1000 pages)
- **Memory**: Peak RSS when generating a 1000-page document with streaming
- **File size**: Compressed vs. uncompressed PDF output at various content densities
- **Font embedding**: Cost of embedding TrueType fonts (with and without subsetting)

A benchmark suite using Rust's `criterion` crate is planned as a separate effort.

---

## Known Issues / Technical Debt

| Item | Description |
|------|-------------|
| Full font embedding | TrueType fonts are embedded in full; subsetting is not yet implemented |
| Per-page font resources | All loaded fonts are declared in every page's resource dict, even if unused on that page |
| Standard font availability | The 14 standard fonts are used without embedding; PDF 2.0 deprecated this guarantee |
| PHP Windows build | `ext-php-rs` requires nightly Rust and MSVC link-time PHP symbol resolution on Windows; version skew between `setup-php` and `windows.php.net` dev pack availability causes persistent LNK2019 linker failures in CI |

---

## What Will Not Be Supported

The following are explicitly out of scope:

- **OCR** — Requires a full OCR pipeline; use a dedicated tool (e.g., Tesseract)
- **Multimedia** (audio, video, 3D) — Not relevant to the target use case
- **JavaScript** — PDF JavaScript is a security concern and unsuitable for server-side generation

