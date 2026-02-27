use std::fs::File;
use std::io::{BufWriter, Write};

use ext_php_rs::prelude::*;
use ext_php_rs::types::{Zval};

use pdf_core::{
    BuiltinFont, Cell, CellOverflow, CellStyle, Color, FitResult, FontRef, ImageFit, ImageId,
    PdfDocument, Rect, Row, Table, TableCursor, TextAlign, TextFlow, TextStyle, TrueTypeFontId,
    WordBreak,
};

// ----------------------------------------------------------
// Color
// ----------------------------------------------------------

/// PHP class: Color
///
/// ```php
/// $red = new Color(1.0, 0.0, 0.0);
/// $gray = Color::gray(0.5);
/// ```
#[php_class]
#[php(name = "Color")]
pub struct PhpColor {
    #[php(prop)]
    pub r: f64,
    #[php(prop)]
    pub g: f64,
    #[php(prop)]
    pub b: f64,
}

#[php_impl]
impl PhpColor {
    pub fn __construct(r: f64, g: f64, b: f64) -> Self {
        PhpColor { r, g, b }
    }

    pub fn gray(level: f64) -> Self {
        PhpColor {
            r: level,
            g: level,
            b: level,
        }
    }
}

impl PhpColor {
    fn to_core(&self) -> Color {
        Color::rgb(self.r, self.g, self.b)
    }
}

// ----------------------------------------------------------
// TextStyle
// ----------------------------------------------------------

/// PHP class: TextStyle
///
/// Builtin font (by name):
/// ```php
/// $style = new TextStyle("Helvetica-Bold", 14.0);
/// ```
///
/// TrueType font (by handle from loadFontFile):
/// ```php
/// $handle = $doc->loadFontFile("fonts/Roboto.ttf");
/// $style = TextStyle::truetype($handle, 12.0);
/// ```
#[php_class]
#[php(name = "TextStyle")]
pub struct PhpTextStyle {
    #[php(prop)]
    pub font_name: String,
    #[php(prop)]
    pub font_size: f64,
    /// -1 means builtin (use font_name), >= 0 means TrueType
    #[php(prop)]
    pub font_handle: i64,
}

#[php_impl]
impl PhpTextStyle {
    /// Create a TextStyle with a builtin font name.
    pub fn __construct(font: Option<String>, font_size: Option<f64>) -> Self {
        PhpTextStyle {
            font_name: font.unwrap_or_else(|| "Helvetica".to_string()),
            font_size: font_size.unwrap_or(12.0),
            font_handle: -1,
        }
    }

    /// Create a TextStyle for a TrueType font handle.
    pub fn truetype(handle: i64, font_size: Option<f64>) -> Self {
        PhpTextStyle {
            font_name: String::new(),
            font_size: font_size.unwrap_or(12.0),
            font_handle: handle,
        }
    }
}

impl PhpTextStyle {
    fn to_core(&self) -> Result<TextStyle, String> {
        let font_ref = if self.font_handle >= 0 {
            FontRef::TrueType(TrueTypeFontId(self.font_handle as usize))
        } else {
            let builtin = BuiltinFont::from_name(&self.font_name).ok_or_else(|| {
                format!(
                    "Unknown font: '{}'. Valid names: \
                     Helvetica, Helvetica-Bold, \
                     Helvetica-Oblique, \
                     Helvetica-BoldOblique, \
                     Times-Roman, Times-Bold, \
                     Times-Italic, Times-BoldItalic, \
                     Courier, Courier-Bold, \
                     Courier-Oblique, \
                     Courier-BoldOblique, \
                     Symbol, ZapfDingbats",
                    self.font_name,
                )
            })?;
            FontRef::Builtin(builtin)
        };

        Ok(TextStyle {
            font: font_ref,
            font_size: self.font_size,
        })
    }
}

// ----------------------------------------------------------
// Rect
// ----------------------------------------------------------

/// PHP class: Rect
///
/// ```php
/// $rect = new Rect(72.0, 720.0, 468.0, 648.0);
/// ```
#[php_class]
#[php(name = "Rect")]
pub struct PhpRect {
    #[php(prop)]
    pub x: f64,
    #[php(prop)]
    pub y: f64,
    #[php(prop)]
    pub width: f64,
    #[php(prop)]
    pub height: f64,
}

#[php_impl]
impl PhpRect {
    pub fn __construct(x: f64, y: f64, width: f64, height: f64) -> Self {
        PhpRect {
            x,
            y,
            width,
            height,
        }
    }
}

