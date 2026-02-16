# Issue 1: Basic PDF Creation
## Description
Create enough code in the pdf-core to produce a minimal pdf that can be opened by any pdf viewer and have no errors. A generated pdf will use a default version 1.7.

IMPORTANT: The design must support the ability for a consumer to flush to the disk after x number of pages written. This is to allow low memory usage by allowing page data to be freed. However, for small files with the need to stream the result, file name can be omitted and disk flushing not an option in this case.

The basic design sketch in psuedo code:
```ts
let p = new PdfDocument()
p.beginDocument(file: "example.pdf")
p.setInfo("Creator", "rust-pdf")
p.setInfo("Title", "A Test Document")

//width/height as pt units.
p.beginPage(width: 612, height: 792, {
    origin: Origin.BottomLeft //this will be default
})

p.place_text("Hello", x: 20, y: 20) //will use default helvetica as other fonts are not yet supported

p.engPage() //not sure if this is needed

p.endDocument() //not sure if this is needed but we need some way to flush the content to the file specificed in beginDocument
```
There should be a way to generate a document like sketched above and output it somewhere so the user can view it.

## Tasks
- [x] Fix Cargo.toml and create project skeleton
- [x] Implement PDF object types (objects.rs)
- [x] Implement PDF binary writer (writer.rs)
- [x] Implement high-level API (document.rs)
- [x] Integration test — produce and validate a real PDF

## Status
complete

---

# Issue 2: Improve test organization
## Description
The previous Issue ogranized tests with some in the tests directory and others in the same file as the code. All tests should be organized in the tests directory.

## Tasks
- [x] Identify the tests to move
- [x] Move the tests

## Status
complete

---

# Issue 3: TextFlows
## Description
A critical feature of this library is TextFlows. TextFlows:
- Allow text to be placed within the specified boundries using different methods:
  - None (Default) - will place the text until the bounding box is full
  - Shrink - scale the font down until it fits within the bounding box or reaches a minimum value. Note, we are currently working only with Helvetica font but any font should be considered
  - Clip - Clip the
- Allow text styling such as "The __bold__ brown fox"

The design will expand over time but this is intended to be the minimum implementation to demonstrate whether it works.

Psuedo Code Usage Sketch
```rs
let path = "sample_output.pdf";
let mut doc = PdfDocument::create(path).unwrap();
doc.set_info("Creator", "rust-pdf");
doc.set_info("Title", "A Test Document");

let mut text_flow = doc.create_textflow("Lorum Ipsum ...Long text to flow 2 pages")
text_flow.add_textflow("bold text", {font_weigth: 900})
text_flow.add_textflow("a little more normal text. The end.")

loop {
    //the start of each loop creates a new page
    doc.begin_page(612.0, 792.0);

    //The fit_textflow result will help inform the loop what to do
    let result = doc.fit_textflow(text_flow, x: 72.0, y: 720.0, width: 100, height: 100);

    //Stop: All the text has been fit and there is no more
    //BoxFull: The box is full and not all the text has been placed
    //BoxEmpty: 
    match result {
        FitResult::Stop => break,
        FitResult::BoxFull => continue,
        FitResult::BoxEmpty => {
            eprintln!("Warning: bounding box too small");
            break;
        }
    }
}
doc.end_page().unwrap();
doc.end_document().unwrap();
```

## Tasks
- [x] Evaluate the psuedo code above and offer design suggestions.
- [x] Plan the remaining steps
- [x] Create fonts.rs with Helvetica metrics
- [x] Add Helvetica-Bold font to document
- [x] Create textflow.rs with data structures
- [x] Implement word wrapping and content stream generation
- [x] Integrate with PdfDocument via fit_textflow
- [x] End-to-end tests

## Status
complete

---

# Issue 4: PHP Extension
## Description
There is enough functionality in the core to proceed with the creation of the php extension. Most likely, there will be a directory, pdf-php or pdf-php-ext.

## Tasks
- [x] Project scaffolding — Create pdf-php directory, Cargo.toml, minimal lib.rs, add to workspace
- [x] Implement TextStyle and Rect PHP classes with constructors and to_core helpers
- [x] Implement TextFlow PHP class wrapping pdf_core::TextFlow
- [x] Implement PdfDocument PHP class with DocumentInner enum and all methods
- [x] Integration test — Create test.php, build release, run with php

## Status
complete

---

# Issue 5: Space is not honored between text flows (bug)
## Description
When two text flows are added and the first one has a space at the end of the string, the result is missing the space between the two textflows.
For example, the following:
```rs
tf.add_text("this is bold ", &normal);
tf.add_text("and this is not", &normal);
```
will produce:
`this is boldandthis is not`

