# TrueType Font Embedding

## Purpose

Allows users to load `.ttf` font files and use them for text placement and text flows, with full Unicode support. Embedded TrueType fonts render identically across all PDF viewers since the font data travels with the document.

## How It Works

### Font Loading

Users load a TrueType font via `load_font_file()` or `load_font_bytes()`, which returns a `FontRef::TrueType(id)`. This `FontRef` is used in `TextStyle` just like builtin fonts:

```rust
let tt_font = doc.load_font_file("fonts/Roboto-Regular.ttf")?;
let style = TextStyle { font: tt_font, font_size: 14.0 };
```

The loading process parses the `.ttf` file with `ttf-parser` and extracts:
- Font name and PostScript name (from `name` table)
- Units per em, ascent, descent, bounding box (from `head`, `hhea`, `OS/2` tables)
- Character-to-glyph mapping (from `cmap` table)
- Glyph advance widths (from `hmtx` table)
- Font descriptor metadata (flags, cap height, stem V, italic angle)

### Unified Font System (`FontRef`)

The `FontRef` enum provides a unified reference for both font types:

```
FontRef::Builtin(BuiltinFont)   -- 14 standard PDF fonts (no embedding)
FontRef::TrueType(TrueTypeFontId) -- loaded .ttf fonts (embedded)
```

`TextStyle.font` uses `FontRef`, replacing the previous `BuiltinFont` field. This is a breaking change from Issue 7 but provides a clean unified API.

### Text Encoding: Builtin vs TrueType

| Aspect | Builtin | TrueType |
|--------|---------|----------|
| Content stream | `(Hello) Tj` | `<00480065006C006C006F> Tj` |
| Encoding | Single-byte Latin | 2-byte glyph IDs (Identity-H) |
| PDF font type | `/Subtype /Type1` | `/Subtype /Type0` composite |

TrueType text is hex-encoded using glyph IDs looked up from the font's `cmap` table. Each character becomes a 4-hex-digit glyph ID.

### PDF Object Structure (5 Objects per Font)

```
Page /Resources /Font /F15 --> Type0 Font dict
    /DescendantFonts --> [CIDFontType2 dict]
        /FontDescriptor --> FontDescriptor dict
            /FontFile2 --> stream (raw .ttf data)
    /ToUnicode --> CMap stream
```

1. **Type0 Font** - Top-level entry in page resources. Uses `/Encoding /Identity-H`.
2. **CIDFontType2** - Describes the TrueType CID font. Contains `/W` widths array.
3. **FontDescriptor** - Metadata: ascent, descent, bbox, flags, etc.
4. **FontFile2** - The raw `.ttf` binary embedded as a stream.
5. **ToUnicode CMap** - Maps glyph IDs back to Unicode for copy/paste support.

### Deferred Object Writing

TrueType font PDF objects are written during `end_document()`, not `end_page()`. This is because:
- The `/W` array grows as more glyphs are used across pages
- The ToUnicode CMap grows similarly
- Page resource dictionaries reference pre-allocated `ObjId`s
- PDF xref tables resolve object locations regardless of write order

### Measurement Dispatch

Text measurement dispatches based on `FontRef` variant:
- `FontRef::Builtin` -> `FontMetrics::measure_text()` (static width tables)
- `FontRef::TrueType` -> `TrueTypeFont::measure_text()` (per-font width data)

## Design Decisions

### Why Type0/CIDFontType2 (not simple TrueType)?

Simple TrueType fonts in PDF are single-byte encoded (max 256 glyphs). The Type0/CIDFontType2 composite structure supports multi-byte encoding for full Unicode coverage. This is the standard approach for modern PDF generators.

### Why full embedding (no subsetting)?

Font subsetting strips unused glyphs to reduce file size (757KB -> ~20KB typical). However, subsetting adds significant complexity. Full embedding is correct and simple. Subsetting is planned for Phase 3 using the `subsetter` crate.

### Why `FontRef` enum instead of a generic handle?

The enum makes the distinction between builtin and TrueType explicit at the type level. Callers dispatch on the variant for encoding and measurement. A generic handle would require runtime lookups and lose type safety.

### Why pre-allocated ObjIds?

Page resource dictionaries are written during `end_page()`, but TrueType font objects are written during `end_document()`. Pre-allocating ObjIds during `end_page()` allows the page to reference font objects that don't exist yet. The PDF xref table resolves this since it maps object numbers to byte offsets regardless of write order.

## Configuration

No configuration options. Fonts are loaded and used directly.

## Limitations

- **No font subsetting** - Full `.ttf` file is embedded, making PDFs larger than necessary. Planned for Phase 3.
- **No compression** - Font file stream is uncompressed. FlateDecode compression planned for Phase 3.
- **No OpenType/OTF support** - Only `.ttf` files are supported. `.otf` files with CFF outlines would need CIDFontType0 handling.
- **No font fallback** - Characters not in the font's cmap produce the `.notdef` glyph (typically a rectangle).

## PHP Extension

```php
$handle = $doc->loadFontFile("fonts/Roboto-Regular.ttf");
$style = TextStyle::truetype($handle, 14.0);

$tf = new TextFlow();
$tf->addText("TrueType text", $style);
```

The font handle is an integer index. `TextStyle::truetype()` creates a style for TrueType fonts, while the regular constructor continues to accept builtin font names as strings.

## History

- **Issue 8** (2026-02-14): Initial implementation. Full TrueType embedding via Type0/CIDFontType2 composite structure. No subsetting or compression.
- **Issue 6**: Research phase that defined the API sketch and PDF structure requirements.