impl PhpRect {
    fn to_core(&self) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        }
    }
}

// ----------------------------------------------------------
// TextFlow
// ----------------------------------------------------------

/// PHP class: TextFlow
///
/// ```php
/// $tf = new TextFlow();
/// $tf->addText("Hello ", new TextStyle());
/// $tf->addText("Bold", new TextStyle("Helvetica-Bold"));
/// $tf->wordBreak = 'break';    // 'break' (default), 'hyphenate', or 'normal'
/// ```
#[php_class]
#[php(name = "TextFlow")]
pub struct PhpTextFlow {
    inner: TextFlow,
    /// Word break mode: "break" (default), "hyphenate", or "normal"
    #[php(prop)]
    pub word_break: String,
}

#[php_impl]
impl PhpTextFlow {
    pub fn __construct() -> Self {
        PhpTextFlow {
            inner: TextFlow::new(),
            word_break: "break".to_string(),
        }
    }

    pub fn add_text(&mut self, text: &str, style: &PhpTextStyle) -> Result<(), String> {
        let core_style = style.to_core()?;
        self.inner.add_text(text, &core_style);
        Ok(())
    }

    pub fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }
}

// ----------------------------------------------------------
// CellStyle
// ----------------------------------------------------------

/// PHP class: CellStyle
///
/// ```php
/// $header = new CellStyle();
/// $header->fontSize = 12.0;
/// $header->backgroundColor = new Color(0.2, 0.3, 0.5);
/// $header->textColor = new Color(1.0, 1.0, 1.0);
/// $header->overflow = 'wrap';      // 'wrap', 'clip', or 'shrink'
/// $header->wordBreak = 'break';    // 'break', 'hyphenate', or 'normal'
/// ```
#[php_class]
#[php(name = "CellStyle")]
pub struct PhpCellStyle {
    #[php(prop)]
    pub font_name: String,
    #[php(prop)]
    pub font_handle: i64,
    #[php(prop)]
    pub font_size: f64,
    #[php(prop)]
    pub padding: f64,
    /// Overflow mode: "wrap", "clip", or "shrink"
    #[php(prop)]
    pub overflow: String,
    /// Word break mode: "break" (default), "hyphenate", or "normal"
    #[php(prop)]
    pub word_break: String,
    /// Text alignment: "left" (default), "center", or "right"
    #[php(prop)]
    pub text_align: String,
    /// Background color (null = none)
    pub background_color: Option<Color>,
    /// Text color (null = default black)
    pub text_color: Option<Color>,
}

#[php_impl]
impl PhpCellStyle {
    pub fn __construct() -> Self {
        PhpCellStyle {
            font_name: "Helvetica".to_string(),
            font_handle: -1,
            font_size: 10.0,
            padding: 4.0,
            overflow: "wrap".to_string(),
            word_break: "break".to_string(),
            text_align: "left".to_string(),
            background_color: None,
            text_color: None,
        }
    }

    /// Set background color (pass null to clear).
    pub fn set_background_color(&mut self, color: Option<&PhpColor>) {
        self.background_color = color.map(|c| c.to_core());
    }

    /// Set text color (pass null to use default black).
    pub fn set_text_color(&mut self, color: Option<&PhpColor>) {
        self.text_color = color.map(|c| c.to_core());
    }

    /// Return a copy of this style as a new CellStyle instance.
    ///
    /// PHP's native `clone` operator does not work on extension objects because
    /// it bypasses the Rust constructor, leaving the internal struct
    /// uninitialized. Use this method instead:
    ///
    /// ```php
    /// $right = $base->clone();
    /// $right->textAlign = 'right';
    /// ```
    pub fn clone(&self) -> Self {
        PhpCellStyle {
            font_name: self.font_name.clone(),
            font_handle: self.font_handle,
            font_size: self.font_size,
            padding: self.padding,
            overflow: self.overflow.clone(),
            word_break: self.word_break.clone(),
            text_align: self.text_align.clone(),
            background_color: self.background_color,
            text_color: self.text_color,
        }
    }
}