## Tasks
- [x] Create a failing test which reproduces the bug
- [x] Fix the bug, test should pass

## Status
complete

---

# Issue 6: Research how other fonts can be used
## Description
We currently support only Helvetica font. We need to allow consumers to specifiy other fonts to be loaded and used. Research how this can be done. 
It looks like fonts/text are covered in section 9 of the PDF32000_2008.pdf specification. This ticket is research only. Update the Research section with the information gathered and create a basic sketch of how the api can support the use of other fonts.

## Tasks
- [x] Create an api sketch illustrates how other fonts can be loaded and used

## Research

### Font Types in PDF (Section 9 of PDF 32000-1:2008)

PDF supports several font types:

| Type | Description |
|------|-------------|
| **Type 1** | Adobe PostScript format. The 14 standard fonts are Type 1. |
| **TrueType** | Apple/Microsoft format (.ttf). Most common font format. |
| **Type 0 (Composite)** | Wrapper that references a CIDFont descendant. Required for full Unicode support. |
| **CIDFontType2** | CIDFont with TrueType outlines. Used when embedding .ttf fonts in a composite structure. |

**Simple vs Composite fonts**: Simple fonts (Type1, TrueType) are single-byte encoded (max 256 glyphs). Composite fonts (Type0 -> CIDFont) support multi-byte encoding for full Unicode. For a modern library, composite fonts are the right choice for embedded fonts.

### The 14 Standard Fonts (No Embedding Required)

These are guaranteed by every PDF viewer without embedding:

1. Helvetica, Helvetica-Bold, Helvetica-Oblique, Helvetica-BoldOblique
2. Times-Roman, Times-Bold, Times-Italic, Times-BoldItalic
3. Courier, Courier-Bold, Courier-Oblique, Courier-BoldOblique
4. Symbol
5. ZapfDingbats

We currently support Helvetica and Helvetica-Bold. All 14 can be added with minimal font dictionaries (no embedding, no FontDescriptor needed). We do need character width tables for each to support proper text measurement.

Note: PDF 2.0 deprecated the guarantee that standard fonts are available without embedding. For PDF/A compliance, even these must be embedded. For now, the non-embedded approach is acceptable.

### Embedding TrueType Fonts (Type0/CIDFont Composite Structure)

To embed a .ttf font with full Unicode support, 5 PDF objects are needed per font:

1. **Type0 Font dict** - Top-level font referenced by page Resources. Uses `/Encoding /Identity-H` (2-byte glyph IDs map directly to CIDs).
2. **CIDFontType2 dict** - Describes the TrueType CIDFont. Contains the `/W` widths array mapping CIDs to advance widths.
3. **FontDescriptor dict** - Font metadata: ascent, descent, bounding box, flags, CapHeight, StemV, and a reference to the font file stream.
4. **FontFile2 stream** - The embedded .ttf binary (optionally compressed with FlateDecode).
5. **ToUnicode CMap stream** - Maps glyph IDs back to Unicode codepoints. Critical for copy/paste and text search in viewers.

Relationship:
```
Page /Resources /Font /Fx -> Type0 Font
    /DescendantFonts -> [CIDFontType2]
        /FontDescriptor -> FontDescriptor
            /FontFile2 -> Font file stream
    /ToUnicode -> CMap stream
```

Text in content streams is written as hex-encoded 2-byte glyph IDs:
```
BT /F3 12 Tf 72 700 Td <00480065006C006C006F> Tj ET
```
Each character's glyph ID is looked up from the font's `cmap` table.

### Font Metrics (Widths)

TrueType fonts use an internal coordinate system defined by `unitsPerEm` (typically 1000 or 2048). PDF uses 1/1000 of a text unit. Conversion:
```
pdf_width = (ttf_advance_width / unitsPerEm) * 1000
```
Widths come from the font's `hmtx` table. Glyph IDs come from the `cmap` table.

### Font Subsetting

Full font files can be 1-20+ MB. Subsetting strips unused glyphs, reducing to 5-50 KB. Subsetted fonts use a 6-letter prefix: `/BaseFont /ABCDEF+MyFont`. This is important for keeping PDF file sizes small but can be added as a later optimization.

### Suggested Rust Crates

| Crate | Purpose |
|-------|---------|
| `ttf-parser` | Parse .ttf/.otf files. Zero-alloc, no unsafe, lightweight. Extract cmap, hmtx, head, OS/2 tables. |
| `subsetter` | Font subsetting (from the typst project). Produces valid subset .ttf from font + glyph IDs. |
| `flate2` | Deflate compression for font file streams. |

### Current Codebase Limitations

