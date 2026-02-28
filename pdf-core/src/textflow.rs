use std::collections::BTreeSet;

use crate::document::format_coord;
use crate::fonts::{BuiltinFont, FontMetrics, FontRef};
use crate::truetype::TrueTypeFont;
use crate::writer::escape_pdf_string;

/// Controls how words wider than the available box width are handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WordBreak {
    /// Break wide words at a character boundary. (Default)
    #[default]
    BreakAll,
    /// Break wide words at a character boundary and insert a hyphen.
    Hyphenate,
    /// Do not break words. Wide words overflow the box.
    Normal,
}

/// Result of fitting text into a bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitResult {
    /// All text has been placed.
    Stop,
    /// The bounding box is full but text remains.
    BoxFull,
    /// The bounding box is too small to fit any text.
    BoxEmpty,
}

/// A bounding rectangle for text placement.
/// (x, y) is the upper-left corner. Text flows top-to-bottom.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Tracks which fonts were actually used during content generation.
#[derive(Debug, Default)]
pub struct UsedFonts {
    pub builtin: BTreeSet<BuiltinFont>,
    pub truetype: BTreeSet<usize>,
}

/// Text styling options.
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font: FontRef,
    pub font_size: f64,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            font: FontRef::Builtin(BuiltinFont::Helvetica),
            font_size: 12.0,
        }
    }
}

impl TextStyle {
    /// Convenience constructor for builtin fonts.
    pub fn builtin(font: BuiltinFont, font_size: f64) -> Self {
        TextStyle {
            font: FontRef::Builtin(font),
            font_size,
        }
    }
}

/// A span of text with associated style.
#[derive(Debug, Clone)]
struct TextSpan {
    text: String,
    style: TextStyle,
}

/// A word extracted from spans, carrying its style and whether
/// it is preceded by a space.
#[derive(Debug, Clone)]
struct Word {
    text: String,
    style: TextStyle,
    leading_space: bool,
}

/// A TextFlow manages styled text and flows it into bounding boxes
/// across one or more pages.
#[derive(Debug)]
pub struct TextFlow {
    spans: Vec<TextSpan>,
    /// Current position into the word list (for multi-page flow).
    cursor: usize,
    /// How to handle words wider than the bounding box.
    pub word_break: WordBreak,
}

impl TextFlow {
    pub fn new() -> Self {
        TextFlow {
            spans: Vec::new(),
            cursor: 0,
            word_break: WordBreak::BreakAll,
        }
    }

    /// Add styled text to the flow.
    pub fn add_text(&mut self, text: &str, style: &TextStyle) {
        self.spans.push(TextSpan {
            text: text.to_string(),
            style: style.clone(),
        });
    }

    /// Returns true if all text has been consumed.
    pub fn is_finished(&self) -> bool {
        let words = self.extract_words();
        self.cursor >= words.len()
    }

