---
layout: default
title: Coordinate Origin
---

# Coordinate Origin

## Purpose

PDF's native coordinate system has its origin at the **bottom-left** corner of each page,
with y increasing upward. This is mathematically natural but unintuitive for developers
coming from screen/web contexts, where the origin is at the **top-left** and y increases
downward.

This feature lets callers opt into a top-left origin at document creation time. When
`Origin::TopLeft` is selected, the library transparently converts all user-supplied
coordinates to PDF space — no PDF CTM tricks, just arithmetic at the API boundary.
Users never deal with raw PDF coordinates; they work entirely in their chosen system.

## How It Works

### Configuration

Pass a `DocumentOptions` value when creating a document:

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

### Coordinate Transforms

The origin setting applies **uniformly** to every coordinate-taking API.

#### Point transform (place_text, move_to, line_to)

```
y_pdf = page_height - y_user
```

#### Rect transform for graphics (rect operator, place_image, add_text_field)

`(x, y)` is the **top-left** corner; height extends downward.

```
y_pdf_bottom = page_height - y_user - height
```

#### Rect transform for layout engines (fit_textflow, fit_row / TableCursor)

These engines use a rect where `y` is the **top edge** in PDF space and `height`
extends downward. With `TopLeft`:

```
y_pdf_top = page_height - y_user
```

The `TableCursor`'s `current_y` is transparently converted on each `fit_row` call
and back-transformed after the row is placed, so the cursor remains in user space
throughout.

### Bottom-Left Unchanged

With `Origin::BottomLeft` (the default), all coordinates pass through the API
untouched. Existing code that passes `DocumentOptions::default()` behaves exactly
as before.

## Design Decisions

### Per-document, not per-page

A document has one mental model. Mixing origins within the same document would be
confusing and error-prone. A single `DocumentOptions` field covers the entire lifecycle
of a `PdfDocument`.

### Breaking change on constructors (pre-1.0)

`PdfDocument::new` and `PdfDocument::create` now take `DocumentOptions` as a required
second argument. Because the crate is pre-1.0 this is an acceptable breaking change.
Callers that want the old behaviour pass `DocumentOptions::default()`.

### Software transform, no PDF CTM

An alternative would be to emit a `cm` (current transformation matrix) operator into
the page content stream to flip the coordinate system. We chose not to do this because:

- CTM tricks interact poorly with existing content that was written in PDF native coords
  (e.g., image placement already does its own page-height arithmetic).
- Software transforms are explicit, debuggable, and have zero PDF overhead.
- A single `page_height - y` subtraction is cheaper than a matrix multiply.

### Rect semantics differ by API

Because some internal engines (textflow, tables) already used `y = top edge` in PDF
space, and graphics operators (`re`) expect `y = bottom edge`, the transform applied
to a `Rect` depends on which API is being called:

| API | rect.y meaning (user space) | Transform applied |
|-----|-----------------------------|-------------------|
| `rect(x, y, w, h)` | top-left corner | `y_pdf_bottom = page_height - y - h` |
| `place_image` | top-left corner | `y_pdf_bottom = page_height - y - h` |
| `add_text_field` | top-left corner | `y_pdf_bottom = page_height - y - h` |
| `fit_textflow` | top of text area | `y_pdf_top = page_height - y` |
| `fit_row` / `TableCursor` | top of table area | `y_pdf_top = page_height - y` |

In all cases, with `BottomLeft`, the rect is passed through unchanged.

## API Summary

| Rust | PHP | Notes |
|------|-----|-------|
| `Origin::BottomLeft` | `'bottom-left'` | Default |
| `Origin::TopLeft` | `'top-left'` | Screen/web style |
| `DocumentOptions { origin }` | `PdfDocumentOptions` | Passed to constructor |
| `PdfDocument::new(w, opts)` | `PdfDocument::createInMemory($opts)` | |
| `PdfDocument::create(p, opts)` | `PdfDocument::create($path, $opts)` | |

## Examples

### Rust

```rust
use pivot_pdf::{DocumentOptions, Origin, PdfDocument, Rect, TextFlow, TextStyle};

let opts = DocumentOptions { origin: Origin::TopLeft };
let mut doc = PdfDocument::create("output.pdf", opts)?;
doc.begin_page(612.0, 792.0);

// y=36 means 36pt from the top of the page.
doc.place_text("Hello, world!", 72.0, 36.0);

// Text area starts 100pt from the top, extends 600pt downward.
let rect = Rect { x: 72.0, y: 100.0, width: 468.0, height: 600.0 };
let mut flow = TextFlow::new();
flow.add_text("Body text...", &TextStyle::default());
doc.fit_textflow(&mut flow, &rect)?;

doc.end_document()?;
```

### PHP

```php
$opts = new PdfDocumentOptions();
$opts->origin = 'top-left';

$doc = PdfDocument::create("output.pdf", $opts);
$doc->beginPage(612.0, 792.0);

// y=36 means 36pt from the top of the page.
$doc->placeText("Hello, world!", 72.0, 36.0);

// Text area starts 100pt from the top.
$rect = new Rect(72.0, 100.0, 468.0, 600.0);
$flow = new TextFlow();
$flow->addText("Body text...", new TextStyle());
$doc->fitTextflow($flow, $rect);

$doc->endDocument();
```

See also the full working examples:
- `examples/rust/top_left_origin.rs`
- `examples/php/top_left_origin.php`

## Limitations

- **No per-page override.** The origin is fixed for the lifetime of the document.
- **TableCursor stores user-space coordinates.** `cursor->currentY()` returns a value
  in user space after `fit_row` calls. When using `TopLeft`, this is a distance from
  the top, not a PDF y-value.

## History

- **Issue 35**: Initial implementation — `Origin` enum, `DocumentOptions` struct,
  transform helpers, and consistent application to all coordinate-taking APIs.
