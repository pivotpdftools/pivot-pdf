use crate::document::format_coord;
use crate::fonts::{BuiltinFont, FontRef};
use crate::graphics::Color;
use crate::textflow::{
    break_word, line_height_for, measure_word, FitResult, Rect, TextStyle, UsedFonts, WordBreak,
};
use crate::truetype::TrueTypeFont;
use crate::writer::escape_pdf_string;

// -------------------------------------------------------
// Public types
// -------------------------------------------------------

/// How text that overflows the cell height is handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellOverflow {
    /// Text wraps across lines; row height grows to fit content (default).
    Wrap,
    /// Text is word-wrapped but clipped to the row's fixed height.
    Clip,
    /// Font size shrinks until all text fits within the row's fixed height.
    Shrink,
}

/// Style options for a table cell.
#[derive(Debug, Clone)]
pub struct CellStyle {
    /// Optional cell background color (overrides row background).
    pub background_color: Option<Color>,
    /// Optional text color. Defaults to PDF's current fill color (black).
    pub text_color: Option<Color>,
    /// Font reference.
    pub font: FontRef,
    /// Font size in points.
    pub font_size: f64,
    /// Padding applied to all four sides, in points.
    pub padding: f64,
    /// How to handle text that exceeds the available cell height.
    pub overflow: CellOverflow,
    /// How to handle words wider than the cell's available width.
    pub word_break: WordBreak,
}

impl Default for CellStyle {
    fn default() -> Self {
        CellStyle {
            background_color: None,
            text_color: None,
            font: FontRef::Builtin(BuiltinFont::Helvetica),
            font_size: 10.0,
            padding: 4.0,
            overflow: CellOverflow::Wrap,
            word_break: WordBreak::BreakAll,
        }
    }
}

/// A single table cell containing text and style.
#[derive(Clone)]
pub struct Cell {
    pub text: String,
    pub style: CellStyle,
}

impl Cell {
    /// Create a cell with the default style.
    pub fn new(text: impl Into<String>) -> Self {
        Cell {
            text: text.into(),
            style: CellStyle::default(),
        }
    }

    /// Create a cell with an explicit style.
    pub fn styled(text: impl Into<String>, style: CellStyle) -> Self {
        Cell {
            text: text.into(),
            style,
        }
    }
}

/// A row of cells in a table.
#[derive(Clone)]
pub struct Row {
    pub cells: Vec<Cell>,
    /// Optional background color applied to the entire row.
    /// Per-cell background_color takes priority.
    pub background_color: Option<Color>,
    /// Fixed row height in points. Required for `Clip` and `Shrink` overflow.
    /// When `None`, height is auto-calculated from cell content (`Wrap` mode).
    pub height: Option<f64>,
}

impl Row {
    /// Create a row with auto-calculated height and no background.
    pub fn new(cells: Vec<Cell>) -> Self {
        Row {
            cells,
            background_color: None,
            height: None,
        }
    }
}

/// Table layout configuration. Holds column widths and visual style; does not
/// store row data. The caller supplies one `Row` at a time to `fit_row`,
/// enabling streaming from a database cursor without buffering the full dataset.
pub struct Table {
    /// Column widths in points.
    pub columns: Vec<f64>,
    /// Reference style for constructing cells. Clone it when creating cells
    /// to apply consistent styling across the table.
    pub default_style: CellStyle,
    /// Border stroke color (default: black).
    pub border_color: Color,
    /// Border line width in points. Set to `0.0` to disable borders.
    pub border_width: f64,
}

impl Table {
    /// Create a new table layout with the given column widths.
    pub fn new(columns: Vec<f64>) -> Self {
        Table {
            columns,
            default_style: CellStyle::default(),
            border_color: Color::rgb(0.0, 0.0, 0.0),
            border_width: 0.5,
        }
    }

