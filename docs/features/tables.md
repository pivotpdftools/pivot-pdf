---
layout: default
title: Tables
---

# Tables

## Purpose

Tables allow structured data — reports, invoices, bills of material — to be rendered as a grid of rows and columns within a PDF. Tables use the same fit-flow algorithm as `fit_textflow`, enabling large datasets to be streamed row-by-row from database cursors or iterators with minimal memory overhead.

## How It Works

The table API uses two types:

- **`Table`** — config only: column widths, border settings, and a default style template. No row storage, no internal cursor.
- **`TableCursor`** — tracks the current Y position on the current page. Created by the caller; reset when starting a new page.

The caller drives the loop, placing one row at a time via `fit_row` on `PdfDocument`:

| `FitResult` | Meaning |
|-------------|---------|
| `Stop` | Row was placed. Advance to the next row. |
| `BoxFull` | Page is full. End page, begin new page, reset cursor, retry the same row. |
| `BoxEmpty` | Rect is intrinsically too small (first_row is still true). Skip. |

### Multi-Page Streaming Pattern

```rust
use pdf_core::{
    BuiltinFont, Cell, CellStyle, Color, FitResult, FontRef,
    PdfDocument, Rect, Row, Table, TableCursor,
};

let table = Table::new(vec![120.0, 200.0, 100.0]);
let rect  = Rect { x: 72.0, y: 720.0, width: 468.0, height: 648.0 };

doc.begin_page(612.0, 792.0);
let mut cursor = TableCursor::new(&rect);

for row in database_results.iter() {
    loop {
        // Insert a header at the top of each page.
        if cursor.is_first_row() {
            doc.fit_row(&table, &header_row, &mut cursor)?;
        }

        match doc.fit_row(&table, row, &mut cursor)? {
            FitResult::Stop    => break,
            FitResult::BoxFull => {
                doc.end_page()?;
                doc.begin_page(612.0, 792.0);
                cursor.reset(&rect);
            }
            FitResult::BoxEmpty => break,
        }
    }
}
doc.end_page()?;
```

`cursor.is_first_row()` returns `true` after construction and after `reset()`, making it natural to insert a repeated header at the top of each page.

## Coordinate System

`Rect` uses the same convention as `fit_textflow`:
- `(x, y)` is the **top-left** corner in PDF absolute coordinates (from page bottom)
- `y` is measured from the **bottom of the page**
- Rows flow downward (decreasing y)

Example: for a US Letter page (612×792 pt) with 1-inch margins:
```rust
Rect { x: 72.0, y: 720.0, width: 468.0, height: 648.0 }
```

## TableCursor

```rust
pub struct TableCursor { ... }

impl TableCursor {
    pub fn new(rect: &Rect) -> Self       // current_y = rect.y, is_first_row = true
    pub fn reset(&mut self, rect: &Rect)  // call when starting a new page
    pub fn is_first_row(&self) -> bool    // true if no rows placed on this page yet
}
```

The cursor is owned by the caller. This means the caller can inspect `is_first_row()` before each `fit_row` call to decide whether to insert a header.

## Row Height

Row height is determined in two ways:

1. **Auto (Wrap mode)**: height = max across all cells of `count_lines × line_height + 2 × padding`
2. **Fixed**: set `row.height = Some(pts)` to override. Required for Clip and Shrink overflow.

## Overflow Modes

Each cell has an `overflow: CellOverflow` field:

| Mode | Behavior | Requires fixed `row.height`? |
|------|----------|------------------------------|
| `Wrap` (default) | Row grows to fit all wrapped text | No |
| `Clip` | Text is word-wrapped but clipped to the row's fixed height | Yes |
| `Shrink` | Font size reduced until text fits within the fixed height | Yes |

Shrink reduces font size by 0.5pt steps down to a minimum of 4pt.

## Borders

Borders are enabled by default (0.5pt black lines). Configure on `Table`:

```rust
table.border_color = Color::rgb(0.5, 0.5, 0.5);
table.border_width = 0.75;
// Disable:
table.border_width = 0.0;
```

Per row, borders draw:
- The outer rectangle (all four sides)
- Vertical dividers between columns

Horizontal dividers at row boundaries are produced by adjacent rows' top/bottom lines.

## Background Colors

Two levels of background fill:

1. **Row background** (`row.background_color`) — fills the entire row
2. **Cell background** (`cell.style.background_color`) — overrides the row background for that cell

```rust
let mut row = Row::new(cells);
row.background_color = Some(Color::gray(0.9));  // light gray row

let mut header_style = CellStyle::default();
header_style.background_color = Some(Color::rgb(0.2, 0.3, 0.5));  // dark blue cell
```