impl PhpCellStyle {
    fn to_core(&self) -> Result<CellStyle, String> {
        let font = if self.font_handle >= 0 {
            FontRef::TrueType(TrueTypeFontId(self.font_handle as usize))
        } else {
            let builtin = BuiltinFont::from_name(&self.font_name).ok_or_else(|| {
                format!("Unknown font: '{}'", self.font_name)
            })?;
            FontRef::Builtin(builtin)
        };

        let overflow = match self.overflow.as_str() {
            "clip" => CellOverflow::Clip,
            "shrink" => CellOverflow::Shrink,
            _ => CellOverflow::Wrap,
        };

        let word_break = match self.word_break.as_str() {
            "hyphenate" => WordBreak::Hyphenate,
            "normal" => WordBreak::Normal,
            _ => WordBreak::BreakAll,
        };

        let text_align = match self.text_align.as_str() {
            "center" => TextAlign::Center,
            "right" => TextAlign::Right,
            _ => TextAlign::Left,
        };

        Ok(CellStyle {
            background_color: self.background_color,
            text_color: self.text_color,
            font,
            font_size: self.font_size,
            padding: self.padding,
            overflow,
            word_break,
            text_align,
        })
    }
}

// ----------------------------------------------------------
// Cell
// ----------------------------------------------------------

/// PHP class: Cell
///
/// ```php
/// $cell = new Cell("Hello");
/// $cell = Cell::styled("Bold", $style);
/// ```
#[php_class]
#[php(name = "Cell")]
pub struct PhpCell {
    text: String,
    style: Option<CellStyle>,
}

#[php_impl]
impl PhpCell {
    pub fn __construct(text: &str) -> Self {
        PhpCell {
            text: text.to_string(),
            style: None,
        }
    }

    /// Create a cell with an explicit style.
    pub fn styled(text: &str, style: &PhpCellStyle) -> Result<Self, String> {
        Ok(PhpCell {
            text: text.to_string(),
            style: Some(style.to_core()?),
        })
    }
}

impl PhpCell {
    fn to_core(self) -> Cell {
        match self.style {
            Some(s) => Cell::styled(self.text, s),
            None => Cell::new(self.text),
        }
    }
}

// ----------------------------------------------------------
// Row
// ----------------------------------------------------------

/// PHP class: Row
///
/// ```php
/// $row = new Row([$cell1, $cell2]);
/// $row->setBackgroundColor(new Color(0.9, 0.9, 0.9));
/// $row->height = 20.0; // optional fixed height
/// ```
#[php_class]
#[php(name = "Row")]
pub struct PhpRow {
    cells: Vec<Cell>,
    background_color: Option<Color>,
    #[php(prop)]
    pub height: Option<f64>,
}

#[php_impl]
impl PhpRow {
    pub fn __construct(cells: Vec<&PhpCell>) -> Self {
        let core_cells = cells.into_iter().map(|c| {
            let cell = PhpCell {
                text: c.text.clone(),
                style: c.style.clone(),
            };
            cell.to_core()
        }).collect();

        PhpRow {
            cells: core_cells,
            background_color: None,
            height: None,
        }
    }

    /// Set the row background color.
    pub fn set_background_color(&mut self, color: Option<&PhpColor>) {
        self.background_color = color.map(|c| c.to_core());
    }
}

impl PhpRow {
    fn to_core(&self) -> Row {
        let mut row = Row::new(self.cells.clone());
        row.background_color = self.background_color;
        row.height = self.height;
        row
    }
}

// ----------------------------------------------------------
// Table
// ----------------------------------------------------------

/// PHP class: Table
///
/// Config-only: stores column widths, border settings, and default style.
/// Pass rows individually to `PdfDocument::fitRow()`.
///
/// ```php
/// $table = new Table([200.0, 200.0, 68.0]);
/// $table->setBorderWidth(0.5);
/// $table->setBorderColor(new Color(0, 0, 0));
///
/// $cursor = new TableCursor(new Rect(72, 720, 468, 648));
/// foreach ($dbRows as $row) {
///     while (true) {
///         $r = $doc->fitRow($table, $row, $cursor);
///         if ($r === 'stop') break;
///         if ($r === 'box_full') {
///             $doc->endPage();
///             $doc->beginPage(612, 792);
///             $cursor->reset(new Rect(72, 720, 468, 648));
///         } else { break; } // box_empty
///     }
/// }
/// ```
#[php_class]
#[php(name = "Table")]
pub struct PhpTable {
    inner: Table,
}

#[php_impl]
impl PhpTable {
    pub fn __construct(columns: Vec<f64>) -> Self {
        PhpTable {
            inner: Table::new(columns),
        }
    }

    pub fn set_border_color(&mut self, color: &PhpColor) {
        self.inner.border_color = color.to_core();
    }

