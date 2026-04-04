---
layout: default
title: Line Graphics
---

# Line Graphics

## Purpose
Provides basic drawing capabilities for PDF documents: lines, rectangles, paths, fill, stroke, colors, and graphics state management. These are the building blocks for borders, boxes, dividers, charts, and any non-text visual element.

## How It Works

### Color Model
Colors use the `Color` struct with RGB components (each 0.0–1.0). Two constructors:
- `Color::rgb(r, g, b)` — explicit RGB
- `Color::gray(level)` — shorthand for equal r/g/b

Colors are set independently for stroke and fill operations, matching PDF's dual-color model.

### Drawing Model
PDF uses a path-based drawing model (like PostScript/SVG):
1. **Construct a path** — `move_to`, `line_to`, `rect`, `close_path`
2. **Paint the path** — `stroke`, `fill`, or `fill_stroke`

Paths are not visible until painted. Multiple path segments can be constructed before a single paint operation.

### Graphics State
`save_state()` / `restore_state()` push/pop the entire graphics state (colors, line width, etc.) on PDF's internal stack. Use these to isolate style changes so they don't affect subsequent drawing.

### PDF Operator Mapping
Each method appends the corresponding PDF content stream operator:

| Method | PDF Operator | Description |
|---|---|---|
| `set_stroke_color(Color)` | `r g b RG` | Set stroke color (RGB) |
| `set_fill_color(Color)` | `r g b rg` | Set fill color (RGB) |
| `set_line_width(f64)` | `w w` | Set line width |
| `move_to(x, y)` | `x y m` | Move current point |
| `line_to(x, y)` | `x y l` | Line from current point |
| `rect(x, y, w, h)` | `x y w h re` | Append rectangle |
| `curve_to(x1,y1,x2,y2,x3,y3)` | `x1 y1 x2 y2 x3 y3 c` | Cubic Bezier curve |
| `arc(cx, cy, r, start, end)` | (sequence of `c`) | Arc via Bezier approximation |
| `circle(cx, cy, r)` | (sequence of `c` + `h`) | Full closed circle |
| `close_path()` | `h` | Close subpath |
| `stroke()` | `S` | Stroke path |
| `fill()` | `f` | Fill path |
| `fill_stroke()` | `B` | Fill and stroke path |
| `save_state()` | `q` | Save graphics state |
| `restore_state()` | `Q` | Restore graphics state |

### Angle Type

The `Angle` type is used by `arc` to accept angles in either degrees or radians without requiring separate methods:

```rust
Angle::degrees(90.0)
Angle::radians(std::f64::consts::FRAC_PI_2)
```

Angles follow standard math convention: 0° points right, and positive angles rotate counter-clockwise.

### Arc Approximation

PDF has no native arc operator. Arcs are approximated using cubic Bezier segments — one per 90° of arc. The control point offset uses the constant `k = 4/3 * tan(α/4)`, which approximates the arc to within 0.027% for a 90° segment. Up to 4 segments are used for a full circle.

## Design Decisions

- **Why direct PDF operators instead of a shape abstraction?** The PDF spec already defines a clean path/paint model. Wrapping it adds complexity without value — users who need graphics typically understand coordinate-based drawing. Higher-level shapes (e.g., `draw_rectangle(x, y, w, h, stroke, fill)`) can be built on top as convenience methods later.

- **Why RGB only (no CMYK, grayscale operators)?** RGB covers the vast majority of screen/web use cases. PDF has separate operators for grayscale (`G`/`g`) and CMYK (`K`/`k`), but RGB via `RG`/`rg` is sufficient for the initial implementation. CMYK support can be added later without breaking changes.

- **Why no resource dictionary changes?** Graphics operations use only content stream operators — they don't reference named resources like fonts do. This keeps the implementation contained to `document.rs` methods that append bytes to `content_ops`.

- **Why method chaining?** All methods return `&mut Self`, matching the existing `place_text()`, `set_info()`, and `begin_page()` patterns. This allows natural drawing sequences: `doc.move_to(0,0).line_to(100,100).stroke()`.

## Usage Examples

```rust
use pivot_pdf::{Angle, Color, DocumentOptions, PdfDocument};

let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
doc.begin_page(612.0, 792.0);

// Stroked rectangle
doc.set_stroke_color(Color::rgb(0.0, 0.0, 0.0));
doc.set_line_width(1.0);
doc.rect(72.0, 72.0, 468.0, 648.0);
doc.stroke();

// Filled rectangle
doc.set_fill_color(Color::gray(0.9));
doc.rect(100.0, 600.0, 200.0, 50.0);
doc.fill();

// Triangle with fill+stroke
doc.save_state();
doc.set_fill_color(Color::rgb(1.0, 0.0, 0.0));
doc.set_stroke_color(Color::rgb(0.0, 0.0, 0.0));
doc.move_to(200.0, 300.0)
    .line_to(300.0, 300.0)
    .line_to(250.0, 400.0)
    .close_path()
    .fill_stroke();
doc.restore_state();

// Cubic Bezier curve (S-curve)
doc.move_to(80.0, 200.0)
    .curve_to(150.0, 250.0, 220.0, 150.0, 290.0, 200.0)
    .stroke();

// Quarter arc (pie slice)
doc.move_to(300.0, 400.0)
    .line_to(360.0, 400.0);
doc.arc(300.0, 400.0, 60.0, Angle::degrees(0.0), Angle::degrees(90.0));
doc.close_path().fill_stroke();

// Full circle (stroked)
doc.circle(450.0, 400.0, 40.0).stroke();

// Full circle (filled)
doc.set_fill_color(Color::rgb(0.8, 0.4, 0.8));
doc.circle(450.0, 300.0, 40.0).fill();

doc.end_document().unwrap();
```

## Limitations & Edge Cases
- RGB color space only (no CMYK or spot colors)
- No dash patterns (`d` operator) — solid lines only
- No line cap/join styles (`J`/`j` operators)
- No clipping paths
- No transparency/opacity (requires ExtGState resource)
- Coordinates respect the document's `Origin` setting (TopLeft or BottomLeft)
- No validation of path construction order (e.g., `stroke()` without prior path is valid PDF but draws nothing)
- Arc approximation error is < 0.027% per 90° segment; imperceptible at typical PDF resolutions

## Related
- PDF 32000-1:2008, Section 8.5 (Path Construction and Painting)
- PDF 32000-1:2008, Section 8.4 (Graphics State)
- `docs/pdf-reference-curated.md` — condensed PDF spec reference

## History of Changes

### Issue 9 (2026-02): Initial implementation
- Added `Color` struct with RGB and grayscale constructors
- Added 12 graphics methods to `PdfDocument`
- PHP extension bindings via `PhpColor` class and 12 method wrappers

### Issue 36 (2026-04): Bezier curves, arcs, and circles
- Added `Angle` type with `degrees`/`radians` constructors and `to_radians()` method
- Added `curve_to(x1, y1, x2, y2, x3, y3)` — maps to PDF `c` operator; all y-coords transformed per origin
- Added `arc_bezier_segments` pure helper — splits arc into ≤90° cubic Bezier segments using `k = 4/3 * tan(α/4)`
- Added `arc(cx, cy, radius, start: Angle, end: Angle)` — moves to start point, then emits Bezier segments
- Added `circle(cx, cy, radius)` — convenience on top of `arc` (full 360°, auto-closed with `h`)
- PHP bindings: `curveTo`, `arc` (degrees), `arcRad` (radians), `circle`