    /// Extract all words from spans, splitting on whitespace and
    /// preserving newlines as separate entries.
    fn extract_words(&self) -> Vec<Word> {
        let mut words = Vec::new();
        let mut had_space = false;
        for span in &self.spans {
            let mut chars = span.text.chars().peekable();

            while chars.peek().is_some() {
                // Consume leading spaces
                while chars.peek() == Some(&' ') {
                    had_space = true;
                    chars.next();
                }

                if chars.peek() == Some(&'\n') {
                    chars.next();
                    words.push(Word {
                        text: "\n".to_string(),
                        style: span.style.clone(),
                        leading_space: false,
                    });
                    had_space = false;
                    continue;
                }

                // Collect word characters
                let mut word = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == ' ' || ch == '\n' {
                        break;
                    }
                    word.push(ch);
                    chars.next();
                }

                if !word.is_empty() {
                    words.push(Word {
                        text: word,
                        style: span.style.clone(),
                        leading_space: had_space && !words.is_empty(),
                    });
                    had_space = false;
                }
            }
        }
        words
    }

    /// Generate PDF content stream operations that fit within
    /// the given rectangle. Returns the content bytes, a
    /// FitResult, and the fonts actually used.
    ///
    /// **Multi-page stability:** when `word_break` is not `Normal`, the word
    /// list is pre-processed by `break_wide_words` before layout. That
    /// function is deterministic for a given `rect.width`, so the internal
    /// cursor index remains valid across successive calls — provided the
    /// caller supplies the same `rect.width` every time for a given flow.
    pub fn generate_content_ops(
        &mut self,
        rect: &Rect,
        tt_fonts: &mut [TrueTypeFont],
    ) -> (Vec<u8>, FitResult, UsedFonts) {
        let empty = UsedFonts::default();
        let raw_words = self.extract_words();
        let words = if self.word_break != WordBreak::Normal {
            break_wide_words(raw_words, rect.width, self.word_break, tt_fonts)
        } else {
            raw_words
        };
        if self.cursor >= words.len() {
            return (Vec::new(), FitResult::Stop, empty);
        }

        let mut output = Vec::new();
        let mut used = UsedFonts::default();
        let first_word = &words[self.cursor];
        let first_line_height = line_height_for(&first_word.style, tt_fonts);

        // Check if even one line fits vertically
        if first_line_height > rect.height {
            return (Vec::new(), FitResult::BoxEmpty, empty);
        }

        output.extend_from_slice(b"BT\n");

        // First baseline: top of rect minus ascent (approximated
        // as font_size since line_height ~ font_size * 1.2).
        let first_baseline_y = rect.y - first_word.style.font_size;
        let mut current_y = first_baseline_y;
        let mut is_first_line = true;
        let mut any_text_placed = false;

        // Track current font state in the content stream
        let mut active_font: Option<FontRef> = None;
        let mut active_size: Option<f64> = None;

        while self.cursor < words.len() {
            let line_height = line_height_for(&words[self.cursor].style, tt_fonts);

            if !is_first_line {
                let next_y = current_y - line_height;
                let bottom = rect.y - rect.height;
                if next_y < bottom {
                    output.extend_from_slice(b"ET\n");
                    return (output, FitResult::BoxFull, used);
                }
            }

            // Collect words that fit on this line
            let line_start = self.cursor;
            let mut line_width: f64 = 0.0;
            let mut line_end = self.cursor;

            while line_end < words.len() {
                let word = &words[line_end];

                if word.text == "\n" {
                    line_end += 1;
                    break;
                }

                let word_width = measure_word(&word.text, &word.style, tt_fonts);
                let space_width = if word.leading_space {
                    measure_word(" ", &word.style, tt_fonts)
                } else {
                    0.0
                };

                let total = line_width + space_width + word_width;
                if total > rect.width && line_end > line_start {
                    break;
                }
                if total > rect.width && line_end == line_start {
                    if !any_text_placed {
                        output.extend_from_slice(b"ET\n");
                        return (Vec::new(), FitResult::BoxEmpty, UsedFonts::default());
                    }
                    line_end += 1;
                    break;
                }

                line_width = total;
                line_end += 1;
            }

            if line_end == line_start {
                break;
            }

            // Emit line positioning
            if is_first_line {
                output.extend_from_slice(
                    format!(
                        "{} {} Td\n",
                        format_coord(rect.x),
                        format_coord(first_baseline_y),
                    )
                    .as_bytes(),
                );
                is_first_line = false;
            } else {
                output.extend_from_slice(
                    format!("0 {} Td\n", format_coord(-line_height),).as_bytes(),
                );
                current_y -= line_height;
            }

            // Emit words for this line
            for i in line_start..line_end {
                let word = &words[i];
                if word.text == "\n" {
                    continue;
                }
                let font_ref = word.style.font;
                let font_size = word.style.font_size;

                // Set font if changed
                if active_font != Some(font_ref) || active_size != Some(font_size) {
                    let name = pdf_font_name(font_ref, tt_fonts);
                    output.extend_from_slice(
                        format!("/{} {} Tf\n", name, format_coord(font_size),).as_bytes(),
                    );
                    active_font = Some(font_ref);
                    active_size = Some(font_size);
                    record_font(&font_ref, &mut used);
                }

                let is_first_on_line = i == line_start;
                let display_text = if word.leading_space && !is_first_on_line {
                    format!(" {}", word.text)
                } else {
                    word.text.clone()
                };

                emit_text(&display_text, font_ref, tt_fonts, &mut output);
            }

            any_text_placed = true;
            self.cursor = line_end;
        }

        output.extend_from_slice(b"ET\n");

        let result = if self.cursor >= words.len() {
            FitResult::Stop
        } else {
            FitResult::BoxFull
        };
        (output, result, used)
    }
}

