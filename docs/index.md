---
layout: default
title: Pivot PDF
---

# Pivot PDF

A PDF creation library written in Rust, designed to be used from any language. The core is implemented in Rust and can be used directly in Rust projects. Language bindings are being built for PHP, Python, Go, C#, and other major languages.

Designed for **low memory and CPU consumption** — even for documents with hundreds of pages — making it well suited for SaaS and web applications that generate reports, contracts, invoices, or bills of material on the fly.

[View on GitHub](https://github.com/pivotpdftools/pivot-pdf){: .btn}

---

## Features

| Feature | Status |
|---------|--------|
| Text placement | ✅ |
| TextFlow — word wrap and multi-page reflow | ✅ |
| 14 built-in PDF fonts | ✅ |
| TrueType font embedding | ✅ |
| Line graphics (paths, rectangles, stroke, fill) | ✅ |
| JPEG and PNG images | ✅ |
| Streaming tables with per-cell styles | ✅ |
| Page editing (post-write overlay, page numbering) | ✅ |
| FlateDecode compression (50–80% size reduction) | ✅ |
| PHP extension | ✅ |

---

## Documentation

### Features

- [Stream Compression](features/compression) — FlateDecode compression for all stream objects
- [Images](features/images) — Place JPEG and PNG images with fit, fill, stretch, and none modes
- [Line Graphics](features/line-graphics) — Paths, rectangles, stroke, fill, and color
- [Tables](features/tables) — Streaming row-by-row layout with per-cell styles and overflow modes
- [TrueType Fonts](features/truetype-fonts) — Embed `.ttf` files with full Unicode support
- [Page Editing](features/edit-page) — Open completed pages for overlay content (e.g. "Page X of Y")

### PHP Extension

- [PHP Installation Guide](php-installation) — Composer package, binary download, php.ini config, and verification

---

## Quick Start

```rust
use pdf_core::{BuiltinFont, FontRef, PdfDocument, TextFlow, TextStyle, Rect};

let mut doc = PdfDocument::create("output.pdf")?;
doc.set_info("Title", "My Document");

let font = doc.load_builtin_font(BuiltinFont::Helvetica)?;
let style = TextStyle::new(FontRef::Builtin(font), 12.0);

let mut tf = TextFlow::new();
tf.add_text("Hello, Pivot PDF!", &style);

doc.begin_page(612.0, 792.0);
doc.fit_textflow(&mut tf, Rect::new(72.0, 72.0, 468.0, 680.0))?;
doc.end_page()?;
doc.end_document()?;
```

---

## Language Bindings

| Language | Status |
|----------|--------|
| Rust | ✅ Available |
| PHP (Linux, macOS) | ✅ Available |
| PHP (Windows) | 🔲 Planned |
| CLI | 🔲 Planned |
| Python | 🔲 Planned |
| Go | 🔲 Planned |
| C# | 🔲 Planned |

---

## License

[MIT](https://github.com/pivotpdftools/pivot-pdf/blob/main/LICENSE)
