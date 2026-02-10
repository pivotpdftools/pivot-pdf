use std::collections::BTreeSet;

use crate::document::format_coord;
use crate::fonts::{BuiltinFont, FontMetrics};
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

/// Text styling options.
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font: BuiltinFont,
    pub font_size: f64,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            font: BuiltinFont::Helvetica,
            font_size: 12.0,
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
                        leading_space: had_space
                            && !words.is_empty(),
                    });
                    had_space = false;
                }
            }
        }
        words
    }

    /// Generate PDF content stream operations that fit within
    /// the given rectangle. Returns the content bytes, a
    /// FitResult, and the set of fonts actually used.
    pub fn generate_content_ops(
        &mut self,
        rect: &Rect,
    ) -> (Vec<u8>, FitResult, BTreeSet<BuiltinFont>) {
        let empty_fonts = BTreeSet::new();
        let words = self.extract_words();
        if self.cursor >= words.len() {
            return (
                Vec::new(),
                FitResult::Stop,
                empty_fonts,
            );
        }

        let mut output = Vec::new();
        let mut fonts_used = BTreeSet::new();
        let first_word = &words[self.cursor];
        let first_line_height = FontMetrics::line_height(
            first_word.style.font,
            first_word.style.font_size,
        );

        // Check if even one line fits vertically
        if first_line_height > rect.height {
            return (
                Vec::new(),
                FitResult::BoxEmpty,
                empty_fonts,
            );
        }

        output.extend_from_slice(b"BT\n");

        // Track current PDF y position (in page coordinates).
        // First baseline: top of rect minus ascent (approximated
        // as font_size since line_height = font_size * 1.2).
        let first_baseline_y =
            rect.y - first_word.style.font_size;
        let mut current_y = first_baseline_y;
        let mut is_first_line = true;
        let mut any_text_placed = false;

        // Track current font state in the content stream
        let mut active_font: Option<BuiltinFont> = None;
        let mut active_size: Option<f64> = None;

        while self.cursor < words.len() {
            // Determine line height from the first word on
            // this line
            let line_font = words[self.cursor].style.font;
            let line_font_size =
                words[self.cursor].style.font_size;
            let line_height = FontMetrics::line_height(
                line_font, line_font_size,
            );

            if !is_first_line {
                // Check if there's room for another line
                let next_y = current_y - line_height;
                let bottom = rect.y - rect.height;
                if next_y < bottom {
                    output.extend_from_slice(b"ET\n");
                    return (
                        output,
                        FitResult::BoxFull,
                        fonts_used,
                    );
                }
            }

            // Collect words that fit on this line
            let line_start_cursor = self.cursor;
            let mut line_width: f64 = 0.0;
            let mut line_end_cursor = self.cursor;

            while line_end_cursor < words.len() {
                let word = &words[line_end_cursor];

                // Newline forces a line break
                if word.text == "\n" {
                    line_end_cursor += 1;
                    break;
                }

                let font = word.style.font;
                let font_size = word.style.font_size;
                let word_width = FontMetrics::measure_text(
                    &word.text, font, font_size,
                );
                let space_width = if word.leading_space {
                    FontMetrics::measure_text(
                        " ", font, font_size,
                    )
                } else {
                    0.0
                };

                let total = line_width + space_width
                    + word_width;
                if total > rect.width
                    && line_end_cursor > line_start_cursor
                {
                    // Word doesn't fit, but we have some words
                    break;
                }
                if total > rect.width
                    && line_end_cursor == line_start_cursor
                {
                    // Single word too wide for the box
                    if !any_text_placed {
                        output.extend_from_slice(b"ET\n");
                        return (
                            Vec::new(),
                            FitResult::BoxEmpty,
                            BTreeSet::new(),
                        );
                    }
                    // Place it anyway if we've placed other
                    // text (it will overflow the line)
                    line_end_cursor += 1;
                    break;
                }

                line_width = total;
                line_end_cursor += 1;
            }

            // If no words collected (shouldn't happen), break
            if line_end_cursor == line_start_cursor {
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
                    format!(
                        "0 {} Td\n",
                        format_coord(-line_height),
                    )
                    .as_bytes(),
                );
                current_y -= line_height;
            }

            // Emit words for this line
            for i in line_start_cursor..line_end_cursor {
                let word = &words[i];
                if word.text == "\n" {
                    continue;
                }
                let font = word.style.font;
                let font_size = word.style.font_size;

                // Set font if changed
                if active_font != Some(font)
                    || active_size != Some(font_size)
                {
                    output.extend_from_slice(
                        format!(
                            "/{} {} Tf\n",
                            font.pdf_name(),
                            format_coord(font_size),
                        )
                        .as_bytes(),
                    );
                    active_font = Some(font);
                    active_size = Some(font_size);
                    fonts_used.insert(font);
                }

                let is_first_on_line =
                    i == line_start_cursor;
                let display_text = if word.leading_space
                    && !is_first_on_line
                {
                    format!(" {}", word.text)
                } else {
                    word.text.clone()
                };
                let escaped =
                    escape_pdf_string(&display_text);
                output.extend_from_slice(
                    format!("({}) Tj\n", escaped)
                        .as_bytes(),
                );
            }

            any_text_placed = true;
            self.cursor = line_end_cursor;
        }

        output.extend_from_slice(b"ET\n");

        if self.cursor >= words.len() {
            (output, FitResult::Stop, fonts_used)
        } else {
            (output, FitResult::BoxFull, fonts_used)
        }
    }
}