    pub fn set_border_width(&mut self, width: f64) {
        self.inner.border_width = width;
    }

    pub fn set_default_style(&mut self, style: &PhpCellStyle) -> Result<(), String> {
        self.inner.default_style = style.to_core()?;
        Ok(())
    }
}

// ----------------------------------------------------------
// TableCursor
// ----------------------------------------------------------

/// PHP class: TableCursor
///
/// Tracks the current Y position while placing rows on a page.
/// Create one cursor per page rect; call `reset()` when starting a new page.
/// Use `isFirstRow()` to detect the top of a page and insert a header row.
///
/// ```php
/// $cursor = new TableCursor(new Rect(72, 720, 468, 648));
/// if ($cursor->isFirstRow()) {
///     $doc->fitRow($table, $headerRow, $cursor);
/// }
/// ```
#[php_class]
#[php(name = "TableCursor")]
pub struct PhpTableCursor {
    inner: TableCursor,
}

#[php_impl]
impl PhpTableCursor {
    pub fn __construct(rect: &PhpRect) -> Self {
        PhpTableCursor {
            inner: TableCursor::new(&rect.to_core()),
        }
    }

    pub fn reset(&mut self, rect: &PhpRect) {
        self.inner.reset(&rect.to_core());
    }

    pub fn is_first_row(&self) -> bool {
        self.inner.is_first_row()
    }

    pub fn current_y(&self) -> f64 {
        self.inner.current_y()
    }
}

// ----------------------------------------------------------
// PdfDocument
// ----------------------------------------------------------

/// Concrete inner types since PdfDocument<W> is generic.
enum DocumentInner {
    File(PdfDocument<BufWriter<File>>),
    Memory(PdfDocument<Vec<u8>>),
}

/// Dispatch a method call to the correct variant.
macro_rules! with_doc {
    ($self:expr, $name:ident, $doc:ident => $body:expr) => {
        match $self.inner.as_mut() {
            Some(inner) => match inner {
                DocumentInner::File($doc) => $body,
                DocumentInner::Memory($doc) => $body,
            },
            None => {
                return Err(format!("{}: document already ended", stringify!($name)));
            }
        }
    };
}

/// PHP class: PdfDocument
///
/// ```php
/// $doc = PdfDocument::create("out.pdf");
/// $doc = PdfDocument::createInMemory();
///
/// // Load TrueType font
/// $handle = $doc->loadFontFile("fonts/Roboto.ttf");
/// $style = TextStyle::truetype($handle, 14.0);
/// ```
#[php_class]
#[php(name = "PdfDocument")]
pub struct PhpPdfDocument {
    inner: Option<DocumentInner>,
}

#[php_impl]
impl PhpPdfDocument {
    pub fn create(path: &str) -> Result<Self, String> {
        let doc = PdfDocument::create(path).map_err(|e| format!("create failed: {}", e))?;
        Ok(PhpPdfDocument {
            inner: Some(DocumentInner::File(doc)),
        })
    }

    pub fn create_in_memory() -> Result<Self, String> {
        let doc =
            PdfDocument::new(Vec::new()).map_err(|e| format!("create_in_memory failed: {}", e,))?;
        Ok(PhpPdfDocument {
            inner: Some(DocumentInner::Memory(doc)),
        })
    }

    /// Load a TrueType font file. Returns an integer handle
    /// for use with TextStyle::truetype().
    pub fn load_font_file(&mut self, path: &str) -> Result<i64, String> {
        with_doc!(self, load_font_file, doc => {
            let font_ref = doc.load_font_file(path)
                .map_err(|e| {
                    format!(
                        "load_font_file failed: {}",
                        e,
                    )
                })?;
            match font_ref {
                FontRef::TrueType(id) => {
                    Ok(id.0 as i64)
                }
                _ => Err(
                    "Unexpected font type".to_string()
                ),
            }
        })
    }

    pub fn set_info(&mut self, key: &str, value: &str) -> Result<(), String> {
        with_doc!(self, set_info, doc => {
            doc.set_info(key, value);
            Ok(())
        })
    }

    pub fn set_compression(&mut self, enabled: bool) -> Result<(), String> {
        with_doc!(self, set_compression, doc => {
            doc.set_compression(enabled);
            Ok(())
        })
    }

    pub fn begin_page(&mut self, width: f64, height: f64) -> Result<(), String> {
        with_doc!(self, begin_page, doc => {
            doc.begin_page(width, height);
            Ok(())
        })
    }

