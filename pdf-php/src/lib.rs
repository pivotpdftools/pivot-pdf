use std::fs::File;
use std::io::{BufWriter, Write};

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;

use pdf_core::{
    FitResult, PdfDocument, Rect, TextFlow, TextStyle,
};

// ----------------------------------------------------------
// TextStyle
// ----------------------------------------------------------

/// PHP class: TextStyle
///
/// ```php
/// $style = new TextStyle();          // defaults
/// $style = new TextStyle(true, 14.0); // bold, 14pt
/// ```
#[php_class(name = "TextStyle")]
pub struct PhpTextStyle {
    #[prop]
    pub bold: bool,
    #[prop]
    pub font_size: f64,
}

#[php_impl]
impl PhpTextStyle {
    pub fn __construct(bold: Option<bool>, font_size: Option<f64>) -> Self {
        PhpTextStyle {
            bold: bold.unwrap_or(false),
            font_size: font_size.unwrap_or(12.0),
        }
    }
}

impl PhpTextStyle {
    fn to_core(&self) -> TextStyle {
        TextStyle {
            bold: self.bold,
            font_size: self.font_size,
        }
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
    pub fn __construct(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Self {
        PhpRect { x, y, width, height }
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
/// $tf->addText("Bold", new TextStyle(true, 14.0));
/// ```
#[php_class(name = "TextFlow")]
pub struct PhpTextFlow {
    inner: TextFlow,
}

#[php_impl]
impl PhpTextFlow {
    pub fn __construct() -> Self {
        PhpTextFlow { inner: TextFlow::new() }
    }

    pub fn add_text(&mut self, text: &str, style: &PhpTextStyle) {
        let core_style = style.to_core();
        self.inner.add_text(text, &core_style);
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
                return Err(format!(
                    "{}: document already ended",
                    stringify!($name)
                ));
            }
        }
    };
}

/// PHP class: PdfDocument
///
/// ```php
/// $doc = PdfDocument::create("out.pdf");
/// $doc = PdfDocument::createInMemory();
/// ```
#[php_class(name = "PdfDocument")]
pub struct PhpPdfDocument {
    inner: Option<DocumentInner>,
}

#[php_impl]
impl PhpPdfDocument {
    pub fn create(path: &str) -> Result<Self, String> {
        let doc = PdfDocument::create(path)
            .map_err(|e| format!("create failed: {}", e))?;
        Ok(PhpPdfDocument {
            inner: Some(DocumentInner::File(doc)),
        })
    }

    pub fn create_in_memory() -> Result<Self, String> {
        let doc = PdfDocument::new(Vec::new())
            .map_err(|e| format!("create_in_memory failed: {}", e))?;
        Ok(PhpPdfDocument {
            inner: Some(DocumentInner::Memory(doc)),
        })
    }

    pub fn set_info(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(), String> {
        with_doc!(self, set_info, doc => {
            doc.set_info(key, value);
            Ok(())
        })
    }

    pub fn begin_page(
        &mut self,
        width: f64,
        height: f64,
    ) -> Result<(), String> {
        with_doc!(self, begin_page, doc => {
            doc.begin_page(width, height);
            Ok(())
        })
    }

    pub fn place_text(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
    ) -> Result<(), String> {
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
            let result = doc.fit_textflow(&mut flow.inner, &core_rect)
                .map_err(|e| format!("fit_textflow failed: {}", e))?;
            Ok(match result {
                FitResult::Stop => "stop".to_string(),
                FitResult::BoxFull => "box_full".to_string(),
                FitResult::BoxEmpty => "box_empty".to_string(),
            })
        })
    }

    pub fn end_page(&mut self) -> Result<(), String> {
        with_doc!(self, end_page, doc => {
            doc.end_page()
                .map_err(|e| format!("end_page failed: {}", e))
        })
    }

    /// End the document. Returns null for file-based docs,
    /// or a binary string for in-memory docs.
    pub fn end_document(&mut self) -> Result<Zval, String> {
        let inner = self.inner.take().ok_or_else(|| {
            "end_document: document already ended".to_string()
        })?;
        match inner {
            DocumentInner::File(doc) => {
                let mut writer = doc.end_document()
                    .map_err(|e| {
                        format!("end_document failed: {}", e)
                    })?;
                writer.flush().map_err(|e| {
                    format!("end_document flush failed: {}", e)
                })?;
                let mut zval = Zval::new();
                zval.set_null();
                Ok(zval)
            }
            DocumentInner::Memory(doc) => {
                let bytes = doc.end_document()
                    .map_err(|e| {
                        format!("end_document failed: {}", e)
                    })?;
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