    /// Generate PDF content stream bytes for a single row.
    ///
    /// Returns the content bytes, a `FitResult`, and the fonts used.
    /// Updates `cursor` to reflect the row's placement.
    pub(crate) fn generate_row_ops(
        &self,
        row: &Row,
        cursor: &mut TableCursor,
        tt_fonts: &mut [TrueTypeFont],
    ) -> (Vec<u8>, FitResult, UsedFonts) {
        let row_height = measure_row_height(row, &self.columns, &self.default_style, tt_fonts);
        let bottom = cursor.rect.y - cursor.rect.height;

        if cursor.current_y - row_height < bottom {
            // Nothing placed yet on this page — rect is too small for this row.
            // Otherwise the page is simply full and the caller should turn it.
            let result = if cursor.first_row {
                FitResult::BoxEmpty
            } else {
                FitResult::BoxFull
            };
            return (Vec::new(), result, UsedFonts::default());
        }

        let mut output: Vec<u8> = Vec::new();
        let mut used = UsedFonts::default();

        draw_row_backgrounds(
            row,
            &self.columns,
            cursor.rect.x,
            cursor.current_y,
            row_height,
            &mut output,
        );

        let mut col_x = cursor.rect.x;
        for (col_idx, &col_width) in self.columns.iter().enumerate() {
            if let Some(cell) = row.cells.get(col_idx) {
                render_cell(
                    cell,
                    col_x,
                    cursor.current_y,
                    col_width,
                    row_height,
                    tt_fonts,
                    &mut output,
                    &mut used,
                );
            }
            col_x += col_width;
        }

        if self.border_width > 0.0 {
            draw_row_borders(
                &self.columns,
                cursor.rect.x,
                cursor.current_y,
                row_height,
                self.border_color,
                self.border_width,
                &mut output,
            );
        }

        cursor.current_y -= row_height;
        cursor.first_row = false;

        (output, FitResult::Stop, used)
    }
}

/// Tracks where the next row will be placed within a page.
///
/// Created once per table area, then passed to each `fit_row` call.
/// Call `reset()` when starting a new page to restore the cursor to
/// the top of the new rect. `is_first_row()` lets the caller detect
/// a fresh page — useful for repeating a header row.
///
/// # Example
/// ```no_run
/// # use pdf_core::{Table, TableCursor, Row, Cell, Rect, FitResult, PdfDocument};
/// # let table = Table::new(vec![200.0, 200.0]);
/// # let rect = Rect { x: 72.0, y: 720.0, width: 400.0, height: 648.0 };
/// # let header = Row::new(vec![Cell::new("Name"), Cell::new("Value")]);
/// # let data: Vec<Row> = vec![];
/// # let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
/// let mut cursor = TableCursor::new(&rect);
/// let mut rows = data.iter().peekable();
/// while rows.peek().is_some() {
///     doc.begin_page(612.0, 792.0);
///     // Repeat header on every page
///     doc.fit_row(&table, &header, &mut cursor).unwrap();
///     while let Some(row) = rows.peek() {
///         match doc.fit_row(&table, row, &mut cursor).unwrap() {
///             FitResult::Stop    => { rows.next(); }
///             FitResult::BoxFull => break,
///             FitResult::BoxEmpty => { rows.next(); break; }
///         }
///     }
///     doc.end_page().unwrap();
///     if rows.peek().is_some() { cursor.reset(&rect); }
/// }
/// ```
pub struct TableCursor {
    /// Bounding rectangle for the current page.
    pub(crate) rect: Rect,
    /// Top of the next row (PDF absolute coordinates, from page bottom).
    pub(crate) current_y: f64,
    /// True when no rows have been placed on the current page yet.
    pub(crate) first_row: bool,
}

impl TableCursor {
    /// Create a cursor positioned at the top of `rect`.
    pub fn new(rect: &Rect) -> Self {
        TableCursor {
            rect: *rect,
            current_y: rect.y,
            first_row: true,
        }
    }

    /// Reset to the top of a new rect. Call this when starting a new page.
    pub fn reset(&mut self, rect: &Rect) {
        self.rect = *rect;
        self.current_y = rect.y;
        self.first_row = true;
    }

