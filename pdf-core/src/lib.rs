//! PDF creation library for Rust.
//!
//! `pivot-pdf` provides a high-level API for building PDF documents
//! programmatically. It is designed for low memory usage, making it suitable
//! for SaaS applications that generate many documents (invoices, reports,
//! contracts, bills of material, etc.).
//!
//! # Quick Start
//!
//! ```no_run
//! use pivot_pdf::{DocumentOptions, PdfDocument, BuiltinFont, Rect, TextFlow, TextStyle};
//!
//! let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
//! doc.set_info("Title", "Hello World");
//! doc.begin_page(612.0, 792.0);
//! doc.place_text("Hello, world!", 72.0, 720.0);
//! doc.end_page().unwrap();
//! let pdf_bytes = doc.end_document().unwrap();
//! ```
//!
//! # Key Types
//!
//! - [`PdfDocument`] — the main entry point for building documents
//! - [`TextFlow`] — flowing styled text across multiple pages
//! - [`Table`] — rendering tabular data with flexible styling
//! - [`BuiltinFont`] — the 14 standard PDF fonts (no embedding required)
//!
//! # Feature Highlights
//!
//! - **Incremental page writing**: each page is flushed to the writer as it
//!   completes, keeping memory constant regardless of document size.
//! - **TrueType fonts**: embed custom fonts from `.ttf` files.
//! - **JPEG and PNG images**: embed with automatic fit/fill/stretch modes.
//! - **FlateDecode compression**: enable with [`PdfDocument::set_compression`].
//! - **PDF merge**: combine existing PDFs with [`merge_pdfs`].

#![warn(missing_docs)]

/// High-level PDF document builder.
pub mod document;
/// Font types and metrics.
pub mod fonts;
/// Color types for PDF graphics.
pub mod graphics;
/// Image loading and placement.
pub mod images;
/// PDF merge operations.
pub mod merger;
/// PDF object types (ISO 32000-1:2008 §7.3).
pub mod objects;
/// PDF reader for parsing existing PDF files.
pub mod reader;
/// Table rendering for tabular data.
pub mod tables;
/// Flowing text layout across bounding boxes.
pub mod textflow;
/// TrueType font parsing and embedding.
pub mod truetype;
/// Low-level PDF binary writer.
pub mod writer;

pub use document::{DocumentOptions, FormFieldError, Origin, PdfDocument};
pub use fonts::{BuiltinFont, FontRef, TrueTypeFontId};
pub use graphics::Color;
pub use images::{ImageFit, ImageId};
pub use merger::{merge_pdfs, MergeOptions, PdfMergeError};
pub use reader::{PdfReadError, PdfReader};
pub use tables::{Cell, CellOverflow, CellStyle, Row, Table, TableCursor, TextAlign};
pub use textflow::{FitResult, Rect, TextFlow, TextStyle, WordBreak};
