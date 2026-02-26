---
layout: default
title: Edit Page
---

# Edit Page

## Purpose

The edit page feature allows adding content to pages after they have been written. The primary use case is **page numbering** — "Page X of Y" — where the total page count is unknown until all pages have been written.

## How It Works

### Two New Methods

| Method | Description |
|--------|-------------|
| `page_count() -> usize` | Returns the number of completed pages (pages for which `end_page()` has been called). |
| `open_page(page_num: usize) -> io::Result<()>` | Opens a completed page (1-indexed) for editing. Any drawing operations that follow are appended as an overlay. Call `end_page()` to close the edit. |

### Overlay Content Streams

When `open_page(n)` is called followed by content operations and `end_page()`, a new content stream is written and appended to page n's `/Contents` array:

```
Page dict (written at end_document):
  /Contents [original_stream overlay_stream]
```

Viewers render all streams in order, so overlay content appears on top of the original content.

### Deferred Page Dictionary Writing

The key mechanism that makes this possible:

| Phase | What's Written Immediately | What's Deferred |
|-------|--------------------------|-----------------|
| `end_page()` (new page) | Content stream | Page dictionary |
| `end_page()` (overlay) | Overlay content stream | — |
| `end_document()` | All page dictionaries | — |

Page dictionaries are small (just object references). Deferring them costs negligible memory while enabling `/Contents` arrays to be built up from multiple streams. Content streams are written immediately to keep memory usage low.

### Page Numbering Pattern

```rust
let mut doc = PdfDocument::create("report.pdf")?;

// --- Pass 1: write all content pages ---
let mut flow = TextFlow::new();
flow.add_text("Report content...", &body_style);

loop {
    doc.begin_page(612.0, 792.0);
    match doc.fit_textflow(&mut flow, &content_rect)? {
        FitResult::Stop => { doc.end_page()?; break; }
        FitResult::BoxFull => doc.end_page()?,
        FitResult::BoxEmpty => { doc.end_page()?; break; }
    }
}

// --- Pass 2: add "Page X of Y" footer overlay ---
let total = doc.page_count();
for i in 1..=total {
    doc.open_page(i)?;
    doc.place_text_styled(
        &format!("Page {} of {}", i, total),
        72.0, 28.0,
        &footer_style,
    );
    doc.end_page()?;
}

doc.end_document()?;
```

### Auto-Close Behaviour

`open_page()` automatically closes any currently open page before opening the edit, matching the behaviour of `begin_page()`. `end_document()` also auto-closes any open edit page.

### Font and Image Resources

Fonts and images used in overlay content are merged into the page's resource dictionary at `end_document()` time. A page edited with a different font than its original content will have both fonts in its `/Resources`.

## Design Decisions

### Why Not PDF Incremental Updates (Section 7.5.6)?

Incremental Updates are designed for appending to an already-**finalized** PDF file (one where `end_document()` has been called and the xref/trailer have been written). Since `open_page()` is called while the document is still being constructed, incremental updates would be unnecessarily complex — requiring `seek()` support on the writer, which breaks streaming to non-seekable targets.

### Why Deferred Page Dicts?

Previously, `end_page()` wrote content streams and page dicts together. The insight is that content streams must be freed immediately (low memory guarantee), but page dicts are tiny (~200 bytes) and can be accumulated without meaningful overhead. Deferring only the page dict write — until `end_document()` — lets us compose `/Contents` arrays from multiple streams without any file seeking.

### Object ID Pre-Allocation

Page dict object IDs are pre-allocated in `end_page()` (when the content stream is written) so that the xref table has a complete picture. The actual page dict bytes are written later in `write_page_dicts()`, but the ID is reserved immediately. This is why no ID gaps or renumbering occur.

## Limitations

- Overlay content is appended after original content. There is no way to insert content beneath the existing content (for that, consider writing a background in the original `begin_page` / `end_page` pass).
- `open_page()` requires an exact 1-indexed page number. Editing pages is sequential — you cannot call `open_page(5)` when only 3 pages have been completed.
- This feature requires `end_document()` not yet called. Editing an already-finalized PDF is not supported.

## Examples

See `pdf-core/examples/generate_page_numbers.rs` for a complete working example showing multi-page textflow with page numbering added in a second pass.

## History

- **Issue 13**: Initial implementation. Added `PageRecord`, deferred page dict writing, `page_count()`, and `open_page()`. Chose deferred page dict approach over PDF Incremental Updates to avoid complexity and maintain streaming compatibility.
