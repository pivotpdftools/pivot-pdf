# Pivot PDF

A PDF creation library written in Rust, designed to be used from any language. The core is implemented in Rust and can be used directly in Rust projects. Language bindings are being built for PHP, Java, C#, Python, Go, and other major languages.

Designed for low memory and CPU consumption — even for documents with hundreds of pages — making it well suited for SaaS and web applications that generate reports, contracts, invoices, or bills of material on the fly.

See Full Documentation: https://pivotpdftools.github.io/pivot-pdf/

## Features

- **Text placement** — place text at arbitrary (x, y) positions using any supported font
- **TextFlow** — automatic word wrap and multi-page text reflow with mixed font styles
- **Word-break control** — configurable overflow handling for long tokens: force-break, hyphenate, or allow overflow; applies to both TextFlow and table cells
- **14 built-in fonts** — all standard PDF fonts (Helvetica, Times, Courier, Symbol, ZapfDingbats) with no embedding required
- **TrueType font embedding** — load `.ttf` files for full Unicode text with automatic metrics extraction
- **Line graphics** — move/lineto paths, rectangles, stroke, fill, fill-stroke, color and line-width control
- **Images** — place JPEG and PNG images (with alpha) using fit, fill, stretch, or none fit modes
- **Tables** — streaming row-by-row table layout with per-cell styles, overflow modes, borders, and background colors
- **Coordinate origin** — choose between bottom-left (PDF native) or top-left (screen/web style) coordinates at document creation time; the library transparently converts
- **Form fields** — fillable AcroForm text fields at specified positions; uniqueness enforced across the document
- **Page editing** — open completed pages for overlay content (e.g. "Page X of Y" numbering)
- **Compression** — optional FlateDecode compression for all stream objects (typically 50–80% size reduction)
- **PDF reading** — open an existing PDF to inspect page count and version string
- **PDF merging** — combine two or more PDF files into a single output, pages appended in order
- **PHP extension** — full PHP binding exposing all features via a native extension

## PHP Installation

Install the Composer package for IDE autocompletion stubs:

```bash
composer require pivot-pdf/pivot-pdf
```

The native extension binary must be installed separately. For the full installation
guide, see [docs/php-installation.md](docs/php-installation.md).

## Requirements

### Rust (pdf-core, pdf-cli)

- Rust stable toolchain — install via [rustup](https://rustup.rs)

### PHP Extension (pdf-php)

- Rust stable toolchain (same as above)
- PHP development headers: `sudo apt install php-dev`
- Clang development libraries: `sudo apt install libclang-dev`

## Building

```bash
# Build all workspace members (debug)
cargo build

# Build release (recommended for PHP extension)
cargo build --release
```

## Running Tests

```bash
# Run all Rust tests
cargo test
```

## Rust Examples

Examples write output PDFs to the `examples/output/` directory.

```bash
# Basic document — place_text
cargo run --example generate_sample -p pdf-examples

# TextFlow — multi-page text reflow with mixed font styles
cargo run --example generate_textflow -p pdf-examples

# Line graphics — paths, rectangles, stroke, fill
cargo run --example generate_graphics -p pdf-examples

# Images — JPEG and PNG placement
cargo run --example generate_images -p pdf-examples

# Tables — streaming row layout with headers
cargo run --example generate_tables -p pdf-examples

# TrueType fonts — embed a .ttf font
cargo run --example generate_truetype -p pdf-examples

# Page numbers — edit completed pages to add "Page X of Y"
cargo run --example generate_page_numbers -p pdf-examples

# Form fields — fillable AcroForm text fields
cargo run --example generate_form_fields -p pdf-examples

# Merge — combine multiple PDFs into one
cargo run --example generate_merge -p pdf-examples
```

## PHP Extension

### Build

```bash
cargo build --release -p pdf-php
```

The compiled extension is at `target/release/libpdf_php.so`.

### Run Tests

```bash
php -d extension=target/release/libpdf_php.so pdf-php/tests/test.php
```

### Run PHP Examples

Each PHP example mirrors its Rust counterpart and writes to the `examples/output/` directory.

```bash
EXT="-d extension=target/release/libpdf_php.so"

php $EXT examples/php/generate_sample.php
php $EXT examples/php/generate_textflow.php
php $EXT examples/php/generate_graphics.php
php $EXT examples/php/generate_images.php
php $EXT examples/php/generate_tables.php
php $EXT examples/php/generate_truetype.php
php $EXT examples/php/generate_page_numbers.php
php $EXT examples/php/generate_form_fields.php
php $EXT examples/php/generate_merge.php
```

IDE type hints and autocompletion are provided by `pdf-php/pdf-php.stubs.php`.

## Workspace Structure

```
pivot-pdf/
├── pdf-core/          # Core library — all PDF generation logic
│   ├── src/
│   └── tests/
├── pdf-php/           # PHP extension wrapping pdf-core
│   ├── src/
│   └── tests/
├── examples/          # Rust and PHP example programs
│   ├── rust/
│   ├── php/
│   └── output/        # Generated PDFs (git-ignored)
└── docs/              # Architecture and feature documentation
```

## Coordinate System

All coordinates are in PDF points (1 pt = 1/72 inch). A standard US Letter page is 612 × 792 pt.

By default, the origin is at the **bottom-left** of the page with y increasing upward (PDF native). Optionally, you can use a **top-left** origin with y increasing downward — more natural for screen/web contexts.

Pass `DocumentOptions` when creating a document:

```rust
use pivot_pdf::{DocumentOptions, Origin, PdfDocument};

// Top-left (screen/web style)
let opts = DocumentOptions { origin: Origin::TopLeft };
let mut doc = PdfDocument::create("out.pdf", opts)?;

// Bottom-left (PDF native, the default)
let mut doc = PdfDocument::create("out.pdf", DocumentOptions::default())?;
```

In PHP:

```php
$opts = new PdfDocumentOptions();
$opts->origin = 'top-left';   // 'bottom-left' (default) or 'top-left'
$doc = PdfDocument::create("out.pdf", $opts);
```

The origin setting applies uniformly to every coordinate-taking API (`place_text`, `fit_textflow`, `fit_row`, `place_image`, `move_to`, `line_to`, `rectangle`, and form fields). See [docs/features/coordinate-origin.md](docs/features/coordinate-origin.md) for full details.

## License

[MIT](LICENSE)
