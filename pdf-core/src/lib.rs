pub mod document;
pub mod fonts;
pub mod graphics;
pub mod images;
pub mod objects;
pub mod reader;
pub mod tables;
pub mod textflow;
pub mod truetype;
pub mod writer;

pub use document::PdfDocument;
pub use fonts::{BuiltinFont, FontRef, TrueTypeFontId};
pub use graphics::Color;
pub use images::{ImageFit, ImageId};
pub use reader::{PdfReadError, PdfReader};
pub use tables::{Cell, CellOverflow, CellStyle, Row, Table, TableCursor, TextAlign};
pub use textflow::{FitResult, Rect, TextFlow, TextStyle, WordBreak};