    pub fn place_text(&mut self, text: &str, x: f64, y: f64) -> Result<(), String> {
        with_doc!(self, place_text, doc => {
            doc.place_text(text, x, y);
            Ok(())
        })
    }

    pub fn place_text_styled(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        style: &PhpTextStyle,
    ) -> Result<(), String> {
        let core_style = style.to_core()?;
        with_doc!(self, place_text_styled, doc => {
            doc.place_text_styled(text, x, y, &core_style);
            Ok(())
        })
    }

    pub fn fit_textflow(
        &mut self,
        flow: &mut PhpTextFlow,
        rect: &PhpRect,
    ) -> Result<String, String> {
        let core_rect = rect.to_core();
        flow.inner.word_break = match flow.word_break.as_str() {
            "hyphenate" => WordBreak::Hyphenate,
            "normal" => WordBreak::Normal,
            _ => WordBreak::BreakAll,
        };
        with_doc!(self, fit_textflow, doc => {
            let result = doc
                .fit_textflow(
                    &mut flow.inner,
                    &core_rect,
                )
                .map_err(|e| {
                    format!(
                        "fit_textflow failed: {}",
                        e,
                    )
                })?;
            Ok(match result {
                FitResult::Stop => {
                    "stop".to_string()
                }
                FitResult::BoxFull => {
                    "box_full".to_string()
                }
                FitResult::BoxEmpty => {
                    "box_empty".to_string()
                }
            })
        })
    }

    /// Place a single row into the table layout on the current page.
    ///
    /// Returns "stop" (placed), "box_full" (page full, turn page and retry),
    /// or "box_empty" (rect too small for this row).
    pub fn fit_row(
        &mut self,
        table: &PhpTable,
        row: &PhpRow,
        cursor: &mut PhpTableCursor,
    ) -> Result<String, String> {
        let core_row = row.to_core();
        with_doc!(self, fit_row, doc => {
            let result = doc
                .fit_row(&table.inner, &core_row, &mut cursor.inner)
                .map_err(|e| format!("fit_row failed: {}", e))?;
            Ok(match result {
                FitResult::Stop => "stop".to_string(),
                FitResult::BoxFull => "box_full".to_string(),
                FitResult::BoxEmpty => "box_empty".to_string(),
            })
        })
    }

    // -------------------------------------------------------
    // Image operations
    // -------------------------------------------------------

    /// Load an image from a file path. Returns an integer handle.
    pub fn load_image_file(&mut self, path: &str) -> Result<i64, String> {
        with_doc!(self, load_image_file, doc => {
            let id = doc.load_image_file(path)
                .map_err(|e| format!("load_image_file failed: {}", e))?;
            Ok(id.0 as i64)
        })
    }

    /// Load an image from raw bytes. Returns an integer handle.
    pub fn load_image_bytes(&mut self, data: &mut Zval) -> Result<i64, String> {
        let bytes = data
            .binary()
            .ok_or_else(|| "Expected binary string".to_string())?
            .to_vec();

        with_doc!(self, load_image_bytes, doc => {
            let id = doc.load_image_bytes(bytes)
                .map_err(|e| format!("load_image_bytes failed: {}", e))?;
            Ok(id.0 as i64)
        })
    }

    /// Place an image on the current page.
    /// fit: "fit" (default), "fill", "stretch", "none"
    pub fn place_image(
        &mut self,
        handle: i64,
        rect: &PhpRect,
        fit: Option<String>,
    ) -> Result<(), String> {
        let image_fit = parse_image_fit(&fit.unwrap_or_else(|| "fit".to_string()))?;
        let core_rect = rect.to_core();
        let image_id = ImageId(handle as usize);
        with_doc!(self, place_image, doc => {
            doc.place_image(&image_id, &core_rect, image_fit);
            Ok(())
        })
    }

    // -------------------------------------------------------
    // Graphics operations
    // -------------------------------------------------------

    pub fn set_stroke_color(&mut self, color: &PhpColor) -> Result<(), String> {
        with_doc!(self, set_stroke_color, doc => {
            doc.set_stroke_color(color.to_core());
            Ok(())
        })
    }

    pub fn set_fill_color(&mut self, color: &PhpColor) -> Result<(), String> {
        with_doc!(self, set_fill_color, doc => {
            doc.set_fill_color(color.to_core());
            Ok(())
        })
    }

    pub fn set_line_width(&mut self, width: f64) -> Result<(), String> {
        with_doc!(self, set_line_width, doc => {
            doc.set_line_width(width);
            Ok(())
        })
    }

