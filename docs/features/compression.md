# Stream Compression

## Purpose

PDF files produced by this library can contain large stream objects — page content, embedded TrueType font files, and ToUnicode CMaps. Without compression, documents with many pages or embedded fonts can become unnecessarily large. FlateDecode compression (zlib/deflate) typically reduces stream sizes by 50-80%.

## How It Works

Compression is controlled by a single boolean toggle on `PdfDocument`:

```rust
let mut doc = PdfDocument::create("output.pdf")?;
doc.set_compression(true);
```

When enabled, all stream objects are compressed with FlateDecode before being written. The `/Filter /FlateDecode` entry is added to each stream's dictionary so PDF readers know to decompress when reading.

Compression is **disabled by default** to maintain backward compatibility and to make debugging easier (uncompressed streams are human-readable).

## What Gets Compressed

Only stream objects can be compressed in PDF. The library produces three types:

| Stream Type | Source | Typical Savings |
|-------------|--------|-----------------|
| Page content | Text operators, graphics commands | Moderate (repetitive text commands) |
| FontFile2 | Embedded .ttf font data | High (binary font data) |
| ToUnicode CMap | Unicode mapping for TrueType fonts | Moderate (structured text) |

Non-stream objects (dictionaries, arrays, references) are not affected.

## Design Decisions

### FlateDecode Only

PDF supports multiple compression filters (LZWDecode, ASCIIHexDecode, ASCII85Decode, etc.). We use only FlateDecode because:

- It's the universal standard — supported by every PDF reader
- Best compression ratio among PDF filters
- `flate2` crate is well-maintained and efficient
- No practical reason to support alternative filters

### Default Compression Level

The `flate2` crate's default compression level (level 6) is used. This provides a good balance of compression ratio vs. CPU time. We don't expose a compression level setting because the difference between levels is marginal for typical PDF content.

### Single Toggle, Not Per-Stream

A single `set_compression(bool)` controls all streams rather than per-stream settings. This keeps the API simple. There's no practical use case for compressing some streams but not others.

## API

### Rust

```rust
pub fn set_compression(&mut self, enabled: bool) -> &mut Self
```

Builder-style method matching the existing pattern (`set_info`, etc.).

### PHP

```php
$doc->setCompression(true);
```

## Limitations

- Compressed streams are not human-readable (use a PDF inspection tool to debug)
- Compression adds minimal CPU overhead during PDF generation
- No font subsetting yet — full font files are embedded and compressed