- `FontId` enum has only 2 hardcoded variants (Helvetica, HelveticaBold)
- Character width tables are hardcoded arrays for ASCII 32-126 only
- `place_text()` is hardcoded to F1/Helvetica at 12pt
- `TextStyle` selects font via a `bold: bool` field only
- Every page declares both F1 and F2 regardless of usage
- Font objects are created at document init with hardcoded object IDs (3, 4)
- `line_height()` uses a fixed 1.2x multiplier for all fonts

### Suggested Implementation Phases

**Phase 1**: Support all 14 standard fonts. No embedding. Add width tables for each. Refactor `FontId`/`TextStyle` to reference any loaded font.

**Phase 2**: TrueType embedding via Type0/CIDFont composite structure. Full .ttf embedded (no subsetting yet). Use `ttf-parser` for metrics extraction.

**Phase 3**: Font subsetting with `subsetter` crate + `flate2` compression.

## API Sketch

### Rust Core API

```rust
// --- Loading fonts ---

// Built-in standard fonts (no embedding needed)
let helvetica = doc.load_builtin_font(BuiltinFont::Helvetica)?;
let times_bold = doc.load_builtin_font(BuiltinFont::TimesBold)?;
let courier = doc.load_builtin_font(BuiltinFont::Courier)?;

// External TrueType fonts (embedded in the PDF)
let roboto = doc.load_font_file("fonts/Roboto-Regular.ttf")?;
let roboto_bold = doc.load_font_file("fonts/Roboto-Bold.ttf")?;

// --- Using fonts in TextStyle ---

// TextStyle references a FontHandle instead of just bold: bool
let normal = TextStyle::new(&helvetica, 12.0);
let heading = TextStyle::new(&roboto_bold, 18.0);
let body = TextStyle::new(&roboto, 11.0);

// --- TextFlow usage (unchanged) ---
let mut tf = TextFlow::new();
tf.add_text("Heading\n", &heading);
tf.add_text("Body text in Roboto. ", &body);
tf.add_text("Some Helvetica text.", &normal);

// --- place_text gains an optional style parameter ---
doc.place_text_styled("Hello", 72.0, 720.0, &normal);
```

### BuiltinFont Enum

```rust
pub enum BuiltinFont {
    Helvetica,
    HelveticaBold,
    HelveticaOblique,
    HelveticaBoldOblique,
    TimesRoman,
    TimesBold,
    TimesItalic,
    TimesBoldItalic,
    Courier,
    CourierBold,
    CourierOblique,
    CourierBoldOblique,
    Symbol,
    ZapfDingbats,
}
```

### FontHandle

```rust
// Opaque handle returned by load_builtin_font / load_font_file.
// Internally holds an index into the document's font registry.
#[derive(Debug, Clone, Copy)]
pub struct FontHandle {
    index: usize,
}
```

### PHP Extension API

```php
// Built-in fonts
$helvetica = $doc->loadBuiltinFont("Helvetica");
$timesBold = $doc->loadBuiltinFont("Times-Bold");

// External font
$roboto = $doc->loadFontFile("fonts/Roboto-Regular.ttf");

// TextStyle takes a FontHandle + size
$heading = new TextStyle($roboto, 18.0);
$body = new TextStyle($helvetica, 12.0);

// Usage unchanged
$tf = new TextFlow();
$tf->addText("Heading\n", $heading);
$tf->addText("Body text.", $body);
```

## Status
complete

---

# Issue 7: Implement Font Handling
## Description
Based on research findings in Issue 6, implmenent Font handling.

## Tasks
- [x] Task 1: Expand BuiltinFont enum and add width tables
- [x] Task 2: Refactor TextStyle and TextFlow
- [x] Task 3: Refactor PdfDocument font management
- [x] Task 4: Update public exports
- [x] Task 5: Update PHP extension
- [x] Task 6: Update example

## Status
complete

---

# Issue 8: Implement True Type Font Handling
## Description
Based on research findings in Issue 6, implement TrueType font handling. Allows users to load `.ttf` files and use them for text placement and text flows. Uses Type0/CIDFontType2 composite structure for full Unicode support. No font subsetting or compression in this phase.

## Tasks
- [x] Task 1: Add `ttf-parser` dependency to `pdf-core/Cargo.toml`
- [x] Task 2: Add `FontRef` enum and `TrueTypeFontId` newtype to `fonts.rs`
- [x] Task 3: Create `truetype.rs` module with `TrueTypeFont` struct
- [x] Task 4: Refactor `TextStyle.font` from `BuiltinFont` to `FontRef`
- [x] Task 5: Update `TextFlow` for dual-path encoding (builtin vs TrueType)
- [x] Task 6: Update `PdfDocument` for TrueType loading and deferred writing
- [x] Task 7: Update public exports and existing tests/examples
- [x] Task 8: Write TrueType tests (13 tests) with DejaVu Sans fixture
- [x] Task 9: Update PHP extension with `loadFontFile()` and `TextStyle::truetype()`
- [x] Task 10: Create example (`generate_truetype.rs`) and documentation

