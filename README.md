# Pivot PDF

A PDF creation library written in Rust, designed to be used from any language. The core is implemented in Rust and can be used directly in Rust projects. Language bindings are being built for PHP, Java, C#, Python, Go, and other major languages.

Designed for low memory and CPU consumption — even for documents with hundreds of pages — making it well suited for SaaS and web applications that generate reports, contracts, invoices, or bills of material on the fly.

## Features

- **Text placement** — place text at arbitrary (x, y) positions using any supported font
- **TextFlow** — automatic word wrap and multi-page text reflow with mixed font styles
- **14 built-in fonts** — all standard PDF fonts (Helvetica, Times, Courier, Symbol, ZapfDingbats) with no embedding required
- **TrueType font embedding** — load `.ttf` files for full Unicode text with automatic metrics extraction
- **Line graphics** — move/lineto paths, rectangles, stroke, fill, fill-stroke, color and line-width control
- **Images** — place JPEG and PNG images (with alpha) using fit, fill, stretch, or none fit modes
- **Tables** — streaming row-by-row table layout with per-cell styles, overflow modes, borders, and background colors
- **Page editing** — open completed pages for overlay content (e.g. "Page X of Y" numbering)
- **Compression** — optional FlateDecode compression for all stream objects (typically 50–80% size reduction)
- **PHP extension** — full PHP binding exposing all features via a native extension

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

All coordinates are in PDF points (1 pt = 1/72 inch) with the origin at the **bottom-left** of the page. A standard US Letter page is 612 × 792 pt.

## License

[MIT](LICENSE)