/// Split any word wider than `max_width` into character-boundary pieces.
///
/// Words that fit are left unchanged. Words that exceed `max_width` are split
/// via `break_word` and re-assembled as `Word` structs that carry the
/// original style and leading-space flag.
///
/// Because `extract_words` always produces the same vector for the same spans,
/// this function is also deterministic — the cursor index stays valid across
/// multiple `generate_content_ops` calls (i.e. across page breaks).
fn break_wide_words(
    words: Vec<Word>,
    max_width: f64,
    mode: WordBreak,
    tt_fonts: &[TrueTypeFont],
) -> Vec<Word> {
    let mut result: Vec<Word> = Vec::with_capacity(words.len());

    for word in words {
        if word.text == "\n" {
            result.push(word);
            continue;
        }

        let word_width = measure_word(&word.text, &word.style, tt_fonts);
        if word_width <= max_width {
            result.push(word);
            continue;
        }

        let ts = TextStyle {
            font: word.style.font,
            font_size: word.style.font_size,
        };
        let pieces = break_word(&word.text, max_width, &ts, mode, tt_fonts);
        let leading_space = word.leading_space;

        for (i, piece) in pieces.into_iter().enumerate() {
            result.push(Word {
                text: piece,
                style: word.style.clone(),
                leading_space: i == 0 && leading_space,
            });
        }
    }

    result
}

/// Break a single word into pieces that each fit within `avail_width`.
///
/// Returns at least one piece. In `Hyphenate` mode a `-` is appended to
/// every piece except the last. Forward progress is always guaranteed: a
/// single character is always emitted even if it exceeds the budget, so
/// the loop cannot run forever on a pathologically narrow box.
pub(crate) fn break_word(
    word: &str,
    avail_width: f64,
    style: &TextStyle,
    mode: WordBreak,
    tt_fonts: &[TrueTypeFont],
) -> Vec<String> {
    let hyphen_w = if mode == WordBreak::Hyphenate {
        measure_word("-", style, tt_fonts)
    } else {
        0.0
    };
    let mut pieces: Vec<String> = Vec::new();
    let mut remaining = word;

    while !remaining.is_empty() {
        let budget = avail_width - hyphen_w;
        let mut prefix_end = 0;
        let mut prefix_width = 0.0;

        for ch in remaining.chars() {
            let next_end = prefix_end + ch.len_utf8();
            let ch_w = measure_word(&remaining[..next_end], style, tt_fonts) - prefix_width;
            if prefix_width + ch_w > budget && prefix_end > 0 {
                break;
            }
            prefix_width += ch_w;
            prefix_end = next_end;
            // A single char already fills the budget — emit it and move on.
            if prefix_width >= budget {
                break;
            }
        }

        // Degenerate: budget so tiny even one char didn't fit — take one char.
        if prefix_end == 0 {
            prefix_end = remaining.chars().next().map_or(0, |c| c.len_utf8());
        }

        let is_last = prefix_end >= remaining.len();
        let piece = if !is_last && mode == WordBreak::Hyphenate {
            format!("{}-", &remaining[..prefix_end])
        } else {
            remaining[..prefix_end].to_string()
        };
        pieces.push(piece);
        remaining = &remaining[prefix_end..];
    }
    pieces
}

/// Compute line height based on font type.
pub(crate) fn line_height_for(style: &TextStyle, tt_fonts: &[TrueTypeFont]) -> f64 {
    match style.font {
        FontRef::Builtin(b) => FontMetrics::line_height(b, style.font_size),
        FontRef::TrueType(id) => tt_fonts[id.0].line_height(style.font_size),
    }
}

/// Measure a word's width based on font type.
pub(crate) fn measure_word(text: &str, style: &TextStyle, tt_fonts: &[TrueTypeFont]) -> f64 {
    match style.font {
        FontRef::Builtin(b) => FontMetrics::measure_text(text, b, style.font_size),
        FontRef::TrueType(id) => tt_fonts[id.0].measure_text(text, style.font_size),
    }
}

/// Get the PDF resource name for a font.
fn pdf_font_name(font: FontRef, tt_fonts: &[TrueTypeFont]) -> String {
    match font {
        FontRef::Builtin(b) => b.pdf_name().to_string(),
        FontRef::TrueType(id) => tt_fonts[id.0].pdf_name.clone(),
    }
}

