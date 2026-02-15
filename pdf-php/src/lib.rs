use std::fs::File;
use std::io::{BufWriter, Write};

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;

use pdf_core::{
    BuiltinFont, Color, FitResult, FontRef, PdfDocument, Rect, TextFlow, TextStyle, TrueTypeFontId,
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
#[php_class(name = "Color")]
pub struct PhpColor {
    #[prop]
    pub r: f64,
    #[prop]
    pub g: f64,
    #[prop]
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
#[php_class(name = "TextStyle")]
pub struct PhpTextStyle {
    #[prop]
    pub font_name: String,
    #[prop]
    pub font_size: f64,
    /// -1 means builtin (use font_name), >= 0 means TrueType
    #[prop]
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
#[php_class(name = "Rect")]
pub struct PhpRect {
    #[prop]
    pub x: f64,
    #[prop]
    pub y: f64,
    #[prop]
    pub width: f64,
    #[prop]
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
/// ```
#[php_class(name = "TextFlow")]
pub struct PhpTextFlow {
    inner: TextFlow,
}

#[php_impl]
impl PhpTextFlow {
    pub fn __construct() -> Self {
        PhpTextFlow {
            inner: TextFlow::new(),
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
#[php_class(name = "PdfDocument")]
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

    pub fn fit_textflow(
        &mut self,
        flow: &mut PhpTextFlow,
        rect: &PhpRect,
    ) -> Result<String, String> {
        let core_rect = rect.to_core();
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

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