## Styling

`CellStyle` controls per-cell appearance:

| Field | Type | Default | Notes |
|-------|------|---------|-------|
| `font` | `FontRef` | Helvetica | Builtin or TrueType |
| `font_size` | `f64` | 10.0 pt | |
| `padding` | `f64` | 4.0 pt | All four sides |
| `overflow` | `CellOverflow` | `Wrap` | |
| `background_color` | `Option<Color>` | None | |
| `text_color` | `Option<Color>` | None (black) | |

The `Table.default_style` field is a reference style — it is not applied automatically. Clone it when constructing cells to reuse a consistent style:

```rust
let style = table.default_style.clone();
Cell::styled("text", style)
```

## Usage Example

```rust
use pdf_core::{
    BuiltinFont, Cell, CellStyle, Color, FitResult, FontRef,
    PdfDocument, Rect, Row, Table, TableCursor,
};

let table = Table::new(vec![120.0, 200.0, 100.0]);

let header_style = CellStyle {
    font: FontRef::Builtin(BuiltinFont::HelveticaBold),
    font_size: 10.0,
    background_color: Some(Color::rgb(0.2, 0.3, 0.5)),
    text_color: Some(Color::rgb(1.0, 1.0, 1.0)),
    ..CellStyle::default()
};

let header_row = Row::new(vec![
    Cell::styled("Name", header_style.clone()),
    Cell::styled("Description", header_style.clone()),
    Cell::styled("Amount", header_style),
]);

let rect = Rect { x: 72.0, y: 720.0, width: 468.0, height: 648.0 };
let mut rows = database_results.iter().peekable();

doc.begin_page(612.0, 792.0);
let mut cursor = TableCursor::new(&rect);

while rows.peek().is_some() {
    if cursor.is_first_row() {
        doc.fit_row(&table, &header_row, &mut cursor)?;
    }

    let row = rows.peek().unwrap();
    match doc.fit_row(&table, row, &mut cursor)? {
        FitResult::Stop    => { rows.next(); }
        FitResult::BoxFull => {
            doc.end_page()?;
            doc.begin_page(612.0, 792.0);
            cursor.reset(&rect);
        }
        FitResult::BoxEmpty => break,
    }
}
doc.end_page()?;
```

## Limitations

- **Left-aligned text only** — no center or right alignment per cell.
- **No column or row span** — each cell occupies exactly one column.
- **Padding is uniform** — all four sides share the same padding value.
- **No table-level min/max width** — column widths must be set explicitly.

## Design Decisions

### Why streaming fit_row instead of fit_table?

The original `fit_table` design required the entire dataset to be loaded into `Vec<Row>` before rendering began. For reports with thousands of rows from a database cursor, this wastes memory. The `fit_row` + `TableCursor` design lets the caller fetch one row at a time and pass it directly, enabling true streaming with O(1) memory per row.

The tradeoff is a slightly more complex calling pattern, but the caller gains full control: they can inspect `cursor.is_first_row()` to insert headers, peek at data to adjust row styles, or interleave table rows with other page content.

### Why does the caller own TableCursor?

Caller ownership of `TableCursor` enables `is_first_row()` to be checked before each `fit_row` call. If the cursor were internal to `Table`, the caller would have no way to inspect page state without additional API surface. The cursor is cheap (three fields) and its lifecycle exactly matches a single page rect.

### Why per-row border drawing?

Borders are drawn per row to naturally support multi-page flow. Each row draws its own outer rectangle and column dividers. The top line of each row overlaps with the bottom line of the previous row, which is visually correct and avoids state carried between rows.

### Why not auto-apply default_style to cells?

In Rust, non-optional struct fields always have a value, making it impossible to distinguish "user explicitly set this" from "this is the default". Rather than adding a parallel `Option<T>` field for every `CellStyle` attribute, the table's `default_style` acts as a template — users clone it when building cells. This keeps the API surface small and avoids hidden behavior.

### Why q/Q around each cell?

Each cell is wrapped in a PDF graphics state save/restore (`q`/`Q`). This isolates each cell's text color, clip path (Clip mode), and any other state changes, preventing style from leaking between adjacent cells. The overhead is minimal (2 bytes per cell).

## History

- **Issue 12** (2026-02): Initial implementation. Tables with Wrap/Clip/Shrink overflow, row/cell backgrounds, configurable borders, and multi-page flow using `fit_table`.
- **Issue 12 redesign** (2026-02): Replaced `fit_table` + internal row storage with `fit_row` + caller-owned `TableCursor`. Enables streaming from database cursors. `is_first_row()` added to support automatic header repetition across pages.
