---
layout: default
title: Word Break
---

# Word Break

## Purpose

Word-break controls what happens when a token (a word with no internal whitespace) is wider than the
available box width. Without it, long URLs, file paths, serial numbers, or any language that doesn't
use spaces overflow the box boundary, corrupting the layout.

Word-break applies consistently to both `fit_textflow` and `fit_row` (table cells).

## How It Works

Before the layout loop runs, any word wider than the box width is split into a sequence of shorter
pieces using a character-boundary scan. Each piece fits within the available width. The pieces are
then laid out as if they were separate words — each on its own line if necessary.

Because `extract_words` (in `TextFlow`) produces a deterministic word list for a given set of spans,
the pre-processed list is also deterministic. The multi-page cursor index therefore remains stable
across successive `fit_textflow` calls, so a long word that forces a page break resumes correctly
on the next page.

## Modes

| Mode | Rust variant | PHP value | Behaviour |
|------|-------------|-----------|-----------|
| Force-break (default) | `WordBreak::BreakAll` | `"break"` | Split at character boundary; no visual marker |
| Hyphenate | `WordBreak::Hyphenate` | `"hyphenate"` | Split with a `-` appended to each non-final piece |
| No break | `WordBreak::Normal` | `"normal"` | Original behaviour — wide words overflow |

`BreakAll` is the default for both `TextFlow` and `CellStyle` so that overflow is prevented by
default without any configuration.

## Configuration

### Rust — TextFlow

```rust
let mut tf = TextFlow::new();
// Default is BreakAll — no change needed for most cases.
tf.word_break = WordBreak::Hyphenate; // opt-in to hyphens

tf.add_text("https://very-long-url.example.com/path/to/resource", &style);
let result = doc.fit_textflow(&mut tf, &rect)?;
```

### Rust — CellStyle

```rust
let style = CellStyle {
    word_break: WordBreak::Hyphenate,
    ..CellStyle::default()
};
let cell = Cell::styled("ABCDEFGHIJKLMNOPQRSTUVWXYZ", style);
```

### PHP — TextFlow

```php
$tf = new TextFlow();
$tf->word_break = 'hyphenate'; // "break" (default), "hyphenate", or "normal"
$tf->addText('https://very-long-url.example.com/path/to/resource', $style);
$doc->fitTextflow($tf, $rect);
```

### PHP — CellStyle

```php
$style = new CellStyle();
$style->word_break = 'hyphenate';
$cell = Cell::styled('ABCDEFGHIJKLMNOPQRSTUVWXYZ', $style);
```

## Interaction with CellOverflow (tables only)

`word_break` and `overflow` are independent knobs that operate at different stages of the
rendering pipeline:

- **`overflow`** controls what happens when the **total content is too tall** for the row.
- **`word_break`** controls what happens when a **single word is too wide** for the column.

### The rendering pipeline

There are three stages where the two settings matter:

**1. Row height measurement** — only when `row.height` is `None` (Wrap mode). `count_lines` uses
`word_break` to decide how many lines a wide word produces. With `BreakAll`, wide words contribute
extra lines; with `Normal`, they count as one line regardless of overflow.

Clip and Shrink skip this step entirely — they require a fixed `row.height` set by the caller.

**2. Font size reduction** — only for `Shrink` mode. `word_break` changes what "fits" means:

- `BreakAll` / `Hyphenate` — only **height** must be satisfied. Wide words are always breakable
  to fit the column width, so width is not a constraint for font shrinking.
- `Normal` — both **height and width** must be satisfied. A word wider than the column can never
  wrap, so the font must shrink until every individual word fits in a single line.

**3. Render-time wrapping** — always runs for all three overflow modes. `word_break` controls
whether `wrap_paragraph` breaks wide words character-by-character or lets them overflow.

### Behaviour matrix

| | `BreakAll` (default) | `Hyphenate` | `Normal` |
|---|---|---|---|
| **Wrap** | Row grows to fit all lines; wide words broken across lines | Same as BreakAll, with hyphens | Row grows to fit wrapped lines; wide words overflow the column |
| **Clip** | Fixed height; wide words broken, then clipped at row boundary | Same as BreakAll, with hyphens | Fixed height; wide words overflow AND are clipped |
| **Shrink** | Font shrinks for height only; word breaking handles width | Same as BreakAll, with hyphens | Font shrinks until both total height and every word width fit |

### Gotcha: `Shrink + BreakAll`

With `Shrink + Normal`, the font shrinks to make every word fit within the column width.
With `Shrink + BreakAll` (the new default), the font only shrinks for height — wide words are
broken at the character boundary instead of being shrunk.

If you want `Shrink` to guarantee that no character-level breaks occur, set
`word_break: WordBreak::Normal` explicitly and let the font reduction handle oversized words.

## Design Decisions

**Default is BreakAll, not Normal.** Overflow is a silent layout bug that is hard to detect in
generated PDFs. Defaulting to force-break prevents the bug by default without requiring callers to
opt in. The old `Normal` behaviour is still available for cases where the caller controls the data
and guarantees token widths.

**Pre-processing, not inline splitting.** The word list is pre-processed once before the layout
loop rather than splitting words inline during layout. This keeps the layout loop simple and,
critically, keeps the cursor index stable across multi-page `fit_textflow` calls.

**`word_break` does not affect `TextFlow`.** The overflow/clip/shrink distinction only exists for
table cells, which have a fixed row height to work against. `TextFlow` always grows vertically to
fit broken lines; there is no height constraint to clip or shrink against.

## Limitations

- Hyphenation is purely mechanical (character-boundary); no dictionary-based hyphenation is applied.
- TrueType fonts measure character widths per-glyph, so break points are accurate. Builtin fonts use
  pre-computed width tables (ASCII 32–126); non-ASCII characters fall back to a default width.
- A single character wider than the available width (e.g. extremely small box) is always emitted on
  its own line to prevent an infinite loop.

## History

- **Issue 20** — Initial implementation. Added `WordBreak` enum, `word_break` field to `TextFlow`
  and `CellStyle`. Shared `break_word` helper lives in `textflow.rs` (`pub(crate)`) and is used
  by both the textflow and table rendering paths. Default changed from overflow to `BreakAll`.