## Status
complete

---

# Issue 9: Implement line graphics
## Description
The library should support the ability to create basic line graphics. We will need to first design the api for this and then implement it.

Ideas for api design (psuedo code):
```ts
let doc = PdfDocument.create("outfile.pdf")
doc.begin_page(612.0, 792.0);

let x = 20;
let y = 20;
doc.move_to(x, y);
doc.lineto(x, y = y + 10);
doc.lineto(x = x + 10, y);
doc.lineto(x, y = y - 10);
doc.stroke(color: "rgb 0 .5 .5");
```

## Tasks
- [x] Task 1: Update ISSUES.md with task breakdown and status
- [x] Task 2: Create Color struct in `pdf-core/src/graphics.rs`
- [x] Task 3: Add 12 graphics methods to `PdfDocument`
- [x] Task 4: Write tests in `pdf-core/tests/graphics_test.rs`
- [x] Task 5: Update PHP extension in `pdf-php/src/lib.rs`
- [x] Task 6: Update PHP stubs in `pdf-php/pdf-php.stubs.php`
- [x] Task 7: Create example `pdf-core/examples/generate_graphics.rs`
- [x] Task 8: Create documentation `docs/features/line-graphics.md`

## Status
complete

---

# Issue 10: Allow compression
## Description
The content in the PDF files is currently written uncompressed. This can lead to very large files. Compression seems to be possible using Filters (section 7.4 of the PDF specification). We would get the most gain from compression using images but we have not implemented images yet. So, this issue deals with compressing the pdf objects that we have implemented thus far.

Questions:
1. Is the design as simple as `doc.enable_compression(true)`?
2. Should every type of pdf object be compressed?
3. Should multipe types of compression filters be supported? If so, what will the api look like? Maybe compression level 0 - 9 for example?

### Answers
1. Yes — `doc.set_compression(true)` is a single boolean toggle on `PdfDocument`.
2. Only stream objects can be compressed. All 3 stream types (page content, FontFile2, ToUnicode CMap) are compressed when enabled.
3. FlateDecode only. It's the universal standard. Compression level is left at flate2's default (level 6).

## Tasks
- [x] Task 1: Add `flate2` dependency to `pdf-core/Cargo.toml`
- [x] Task 2: Add `compress` field, `set_compression()` method, and `make_stream()` helper to `PdfDocument`
- [x] Task 3: Update all 3 stream creation sites to use `make_stream()`
- [x] Task 4: Write compression tests in `pdf-core/tests/document_test.rs`
- [x] Task 5: Update PHP extension with `setCompression()` method and stub
- [x] Task 6: Update ISSUES.md and create documentation

## Status
complete

---

# Issue 11: Images
## Description
Add support for images.

API Ideas:
```ts
let doc = PdfDocument.create("outfile.pdf")
doc.begin_page(612.0, 792.0);

let image = new Image("path/to/image");
// and/or
let image = new Image(bytes);

//  with optionalFitOptions having properties something like fit: clip | shrink | scale
let result = doc.place_image(image, rect, optionalFitOptions); 
match result {
    FitResult::Stop => break,
    FitResult::BoxFull => continue,
    FitResult::BoxEmpty => {
        eprintln!("Warning: bounding box too small",);
        break;
    }
}
```

## Tasks
- [x] Task 1: Add `png` dependency to `pdf-core/Cargo.toml`
- [x] Task 2: Create `pdf-core/src/images.rs` (types, JPEG parser, PNG parser, format detection, placement calculator)
- [x] Task 3: Add test fixture images (test.jpg, test.png, test_alpha.png)
- [x] Task 4: Integrate images into `PdfDocument` (load/place/write methods, resource tracking)
- [x] Task 5: Update `end_page()` for image resources (XObject dict in page Resources)
- [x] Task 6: Update public exports in `pdf-core/src/lib.rs`
- [x] Task 7: Write tests in `pdf-core/tests/images_test.rs` (19 tests)
- [x] Task 8: Update PHP extension (`pdf-php/src/lib.rs`) and stubs (`pdf-php/pdf-php.stubs.php`)
- [x] Task 9: Create example (`pdf-core/examples/generate_images.rs`)
- [x] Task 10: Create documentation (`docs/features/images.md`)
- [x] Task 11: Update ISSUES.md with tasks and status

## Status
complete

---