    /// Returns `true` if no rows have been placed on the current page yet.
    ///
    /// Use this to detect the start of a new page so you can insert a
    /// repeated header row before placing data rows.
    pub fn is_first_row(&self) -> bool {
        self.first_row
    }

    /// Returns the Y coordinate where the next row would be placed.
    ///
    /// After placing all rows, this equals the bottom edge of the last row.
    /// Use it to position content that follows the table (e.g., totals section)
    /// without guessing where the table ended.
    pub fn current_y(&self) -> f64 {
        self.current_y
    }
}

// -------------------------------------------------------
// Measurement helpers
// -------------------------------------------------------

/// Compute the height needed for a row based on its content.
///
/// Returns `row.height` directly for fixed-height rows (Clip/Shrink modes).
/// Otherwise computes the maximum cell height across all columns.
fn measure_row_height(
    row: &Row,
    columns: &[f64],
    default_style: &CellStyle,
    tt_fonts: &[TrueTypeFont],
) -> f64 {
    if let Some(h) = row.height {
        return h;
    }
    columns
        .iter()
        .enumerate()
        .map(|(col_idx, &col_width)| {
            if let Some(cell) = row.cells.get(col_idx) {
                measure_cell_height(&cell.text, &cell.style, col_width, tt_fonts)
            } else {
                // Empty column: height of one line plus padding
                let ts = make_text_style(default_style);
                line_height_for(&ts, tt_fonts) + 2.0 * default_style.padding
            }
        })
        .fold(0.0_f64, f64::max)
}

/// Compute the height needed to display a cell's text content with wrapping.
fn measure_cell_height(
    text: &str,
    style: &CellStyle,
    col_width: f64,
    tt_fonts: &[TrueTypeFont],
) -> f64 {
    let avail_width = col_width - 2.0 * style.padding;
    let ts = make_text_style(style);
    let lh = line_height_for(&ts, tt_fonts);
    let lines = count_lines(text, avail_width, &ts, style.word_break, tt_fonts);
    lines as f64 * lh + 2.0 * style.padding
}

/// Convert a `CellStyle` to a `TextStyle` for use with measurement helpers.
fn make_text_style(style: &CellStyle) -> TextStyle {
    TextStyle {
        font: style.font,
        font_size: style.font_size,
    }
}

/// Count the total number of wrapped lines for `text` given the available width.
fn count_lines(
    text: &str,
    avail_width: f64,
    style: &TextStyle,
    word_break: WordBreak,
    tt_fonts: &[TrueTypeFont],
) -> usize {
    if text.is_empty() {
        return 1;
    }
    text.split('\n')
        .map(|para| count_paragraph_lines(para, avail_width, style, word_break, tt_fonts))
        .sum::<usize>()
        .max(1)
}

/// Count lines for a single paragraph (no newlines).
fn count_paragraph_lines(
    text: &str,
    avail_width: f64,
    style: &TextStyle,
    word_break: WordBreak,
    tt_fonts: &[TrueTypeFont],
) -> usize {
    let text = text.trim();
    if text.is_empty() {
        return 1;
    }
    let mut lines = 1usize;
    let mut line_width = 0.0_f64;

    for word in text.split_whitespace() {
        let word_w = measure_word(word, style, tt_fonts);
        let space_w = if line_width == 0.0 {
            0.0
        } else {
            measure_word(" ", style, tt_fonts)
        };
        let needed = line_width + space_w + word_w;

        if needed > avail_width && line_width > 0.0 {
            lines += 1;
            line_width = word_w;
            // If this word still overflows on its own line, count extra lines.
            if word_break != WordBreak::Normal && word_w > avail_width {
                lines += count_break_lines(word, avail_width, style, word_break, tt_fonts) - 1;
                line_width = trailing_piece_width(word, avail_width, style, word_break, tt_fonts);
            }
        } else if word_break != WordBreak::Normal && word_w > avail_width {
            // First word on a fresh line and it's still too wide.
            lines += count_break_lines(word, avail_width, style, word_break, tt_fonts) - 1;
            line_width = trailing_piece_width(word, avail_width, style, word_break, tt_fonts);
        } else {
            line_width = needed;
        }
    }
    lines
}