/// Record a font as used.
fn record_font(font: &FontRef, used: &mut UsedFonts) {
    match font {
        FontRef::Builtin(b) => {
            used.builtin.insert(*b);
        }
        FontRef::TrueType(id) => {
            used.truetype.insert(id.0);
        }
    }
}

/// Emit text as either literal `(text) Tj` for builtin fonts
/// or hex `<glyph_ids> Tj` for TrueType fonts.
fn emit_text(text: &str, font: FontRef, tt_fonts: &mut [TrueTypeFont], output: &mut Vec<u8>) {
    match font {
        FontRef::Builtin(_) => {
            let escaped = escape_pdf_string(text);
            output.extend_from_slice(format!("({}) Tj\n", escaped).as_bytes());
        }
        FontRef::TrueType(id) => {
            let hex = tt_fonts[id.0].encode_text_hex(text);
            output.extend_from_slice(format!("{} Tj\n", hex).as_bytes());
        }
    }
}

#[cfg(test)]
mod break_word_tests {
    use super::*;
    use crate::fonts::BuiltinFont;

    /// Helvetica 12pt TextStyle — the font we use throughout the tests.
    fn hv12() -> TextStyle {
        TextStyle::builtin(BuiltinFont::Helvetica, 12.0)
    }

    /// Measure a string with Helvetica 12pt, no TrueType fonts.
    fn w(text: &str) -> f64 {
        measure_word(text, &hv12(), &[])
    }

    // -------------------------------------------------------
    // Basic correctness
    // -------------------------------------------------------

    #[test]
    fn empty_word_returns_empty_vec() {
        // The outer while-loop exits immediately for an empty string.
        let pieces = break_word("", 100.0, &hv12(), WordBreak::BreakAll, &[]);
        assert!(pieces.is_empty());
    }

    #[test]
    fn word_that_fits_returns_single_unchanged_piece() {
        let style = hv12();
        let avail = w("hello") + 1.0; // generous budget
        let pieces = break_word("hello", avail, &style, WordBreak::BreakAll, &[]);
        assert_eq!(pieces, vec!["hello"]);
    }

    #[test]
    fn word_exactly_at_boundary_is_not_broken() {
        // When the word fills the budget exactly the loop exits on the
        // `prefix_width >= budget` break, then `prefix_end == remaining.len()`,
        // so it's treated as the last piece — no split.
        let style = hv12();
        let avail = w("www"); // exactly 3 w's wide
        let pieces = break_word("www", avail, &style, WordBreak::BreakAll, &[]);
        assert_eq!(pieces, vec!["www"]);
    }

    // -------------------------------------------------------
    // BreakAll mode
    // -------------------------------------------------------

    #[test]
    fn break_all_splits_evenly_on_char_boundary() {
        // "wwwwww" at budget of exactly 3 w's → ["www", "www"].
        // Helvetica 'w' = 722/1000 em → at 12pt = 8.664 pt.
        let style = hv12();
        let avail = w("www"); // ~25.992 pt; "wwww" = ~34.656 pt won't fit
        let pieces = break_word("wwwwww", avail, &style, WordBreak::BreakAll, &[]);
        assert_eq!(pieces, vec!["www", "www"]);
    }

    #[test]
    fn break_all_produces_no_hyphens() {
        let style = hv12();
        let avail = w("ww"); // force a split
        let pieces = break_word("wwww", avail, &style, WordBreak::BreakAll, &[]);
        for piece in &pieces {
            assert!(
                !piece.ends_with('-'),
                "BreakAll should not add hyphens, got: {:?}",
                pieces
            );
        }
    }

    #[test]
    fn break_all_three_pieces() {
        // "iiiiiiiii" (9 i's) at width of 3 i's → ["iii", "iii", "iii"].
        // Helvetica 'i' = 222/1000 em → at 12pt = 2.664 pt.
        let style = hv12();
        let avail = w("iii");
        let pieces = break_word("iiiiiiiii", avail, &style, WordBreak::BreakAll, &[]);
        assert_eq!(pieces, vec!["iii", "iii", "iii"]);
    }

    // -------------------------------------------------------
    // Hyphenate mode
    // -------------------------------------------------------

