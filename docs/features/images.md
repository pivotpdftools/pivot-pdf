---
layout: default
title: Image Support (JPEG + PNG)
---

# Image Support (JPEG + PNG)

## Purpose

Enables embedding raster images (photos, logos, diagrams) into PDF documents. Supports JPEG and PNG formats with transparency handling for PNG images with alpha channels.

## How It Works

### JPEG Handling

JPEG data is embedded as-is using PDF's `/Filter /DCTDecode`. The PDF spec natively understands JPEG streams, so:

- No pixel decoding or re-encoding is needed
- No quality loss from re-compression
- The raw JPEG bytes become the stream data
- Only the SOF (Start of Frame) marker is parsed to extract width, height, and component count

This makes JPEG embedding extremely efficient — the library only reads ~10 bytes from the header.

### PNG Handling

PNG images are decoded to raw pixels using the `png` crate, then embedded as uncompressed (or FlateDecode-compressed) pixel data:

- **RGB**: Pixels stored directly as DeviceRGB data
- **RGBA**: Split into two streams — RGB pixel data and a separate DeviceGray SMask (soft mask) for the alpha channel
- **Grayscale / GrayscaleAlpha**: Handled similarly with DeviceGray color space
- Palette/indexed PNGs are automatically expanded to RGB by the `png` crate

### Image Lifecycle

1. **Load** — `load_image_file()` or `load_image_bytes()` parses the image and stores it at the document level. Returns an `ImageId` handle.
2. **Place** — `place_image()` calculates placement geometry and appends content stream operators (`q`, `cm`, `Do`, `Q`) to the current page.
3. **Write** — During `end_page()`, image XObjects are written for any newly-used images. Images already written by previous pages are not re-written.

This means the same image placed on multiple pages produces only one XObject in the PDF file.

## Design Decisions

### No `image` crate dependency

JPEG needs only ~40 lines of header parsing (SOF marker scan). PNG decoding is handled by the lightweight `png` crate. The full `image` crate would add significant compile time and binary size for functionality we don't need.

### DCTDecode for JPEG (never double-compress)

JPEG streams use `/Filter /DCTDecode` exclusively, even when document-level compression is enabled. Compressing already-compressed JPEG data with FlateDecode would waste CPU and potentially increase file size.

### SMask for PNG transparency

PDF doesn't support inline alpha channels. Instead, RGBA images are split into:
- Main image: RGB data with `/ColorSpace /DeviceRGB`
- Soft mask: Grayscale alpha data as a separate Image XObject referenced via `/SMask`

This is the standard PDF approach for transparency (PDF 1.4+).

## Fit Modes

| Mode | Behavior |
|------|----------|
| `Fit` | Scale to fit within the rect, preserving aspect ratio. Centered. May leave empty space. |
| `Fill` | Scale to cover the entire rect, preserving aspect ratio. Centered. Clips overflow. |
| `Stretch` | Scale to fill the rect exactly. May distort the image. |
| `None` | Natural size: 1 pixel = 1 point. Positioned at top-left of rect. |

## Usage Examples

### Rust

```rust
let mut doc = PdfDocument::create("output.pdf").unwrap();
doc.set_compression(true);

let logo = doc.load_image_file("logo.png").unwrap();
let photo = doc.load_image_bytes(jpeg_bytes).unwrap();

doc.begin_page(612.0, 792.0);

let rect = Rect { x: 72.0, y: 72.0, width: 200.0, height: 150.0 };
doc.place_image(&logo, &rect, ImageFit::Fit);
doc.place_image(&photo, &rect, ImageFit::Fill);

doc.end_document().unwrap();
```

### PHP

```php
$doc = PdfDocument::create("output.pdf");
$doc->setCompression(true);

$logo = $doc->loadImageFile("logo.png");
$photo = $doc->loadImageBytes($jpegData);

$doc->beginPage(612.0, 792.0);

$rect = new Rect(72.0, 72.0, 200.0, 150.0);
$doc->placeImage($logo, $rect, "fit");
$doc->placeImage($photo, $rect, "fill");

$doc->endDocument();
```

## Limitations

- **No CMYK JPEG**: Only 1-component (grayscale) and 3-component (RGB) JPEGs are supported. 4-component CMYK JPEGs will return an error.
- **No EXIF rotation**: EXIF orientation tags are not read. Images may appear rotated if the source has EXIF rotation metadata.
- **No SVG**: Vector image support is deferred to a future issue.
- **No 16-bit PNG**: Only 8-bit-per-channel PNGs are supported.
- **No indexed PNG direct embedding**: Palette PNGs are expanded to RGB (no `/Indexed` color space optimization).

## History

- **Issue 11**: Initial implementation — JPEG DCTDecode, PNG with FlateDecode, RGBA transparency via SMask, four fit modes.
