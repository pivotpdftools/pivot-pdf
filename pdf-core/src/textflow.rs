use std::collections::BTreeSet;

use crate::document::format_coord;
use crate::fonts::{BuiltinFont, FontMetrics, FontRef};
use crate::truetype::TrueTypeFont;
use crate::writer::escape_pdf_string;

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
}

impl TextFlow {
    pub fn new() -> Self {
        TextFlow {
            spans: Vec::new(),
            cursor: 0,
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
    pub fn generate_content_ops(
        &mut self,
        rect: &Rect,
        tt_fonts: &mut [TrueTypeFont],
    ) -> (Vec<u8>, FitResult, UsedFonts) {
        let empty = UsedFonts::default();
        let words = self.extract_words();
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