    #[test]
    fn hyphenate_adds_hyphen_to_non_last_pieces() {
        // Budget = 3w - hyphen_width.  'w' = 8.664pt, '-' = 3.996pt.
        // Budget ≈ 25.992 - 3.996 = 21.996pt → "ww" (17.328) fits, "www" doesn't.
        // So each non-last piece holds 2 w's plus a hyphen.
        let style = hv12();
        let avail = w("www"); // ~25.992 pt
        let pieces = break_word("wwwwww", avail, &style, WordBreak::Hyphenate, &[]);
        // Every piece except the last must end with '-'.
        let (last, rest) = pieces.split_last().unwrap();
        for piece in rest {
            assert!(
                piece.ends_with('-'),
                "non-last piece should end with '-', got: {:?}",
                piece
            );
        }
        assert!(
            !last.ends_with('-'),
            "last piece must not end with '-', got: {:?}",
            last
        );
    }

    #[test]
    fn hyphenate_last_piece_never_ends_with_hyphen() {
        // The final piece of any split must not carry a hyphen.
        // Use a word that requires 3 pieces so the invariant is non-trivial.
        let style = hv12();
        let avail = w("www"); // ~25.992 pt → forces multi-piece split
        let pieces = break_word("wwwwwwww", avail, &style, WordBreak::Hyphenate, &[]);
        assert!(pieces.len() > 1, "expected a split");
        assert!(!pieces.last().unwrap().ends_with('-'));
    }

    #[test]
    fn hyphenate_word_fitting_budget_produces_one_piece_without_hyphen() {
        // When avail is large enough that the word fits even after reserving
        // room for a hyphen, break_word returns a single unhyphenated piece.
        // avail = word_width + hyphen_width + 1pt leaves the budget ≥ word_width.
        let style = hv12();
        let avail = w("hello") + w("-") + 1.0;
        let pieces = break_word("hello", avail, &style, WordBreak::Hyphenate, &[]);
        assert_eq!(pieces, vec!["hello"]);
    }

    #[test]
    fn hyphenate_pieces_respect_hyphen_width_budget() {
        // Each non-last piece (including its hyphen) must fit within avail.
        let style = hv12();
        let avail = w("www"); // ~25.992 pt
        let pieces = break_word("wwwwwwwwww", avail, &style, WordBreak::Hyphenate, &[]);
        for piece in &pieces {
            let piece_w = measure_word(piece, &style, &[]);
            assert!(
                piece_w <= avail + f64::EPSILON,
                "piece {:?} ({:.3}pt) exceeds avail ({:.3}pt)",
                piece,
                piece_w,
                avail
            );
        }
    }

    // -------------------------------------------------------
    // Forward-progress guarantee (degenerate narrow box)
    // -------------------------------------------------------

    #[test]
    fn single_char_wider_than_budget_still_emitted() {
        // When even one character is wider than the budget, the fallback
        // takes one character unconditionally so the loop always terminates.
        let style = hv12();
        let tiny = 1.0; // far smaller than any glyph
        let pieces = break_word("iii", tiny, &style, WordBreak::BreakAll, &[]);
        // One char per piece — forward progress guaranteed.
        assert_eq!(pieces, vec!["i", "i", "i"]);
    }

    #[test]
    fn single_char_word_with_tiny_budget_returns_that_char() {
        let style = hv12();
        let pieces = break_word("w", 1.0, &style, WordBreak::BreakAll, &[]);
        assert_eq!(pieces, vec!["w"]);
    }

    // -------------------------------------------------------
    // Unicode safety
    // -------------------------------------------------------

    #[test]
    fn multibyte_chars_split_on_codepoint_boundary() {
        // "é" is U+00E9, encoded as 2 bytes in UTF-8.
        // Ensure break_word never produces an invalid UTF-8 slice.
        // (The font will fall back to a default width for non-ASCII, which is fine.)
        let style = hv12();
        let pieces = break_word("éàü", 1.0, &style, WordBreak::BreakAll, &[]);
        // Each piece must be valid UTF-8 (Rust strings guarantee this).
        for piece in &pieces {
            assert!(!piece.is_empty());
        }
        // All characters must be accounted for.
        let rejoined: String = pieces.join("");
        assert_eq!(rejoined, "éàü");
    }
}