    pub fn move_to(&mut self, x: f64, y: f64) -> Result<(), String> {
        with_doc!(self, move_to, doc => {
            doc.move_to(x, y);
            Ok(())
        })
    }

    pub fn line_to(&mut self, x: f64, y: f64) -> Result<(), String> {
        with_doc!(self, line_to, doc => {
            doc.line_to(x, y);
            Ok(())
        })
    }

    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) -> Result<(), String> {
        with_doc!(self, rect, doc => {
            doc.rect(x, y, width, height);
            Ok(())
        })
    }

    pub fn close_path(&mut self) -> Result<(), String> {
        with_doc!(self, close_path, doc => {
            doc.close_path();
            Ok(())
        })
    }

    pub fn stroke(&mut self) -> Result<(), String> {
        with_doc!(self, stroke, doc => {
            doc.stroke();
            Ok(())
        })
    }

    pub fn fill(&mut self) -> Result<(), String> {
        with_doc!(self, fill, doc => {
            doc.fill();
            Ok(())
        })
    }

    pub fn fill_stroke(&mut self) -> Result<(), String> {
        with_doc!(self, fill_stroke, doc => {
            doc.fill_stroke();
            Ok(())
        })
    }

    pub fn save_state(&mut self) -> Result<(), String> {
        with_doc!(self, save_state, doc => {
            doc.save_state();
            Ok(())
        })
    }

    pub fn restore_state(&mut self) -> Result<(), String> {
        with_doc!(self, restore_state, doc => {
            doc.restore_state();
            Ok(())
        })
    }

    /// Returns the number of completed pages.
    pub fn page_count(&self) -> Result<i64, String> {
        match self.inner.as_ref() {
            Some(inner) => match inner {
                DocumentInner::File(doc) => Ok(doc.page_count() as i64),
                DocumentInner::Memory(doc) => Ok(doc.page_count() as i64),
            },
            None => Err("page_count: document already ended".to_string()),
        }
    }

    /// Open a completed page for editing (1-indexed).
    ///
    /// Used for adding overlay content such as page numbers after all
    /// pages have been written. If a page is currently open, it is
    /// automatically closed first.
    pub fn open_page(&mut self, page_num: i64) -> Result<(), String> {
        if page_num < 1 {
            return Err(format!("open_page: page_num must be >= 1, got {}", page_num));
        }
        with_doc!(self, open_page, doc => {
            doc.open_page(page_num as usize)
                .map_err(|e| format!("open_page failed: {}", e))
        })
    }

    pub fn end_page(&mut self) -> Result<(), String> {
        with_doc!(self, end_page, doc => {
            doc.end_page().map_err(|e| {
                format!("end_page failed: {}", e)
            })
        })
    }

    /// End the document. Returns null for file-based docs,
    /// or a binary string for in-memory docs.
    pub fn end_document(&mut self) -> Result<Zval, String> {
        let inner = self
            .inner
            .take()
            .ok_or_else(|| "end_document: document already ended".to_string())?;
        match inner {
            DocumentInner::File(doc) => {
                let mut writer = doc
                    .end_document()
                    .map_err(|e| format!("end_document failed: {}", e,))?;
                writer
                    .flush()
                    .map_err(|e| format!("end_document flush failed: {}", e,))?;
                let mut zval = Zval::new();
                zval.set_null();
                Ok(zval)
            }
            DocumentInner::Memory(doc) => {
                let bytes = doc
                    .end_document()
                    .map_err(|e| format!("end_document failed: {}", e,))?;
                let mut zval = Zval::new();
                zval.set_binary(bytes);
                Ok(zval)
            }
        }
    }
}

fn parse_image_fit(s: &str) -> Result<ImageFit, String> {
    match s {
        "fit" => Ok(ImageFit::Fit),
        "fill" => Ok(ImageFit::Fill),
        "stretch" => Ok(ImageFit::Stretch),
        "none" => Ok(ImageFit::None),
        _ => Err(format!(
            "Invalid fit mode: '{}'. Valid: fit, fill, stretch, none",
            s
        )),
    }
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<PhpColor>()
        .class::<PhpTextStyle>()
        .class::<PhpRect>()
        .class::<PhpTextFlow>()
        .class::<PhpCellStyle>()
        .class::<PhpCell>()
        .class::<PhpRow>()
        .class::<PhpTable>()
        .class::<PhpTableCursor>()
        .class::<PhpPdfDocument>()
}