/// Count how many lines a single oversized word occupies when broken.
fn count_break_lines(
    word: &str,
    avail_width: f64,
    style: &TextStyle,
    word_break: WordBreak,
    tt_fonts: &[TrueTypeFont],
) -> usize {
    break_word(word, avail_width, style, word_break, tt_fonts).len()
}

/// Width of the last piece when a word is broken across lines.
fn trailing_piece_width(
    word: &str,
    avail_width: f64,
    style: &TextStyle,
    word_break: WordBreak,
    tt_fonts: &[TrueTypeFont],
) -> f64 {
    break_word(word, avail_width, style, word_break, tt_fonts)
        .last()
        .map_or(0.0, |p| measure_word(p, style, tt_fonts))
}

/// Word-wrap `text` into lines that fit within `avail_width`.
fn wrap_text(
    text: &str,
    avail_width: f64,
    style: &TextStyle,
    word_break: WordBreak,
    tt_fonts: &[TrueTypeFont],
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for para in text.split('\n') {
        wrap_paragraph(para.trim(), avail_width, style, word_break, tt_fonts, &mut lines);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Word-wrap a single paragraph into lines, appending to `out`.
fn wrap_paragraph(
    text: &str,
    avail_width: f64,
    style: &TextStyle,
    word_break: WordBreak,
    tt_fonts: &[TrueTypeFont],
    out: &mut Vec<String>,
) {
    if text.is_empty() {
        out.push(String::new());
        return;
    }
    let mut current_line = String::new();
    let mut line_width = 0.0_f64;

    for word in text.split_whitespace() {
        let word_w = measure_word(word, style, tt_fonts);
        let space_w = if current_line.is_empty() {
            0.0
        } else {
            measure_word(" ", style, tt_fonts)
        };
        let needed = line_width + space_w + word_w;

        if needed > avail_width && !current_line.is_empty() {
            out.push(current_line.clone());
            current_line = String::new();
            line_width = 0.0;
            // Fall through to place word on fresh line (may need breaking).
            place_word_on_line(word, avail_width, style, word_break, tt_fonts, &mut current_line, &mut line_width, out);
        } else if word_w > avail_width && word_break != WordBreak::Normal && current_line.is_empty() {
            // Fresh line, word is too wide — break it.
            place_word_on_line(word, avail_width, style, word_break, tt_fonts, &mut current_line, &mut line_width, out);
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
            line_width = needed;
        }
    }
    if !current_line.is_empty() {
        out.push(current_line);
    }
}

/// Append a single word to lines, breaking it if it is wider than `avail_width`.
///
/// All full pieces except the last are pushed to `out`. The last piece is
/// accumulated into `current_line`/`line_width` so subsequent words can
/// continue on the same line.
fn place_word_on_line(
    word: &str,
    avail_width: f64,
    style: &TextStyle,
    word_break: WordBreak,
    tt_fonts: &[TrueTypeFont],
    current_line: &mut String,
    line_width: &mut f64,
    out: &mut Vec<String>,
) {
    let word_w = measure_word(word, style, tt_fonts);

    if word_w <= avail_width || word_break == WordBreak::Normal {
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
        *line_width += word_w;
        return;
    }

    let pieces = break_word(word, avail_width, style, word_break, tt_fonts);
    let last_idx = pieces.len() - 1;
    for (i, piece) in pieces.into_iter().enumerate() {
        if i < last_idx {
            out.push(piece);
        } else {
            *current_line = piece.clone();
            *line_width = measure_word(&piece, style, tt_fonts);
        }
    }
}

// -------------------------------------------------------
// Rendering helpers
// -------------------------------------------------------

/// Get the PDF resource name for a font.
fn pdf_font_name(font: FontRef, tt_fonts: &[TrueTypeFont]) -> String {
    match font {
        FontRef::Builtin(b) => b.pdf_name().to_string(),
        FontRef::TrueType(id) => tt_fonts[id.0].pdf_name.clone(),
    }
}

/// Record a font as used in the current page.
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

/// Emit a text string using the correct encoding for the font type.
fn emit_cell_text(text: &str, font: FontRef, tt_fonts: &mut [TrueTypeFont], output: &mut Vec<u8>) {
    if text.is_empty() {
        return;
    }
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

/// Draw row and cell background fills.
///
/// Row background is drawn first; per-cell backgrounds overlay on top.
fn draw_row_backgrounds(
    row: &Row,
    columns: &[f64],
    row_x: f64,
    row_top: f64,
    row_height: f64,
    output: &mut Vec<u8>,
) {
    let row_bottom = row_top - row_height;

    if let Some(bg) = row.background_color {
        let total_width: f64 = columns.iter().sum();
        output.extend_from_slice(
            format!(
                "{} {} {} rg\n{} {} {} {} re\nf\n",
                format_coord(bg.r),
                format_coord(bg.g),
                format_coord(bg.b),
                format_coord(row_x),
                format_coord(row_bottom),
                format_coord(total_width),
                format_coord(row_height),
            )
            .as_bytes(),
        );
    }

    let mut col_x = row_x;
    for (col_idx, &col_width) in columns.iter().enumerate() {
        if let Some(cell) = row.cells.get(col_idx) {
            if let Some(bg) = cell.style.background_color {
                output.extend_from_slice(
                    format!(
                        "{} {} {} rg\n{} {} {} {} re\nf\n",
                        format_coord(bg.r),
                        format_coord(bg.g),
                        format_coord(bg.b),
                        format_coord(col_x),
                        format_coord(row_bottom),
                        format_coord(col_width),
                        format_coord(row_height),
                    )
                    .as_bytes(),
                );
            }
        }
        col_x += col_width;
    }
}

/// Draw row borders: outer rectangle plus vertical column dividers.
fn draw_row_borders(
    columns: &[f64],
    row_x: f64,
    row_top: f64,
    row_height: f64,
    border_color: Color,
    border_width: f64,
    output: &mut Vec<u8>,
) {
    let row_bottom = row_top - row_height;
    let total_width: f64 = columns.iter().sum();

    output.extend_from_slice(b"q\n");
    output.extend_from_slice(
        format!(
            "{} {} {} RG\n{} w\n",
            format_coord(border_color.r),
            format_coord(border_color.g),
            format_coord(border_color.b),
            format_coord(border_width),
        )
        .as_bytes(),
    );

    // Outer rectangle of the row
    output.extend_from_slice(
        format!(
            "{} {} {} {} re\nS\n",
            format_coord(row_x),
            format_coord(row_bottom),
            format_coord(total_width),
            format_coord(row_height),
        )
        .as_bytes(),
    );

    // Vertical column dividers (not drawn after the last column)
    let mut col_x = row_x;
    for &col_width in &columns[..columns.len().saturating_sub(1)] {
        col_x += col_width;
        output.extend_from_slice(
            format!(
                "{} {} m\n{} {} l\nS\n",
                format_coord(col_x),
                format_coord(row_top),
                format_coord(col_x),
                format_coord(row_bottom),
            )
            .as_bytes(),
        );
    }

    output.extend_from_slice(b"Q\n");
}

/// Render the text content of a single cell.
///
/// Wraps each cell in `q/Q` to isolate graphics state. Applies clip region
/// for `Clip` mode and reduces font size for `Shrink` mode.
fn render_cell(
    cell: &Cell,
    cell_x: f64,
    row_top: f64,
    col_width: f64,
    row_height: f64,
    tt_fonts: &mut [TrueTypeFont],
    output: &mut Vec<u8>,
    used: &mut UsedFonts,
) {
    let style = &cell.style;
    let avail_width = (col_width - 2.0 * style.padding).max(0.0);
    let avail_height = (row_height - 2.0 * style.padding).max(0.0);

    // Resolve effective font size (may be reduced for Shrink mode)
    let effective_font_size = if style.overflow == CellOverflow::Shrink {
        shrink_font_size(
            &cell.text,
            style.font,
            style.font_size,
            avail_width,
            avail_height,
            style.word_break,
            tt_fonts,
        )
    } else {
        style.font_size
    };

    let ts = TextStyle {
        font: style.font,
        font_size: effective_font_size,
    };
    let lh = line_height_for(&ts, tt_fonts);
    let lines = wrap_text(&cell.text, avail_width, &ts, style.word_break, tt_fonts);

    output.extend_from_slice(b"q\n");

    // Apply clipping rectangle for Clip mode
    if style.overflow == CellOverflow::Clip {
        let clip_bottom = row_top - row_height;
        output.extend_from_slice(
            format!(
                "{} {} {} {} re\nW\nn\n",
                format_coord(cell_x),
                format_coord(clip_bottom),
                format_coord(col_width),
                format_coord(row_height),
            )
            .as_bytes(),
        );
    }

    let text_x = cell_x + style.padding;
    // Baseline: top of cell minus top padding minus font size (approximates ascent)
    let first_line_y = row_top - style.padding - effective_font_size;

    output.extend_from_slice(b"BT\n");

    // Always set an explicit fill color for text. Without this, the fill
    // color from background drawing (set outside q/Q) would bleed into
    // text rendering, making text invisible on colored backgrounds.
    let text_color = style.text_color.unwrap_or_else(|| Color::rgb(0.0, 0.0, 0.0));
    output.extend_from_slice(
        format!(
            "{} {} {} rg\n",
            format_coord(text_color.r),
            format_coord(text_color.g),
            format_coord(text_color.b),
        )
        .as_bytes(),
    );

    let font_name = pdf_font_name(ts.font, tt_fonts);
    output.extend_from_slice(
        format!("/{} {} Tf\n", font_name, format_coord(effective_font_size)).as_bytes(),
    );
    record_font(&ts.font, used);

    output.extend_from_slice(
        format!(
            "{} {} Td\n",
            format_coord(text_x),
            format_coord(first_line_y),
        )
        .as_bytes(),
    );

    let mut is_first = true;
    for line in &lines {
        if !is_first {
            output
                .extend_from_slice(format!("0 {} Td\n", format_coord(-lh)).as_bytes());
        }
        emit_cell_text(line, ts.font, tt_fonts, output);
        is_first = false;
    }

    output.extend_from_slice(b"ET\n");
    output.extend_from_slice(b"Q\n");
}

/// Reduce font size by 0.5pt steps until the text fits within the available
/// dimensions, stopping at a minimum of 4pt.
///
/// When `word_break` is not `Normal`, every word can be broken, so only the
/// height constraint needs to be satisfied. When `Normal`, width must also
/// fit (a word wider than the column can never wrap — only shrinking helps).
fn shrink_font_size(
    text: &str,
    font: FontRef,
    initial_size: f64,
    avail_width: f64,
    avail_height: f64,
    word_break: WordBreak,
    tt_fonts: &[TrueTypeFont],
) -> f64 {
    const MIN_FONT_SIZE: f64 = 4.0;
    const STEP: f64 = 0.5;

    let mut font_size = initial_size;
    loop {
        let ts = TextStyle { font, font_size };
        let lh = line_height_for(&ts, tt_fonts);
        let lines = count_lines(text, avail_width, &ts, word_break, tt_fonts);
        let fits_height = lines as f64 * lh <= avail_height;
        let fits_width = word_break != WordBreak::Normal
            || text.split_whitespace().all(|w| measure_word(w, &ts, tt_fonts) <= avail_width);
        if (fits_height && fits_width) || font_size <= MIN_FONT_SIZE {
            break;
        }
        font_size = (font_size - STEP).max(MIN_FONT_SIZE);
    }
    font_size
}
