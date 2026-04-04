use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::fonts::{BuiltinFont, FontRef, TrueTypeFontId};
use crate::graphics::{Angle, Color};
use crate::images::{self, ImageData, ImageFit, ImageFormat, ImageId};
use crate::objects::{ObjId, PdfObject};
use crate::tables::{Row, Table, TableCursor};
use crate::textflow::{FitResult, Rect, TextFlow, TextStyle};
use crate::truetype::TrueTypeFont;
use crate::writer::PdfWriter;

// -------------------------------------------------------
// Coordinate origin types
// -------------------------------------------------------

/// Coordinate origin used for all user-supplied coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// PDF's native bottom-left origin; y increases upward. This is the default.
    BottomLeft,
    /// Screen/web-style top-left origin; y increases downward.
    TopLeft,
}

/// Options for configuring a new PDF document.
#[derive(Debug, Clone)]
pub struct DocumentOptions {
    /// Coordinate origin for all user-supplied coordinates.
    /// Defaults to `Origin::BottomLeft` (PDF native).
    pub origin: Origin,
}

impl Default for DocumentOptions {
    fn default() -> Self {
        DocumentOptions {
            origin: Origin::BottomLeft,
        }
    }
}

// -------------------------------------------------------
// Form field types
// -------------------------------------------------------

/// Errors that can occur when adding form fields to a document.
#[derive(Debug)]
pub enum FormFieldError {
    /// `add_text_field` was called without an active page.
    NoActivePage,
    /// A field with the given name already exists in this document.
    DuplicateName(String),
}

impl fmt::Display for FormFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormFieldError::NoActivePage => write!(f, "add_text_field called with no active page"),
            FormFieldError::DuplicateName(name) => {
                write!(f, "duplicate field name: '{}'", name)
            }
        }
    }
}

impl std::error::Error for FormFieldError {}

/// A form field definition accumulated while a page is open.
struct FormFieldDef {
    name: String,
    rect: Rect,
}

/// A form field with a pre-allocated PDF object ID, stored in `PageRecord`.
struct FormFieldRecord {
    name: String,
    rect: Rect,
    obj_id: ObjId,
}

const CATALOG_OBJ: ObjId = ObjId(1, 0);
const PAGES_OBJ: ObjId = ObjId(2, 0);
const FIRST_PAGE_OBJ_NUM: u32 = 3;

/// Pre-allocated object IDs for an image XObject.
struct ImageObjIds {
    xobject: ObjId,
    smask: Option<ObjId>,
    pdf_name: String,
}

/// Pre-allocated object IDs for a TrueType font's PDF objects.
struct TrueTypeFontObjIds {
    type0: ObjId,
    cid_font: ObjId,
    descriptor: ObjId,
    font_file: ObjId,
    tounicode: ObjId,
}

/// Accumulated record for a completed page.
/// Page dictionaries are deferred until `end_document()` so that
/// overlay content streams (e.g. page numbers) can be appended
/// after all pages have been written.
struct PageRecord {
    obj_id: ObjId,
    /// Content stream IDs: first is the main stream, any beyond that are overlays.
    content_ids: Vec<ObjId>,
    width: f64,
    height: f64,
    used_fonts: BTreeSet<BuiltinFont>,
    used_truetype_fonts: BTreeSet<usize>,
    used_images: BTreeSet<usize>,
    /// Form fields on this page with pre-allocated object IDs.
    fields: Vec<FormFieldRecord>,
}

/// High-level API for building PDF documents.
///
/// Generic over `Write` so it works with files (`BufWriter<File>`),
/// in-memory buffers (`Vec<u8>`), or any other writer.
///
/// Pages are written incrementally: `end_page()` flushes page data
/// to the writer and frees page content from memory. This keeps
/// memory usage low even for documents with hundreds of pages.
pub struct PdfDocument<W: Write> {
    writer: PdfWriter<W>,
    info: Vec<(String, String)>,
    page_records: Vec<PageRecord>,
    current_page: Option<PageBuilder>,
    next_obj_num: u32,
    /// Maps each used builtin font to its written ObjId.
    font_obj_ids: BTreeMap<BuiltinFont, ObjId>,
    /// Loaded TrueType fonts.
    truetype_fonts: Vec<TrueTypeFont>,
    /// Pre-allocated ObjIds for TrueType fonts (by index).
    truetype_font_obj_ids: BTreeMap<usize, TrueTypeFontObjIds>,
    /// Next font number for PDF resource names (F15, F16, ...).
    next_font_num: u32,
    /// Whether to compress stream objects with FlateDecode.
    compress: bool,
    /// Loaded images.
    images: Vec<ImageData>,
    /// Pre-allocated ObjIds for images (by index).
    image_obj_ids: BTreeMap<usize, ImageObjIds>,
    /// Images whose XObjects have already been written.
    written_images: BTreeSet<usize>,
    /// Next image number for PDF resource names (Im1, Im2, ...).
    next_image_num: u32,
    /// Document-level set of used form field names (enforces uniqueness).
    form_field_names: BTreeSet<String>,
    /// Coordinate origin for all user-supplied coordinates.
    origin: Origin,
}

struct PageBuilder {
    width: f64,
    height: f64,
    content_ops: Vec<u8>,
    used_fonts: BTreeSet<BuiltinFont>,
    used_truetype_fonts: BTreeSet<usize>,
    used_images: BTreeSet<usize>,
    /// When `Some(idx)`, this builder is adding an overlay to `page_records[idx]`
    /// rather than creating a new page.
    overlay_for: Option<usize>,
    /// Form fields added while this page was open.
    fields: Vec<FormFieldDef>,
}

impl PdfDocument<BufWriter<File>> {
    /// Create a new PDF document that writes to a file.
    pub fn create<P: AsRef<Path>>(path: P, options: DocumentOptions) -> io::Result<Self> {
        let file = File::create(path)?;
        Self::new(BufWriter::new(file), options)
    }
}

impl<W: Write> PdfDocument<W> {
    /// Create a new PDF document that writes to the given writer.
    /// Writes the PDF header immediately.
    pub fn new(writer: W, options: DocumentOptions) -> io::Result<Self> {
        let mut pdf_writer = PdfWriter::new(writer);
        pdf_writer.write_header()?;

        Ok(PdfDocument {
            writer: pdf_writer,
            info: Vec::new(),
            page_records: Vec::new(),
            current_page: None,
            next_obj_num: FIRST_PAGE_OBJ_NUM,
            font_obj_ids: BTreeMap::new(),
            truetype_fonts: Vec::new(),
            truetype_font_obj_ids: BTreeMap::new(),
            next_font_num: 15,
            compress: false,
            images: Vec::new(),
            image_obj_ids: BTreeMap::new(),
            written_images: BTreeSet::new(),
            next_image_num: 1,
            form_field_names: BTreeSet::new(),
            origin: options.origin,
        })
    }

    /// Set a document info entry (e.g. "Creator", "Title").
    pub fn set_info(&mut self, key: &str, value: &str) -> &mut Self {
        self.info.push((key.to_string(), value.to_string()));
        self
    }

    /// Enable or disable FlateDecode compression for stream objects.
    /// When enabled, page content, embedded fonts, and ToUnicode CMaps
    /// are compressed, typically reducing file size by 50-80%.
    /// Disabled by default.
    pub fn set_compression(&mut self, enabled: bool) -> &mut Self {
        self.compress = enabled;
        self
    }

    /// Load a TrueType font from a file path.
    /// Returns a FontRef that can be used in TextStyle.
    pub fn load_font_file<P: AsRef<Path>>(&mut self, path: P) -> Result<FontRef, String> {
        let data =
            std::fs::read(path.as_ref()).map_err(|e| format!("Failed to read font file: {}", e))?;
        self.load_font_bytes(data)
    }

    /// Load a TrueType font from raw bytes.
    /// Returns a FontRef that can be used in TextStyle.
    pub fn load_font_bytes(&mut self, data: Vec<u8>) -> Result<FontRef, String> {
        let font_num = self.next_font_num;
        self.next_font_num += 1;
        let font = TrueTypeFont::from_bytes(data, font_num)?;
        let idx = self.truetype_fonts.len();
        self.truetype_fonts.push(font);
        Ok(FontRef::TrueType(TrueTypeFontId(idx)))
    }

    /// Returns the number of completed pages (pages for which `end_page` has been called).
    pub fn page_count(&self) -> usize {
        self.page_records.len()
    }

    /// Begin a new page with the given dimensions in points.
    /// If a page is currently open, it is automatically closed.
    pub fn begin_page(&mut self, width: f64, height: f64) -> &mut Self {
        if self.current_page.is_some() {
            let _ = self.end_page();
        }
        self.current_page = Some(PageBuilder {
            width,
            height,
            content_ops: Vec::new(),
            used_fonts: BTreeSet::new(),
            used_truetype_fonts: BTreeSet::new(),
            used_images: BTreeSet::new(),
            overlay_for: None,
            fields: Vec::new(),
        });
        self
    }

    /// Open a completed page for editing (1-indexed).
    ///
    /// Used for adding overlay content such as page numbers ("Page X of Y")
    /// after all pages have been written. The overlay content is written as
    /// an additional content stream appended to the page's `/Contents` array.
    ///
    /// If a page is currently open, it is automatically closed first.
    ///
    /// Returns an error if `page_num` is out of range.
    pub fn open_page(&mut self, page_num: usize) -> io::Result<()> {
        if page_num == 0 || page_num > self.page_records.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "open_page: page_num {} out of range (1..={})",
                    page_num,
                    self.page_records.len()
                ),
            ));
        }

        if self.current_page.is_some() {
            self.end_page()?;
        }

        let idx = page_num - 1;
        let width = self.page_records[idx].width;
        let height = self.page_records[idx].height;

        self.current_page = Some(PageBuilder {
            width,
            height,
            content_ops: Vec::new(),
            used_fonts: BTreeSet::new(),
            used_truetype_fonts: BTreeSet::new(),
            used_images: BTreeSet::new(),
            overlay_for: Some(idx),
            fields: Vec::new(),
        });

        Ok(())
    }

    /// Place text at position (x, y) using default 12pt Helvetica.
    ///
    /// With [`Origin::BottomLeft`] (default), `(x, y)` is in PDF's native
    /// bottom-left coordinate system. With [`Origin::TopLeft`], `y` is measured
    /// from the top of the page, increasing downward.
    pub fn place_text(&mut self, text: &str, x: f64, y: f64) -> &mut Self {
        let y_pdf = self.transform_y(y);
        let page = self
            .current_page
            .as_mut()
            .expect("place_text called with no open page");
        page.used_fonts.insert(BuiltinFont::Helvetica);
        let escaped = crate::writer::escape_pdf_string(text);
        let ops = format!(
            "BT\n/F1 12 Tf\n{} {} Td\n({}) Tj\nET\n",
            format_coord(x),
            format_coord(y_pdf),
            escaped,
        );
        page.content_ops.extend_from_slice(ops.as_bytes());
        self
    }

    /// Place text at position (x, y) with the given style.
    ///
    /// With [`Origin::BottomLeft`] (default), `(x, y)` is in PDF's native
    /// bottom-left coordinate system. With [`Origin::TopLeft`], `y` is measured
    /// from the top of the page, increasing downward.
    pub fn place_text_styled(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        style: &TextStyle,
    ) -> &mut Self {
        // Encode text before borrowing page mutably
        let (font_name, text_op) = match style.font {
            FontRef::Builtin(b) => {
                let escaped = crate::writer::escape_pdf_string(text);
                (b.pdf_name().to_string(), format!("({}) Tj", escaped))
            }
            FontRef::TrueType(id) => {
                let font = &mut self.truetype_fonts[id.0];
                let hex = font.encode_text_hex(text);
                (font.pdf_name.clone(), format!("{} Tj", hex))
            }
        };

        let y_pdf = self.transform_y(y);
        let page = self
            .current_page
            .as_mut()
            .expect("place_text_styled called with no open page");

        match style.font {
            FontRef::Builtin(b) => {
                page.used_fonts.insert(b);
            }
            FontRef::TrueType(id) => {
                page.used_truetype_fonts.insert(id.0);
            }
        }

        let ops = format!(
            "BT\n/{} {} Tf\n{} {} Td\n{}\nET\n",
            font_name,
            format_coord(style.font_size),
            format_coord(x),
            format_coord(y_pdf),
            text_op,
        );
        page.content_ops.extend_from_slice(ops.as_bytes());
        self
    }

    /// Fit a TextFlow into a bounding rectangle on the current page.
    ///
    /// The flow's cursor advances so subsequent calls continue where it left off
    /// (for multi-page flow). With [`Origin::TopLeft`], `(rect.x, rect.y)` is
    /// the top-left corner of the area.
    pub fn fit_textflow(&mut self, flow: &mut TextFlow, rect: &Rect) -> io::Result<FitResult> {
        let pdf_rect = self.transform_rect_top_edge(rect);
        let (ops, result, used_fonts) =
            flow.generate_content_ops(&pdf_rect, &mut self.truetype_fonts);

        let page = self
            .current_page
            .as_mut()
            .expect("fit_textflow called with no open page");
        page.content_ops.extend_from_slice(&ops);
        page.used_fonts.extend(used_fonts.builtin);
        page.used_truetype_fonts.extend(used_fonts.truetype);
        Ok(result)
    }

    /// Place a single table row on the current page.
    ///
    /// `cursor` tracks the current Y position within the page. Pass the same
    /// cursor to successive calls; call `cursor.reset()` when starting a new page.
    ///
    /// Returns:
    /// - `Stop`     — row placed; advance to the next row.
    /// - `BoxFull`  — page full; end the page, begin a new one, reset the cursor, retry.
    /// - `BoxEmpty` — rect too small for this row even from the top; skip or abort.
    pub fn fit_row(
        &mut self,
        table: &Table,
        row: &Row,
        cursor: &mut TableCursor,
    ) -> io::Result<FitResult> {
        let total_span: usize = row.cells.iter().map(|c| c.col_span.max(1)).sum();
        if total_span != table.columns.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "row col_span sum ({}) must equal table column count ({})",
                    total_span,
                    table.columns.len()
                ),
            ));
        }

        let (ops, result, used_fonts) = match self.origin {
            Origin::BottomLeft => table.generate_row_ops(row, cursor, &mut self.truetype_fonts),
            Origin::TopLeft => {
                let page_h = self.current_page_height();
                // Convert cursor from user (TopLeft) to PDF top-edge coords.
                let pdf_top = page_h - cursor.rect.y;
                let pdf_rect = Rect {
                    x: cursor.rect.x,
                    y: pdf_top,
                    width: cursor.rect.width,
                    height: cursor.rect.height,
                };
                let pdf_current_y = page_h - cursor.current_y;
                let mut pdf_cursor = crate::tables::TableCursor {
                    rect: pdf_rect,
                    current_y: pdf_current_y,
                    first_row: cursor.first_row,
                };
                let result = table.generate_row_ops(row, &mut pdf_cursor, &mut self.truetype_fonts);
                // Back-transform: current_y in PDF → user space.
                cursor.current_y = page_h - pdf_cursor.current_y;
                cursor.first_row = pdf_cursor.first_row;
                result
            }
        };

        let page = self
            .current_page
            .as_mut()
            .expect("fit_row called with no open page");
        page.content_ops.extend_from_slice(&ops);
        page.used_fonts.extend(used_fonts.builtin);
        page.used_truetype_fonts.extend(used_fonts.truetype);
        Ok(result)
    }

    // -------------------------------------------------------
    // Image operations
    // -------------------------------------------------------

    /// Load an image from a file path.
    /// Returns an ImageId that can be used with `place_image`.
    pub fn load_image_file<P: AsRef<Path>>(&mut self, path: P) -> Result<ImageId, String> {
        let data = std::fs::read(path.as_ref())
            .map_err(|e| format!("Failed to read image file: {}", e))?;
        self.load_image_bytes(data)
    }

    /// Load an image from raw bytes (JPEG or PNG).
    /// Returns an ImageId that can be used with `place_image`.
    pub fn load_image_bytes(&mut self, data: Vec<u8>) -> Result<ImageId, String> {
        let image_data = images::load_image(data)?;
        let idx = self.images.len();
        self.images.push(image_data);
        Ok(ImageId(idx))
    }

    /// Place an image on the current page within the given bounding rect.
    ///
    /// With [`Origin::TopLeft`], `(rect.x, rect.y)` is the top-left corner and
    /// `rect.height` extends downward. With [`Origin::BottomLeft`] (default),
    /// `rect.y` is the bottom edge in PDF space.
    pub fn place_image(&mut self, image: &ImageId, rect: &Rect, fit: ImageFit) -> &mut Self {
        let idx = image.0;
        let img = &self.images[idx];

        // Transform rect to PDF bottom-left space before computing placement.
        let pdf_rect = self.transform_rect(rect);
        let placement = images::calculate_placement(img.width, img.height, &pdf_rect, fit);

        self.ensure_image_obj_ids(idx);
        let pdf_name = self.image_obj_ids[&idx].pdf_name.clone();

        let page = self
            .current_page
            .as_mut()
            .expect("place_image called with no open page");
        page.used_images.insert(idx);

        // Build content stream operators
        let mut ops = String::new();
        ops.push_str("q\n");

        // Clipping (for Fill mode)
        if let Some(clip) = &placement.clip {
            ops.push_str(&format!(
                "{} {} {} {} re W n\n",
                format_coord(clip.x),
                format_coord(clip.y),
                format_coord(clip.width),
                format_coord(clip.height),
            ));
        }

        // Transformation matrix: scale and position
        // cm matrix: [width 0 0 height x y]
        ops.push_str(&format!(
            "{} 0 0 {} {} {} cm\n",
            format_coord(placement.width),
            format_coord(placement.height),
            format_coord(placement.x),
            format_coord(placement.y),
        ));

        // Paint the image
        ops.push_str(&format!("/{} Do\n", pdf_name));
        ops.push_str("Q\n");

        page.content_ops.extend_from_slice(ops.as_bytes());
        self
    }

    /// Add a fillable text field to the current page.
    ///
    /// `name` must be unique across the document. Returns an error if called
    /// with no active page or if the name has already been used.
    ///
    /// With [`Origin::TopLeft`], `(rect.x, rect.y)` is the top-left corner.
    pub fn add_text_field(&mut self, name: &str, rect: Rect) -> Result<(), FormFieldError> {
        if self.current_page.is_none() {
            return Err(FormFieldError::NoActivePage);
        }
        if self.form_field_names.contains(name) {
            return Err(FormFieldError::DuplicateName(name.to_string()));
        }
        let pdf_rect = self.transform_rect(&rect);
        self.form_field_names.insert(name.to_string());
        let page = self.current_page.as_mut().unwrap();
        page.fields.push(FormFieldDef {
            name: name.to_string(),
            rect: pdf_rect,
        });
        Ok(())
    }

    /// Pre-allocate ObjIds for an image if not yet done.
    fn ensure_image_obj_ids(&mut self, idx: usize) {
        if self.image_obj_ids.contains_key(&idx) {
            return;
        }
        let xobject = ObjId(self.next_obj_num, 0);
        self.next_obj_num += 1;

        let smask = if self.images[idx].smask_data.is_some() {
            let id = ObjId(self.next_obj_num, 0);
            self.next_obj_num += 1;
            Some(id)
        } else {
            None
        };

        let pdf_name = format!("Im{}", self.next_image_num);
        self.next_image_num += 1;

        self.image_obj_ids.insert(
            idx,
            ImageObjIds {
                xobject,
                smask,
                pdf_name,
            },
        );
    }

    /// Write the image XObject stream(s) for the given image index.
    fn write_image_xobject(&mut self, idx: usize) -> io::Result<()> {
        if self.written_images.contains(&idx) {
            return Ok(());
        }

        let img = &self.images[idx];
        let obj_ids = &self.image_obj_ids[&idx];
        let xobject_id = obj_ids.xobject;
        let smask_id = obj_ids.smask;

        // Write SMask XObject first if alpha data exists
        if let (Some(smask_obj_id), Some(smask_data)) = (smask_id, img.smask_data.as_ref()) {
            let smask_stream = self.make_stream(
                vec![
                    ("Type", PdfObject::name("XObject")),
                    ("Subtype", PdfObject::name("Image")),
                    ("Width", PdfObject::Integer(img.width as i64)),
                    ("Height", PdfObject::Integer(img.height as i64)),
                    ("ColorSpace", PdfObject::name("DeviceGray")),
                    ("BitsPerComponent", PdfObject::Integer(8)),
                ],
                smask_data.clone(),
            );
            self.writer.write_object(smask_obj_id, &smask_stream)?;
        }

        // Build image XObject dict entries
        let mut entries: Vec<(&str, PdfObject)> = vec![
            ("Type", PdfObject::name("XObject")),
            ("Subtype", PdfObject::name("Image")),
            ("Width", PdfObject::Integer(img.width as i64)),
            ("Height", PdfObject::Integer(img.height as i64)),
            ("ColorSpace", PdfObject::name(img.color_space.pdf_name())),
            (
                "BitsPerComponent",
                PdfObject::Integer(img.bits_per_component as i64),
            ),
        ];

        if let Some(smask_obj_id) = smask_id {
            entries.push(("SMask", PdfObject::Reference(smask_obj_id)));
        }

        // For JPEG: embed raw data with DCTDecode, never double-compress
        // For PNG (decoded pixels): use make_stream for optional FlateDecode
        let image_obj = match img.format {
            ImageFormat::Jpeg => {
                entries.push(("Filter", PdfObject::name("DCTDecode")));
                PdfObject::stream(entries, img.data.clone())
            }
            ImageFormat::Png => self.make_stream(entries, img.data.clone()),
        };

        self.writer.write_object(xobject_id, &image_obj)?;
        self.written_images.insert(idx);
        Ok(())
    }

    // -------------------------------------------------------
    // Graphics operations
    // -------------------------------------------------------

    /// Set the stroke color (PDF `RG` operator).
    pub fn set_stroke_color(&mut self, color: Color) -> &mut Self {
        let page = self
            .current_page
            .as_mut()
            .expect("set_stroke_color called with no open page");
        let ops = format!(
            "{} {} {} RG\n",
            format_coord(color.r),
            format_coord(color.g),
            format_coord(color.b),
        );
        page.content_ops.extend_from_slice(ops.as_bytes());
        self
    }

    /// Set the fill color (PDF `rg` operator).
    pub fn set_fill_color(&mut self, color: Color) -> &mut Self {
        let page = self
            .current_page
            .as_mut()
            .expect("set_fill_color called with no open page");
        let ops = format!(
            "{} {} {} rg\n",
            format_coord(color.r),
            format_coord(color.g),
            format_coord(color.b),
        );
        page.content_ops.extend_from_slice(ops.as_bytes());
        self
    }

    /// Set the line width (PDF `w` operator).
    pub fn set_line_width(&mut self, width: f64) -> &mut Self {
        let page = self
            .current_page
            .as_mut()
            .expect("set_line_width called with no open page");
        let ops = format!("{} w\n", format_coord(width));
        page.content_ops.extend_from_slice(ops.as_bytes());
        self
    }

    /// Move to a point without drawing (PDF `m` operator).
    ///
    /// With [`Origin::TopLeft`], `y` is measured from the top of the page,
    /// increasing downward.
    pub fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        let y_pdf = self.transform_y(y);
        let page = self
            .current_page
            .as_mut()
            .expect("move_to called with no open page");
        let ops = format!("{} {} m\n", format_coord(x), format_coord(y_pdf));
        page.content_ops.extend_from_slice(ops.as_bytes());
        self
    }

    /// Draw a line to a point (PDF `l` operator).
    ///
    /// With [`Origin::TopLeft`], `y` is measured from the top of the page,
    /// increasing downward.
    pub fn line_to(&mut self, x: f64, y: f64) -> &mut Self {
        let y_pdf = self.transform_y(y);
        let page = self
            .current_page
            .as_mut()
            .expect("line_to called with no open page");
        let ops = format!("{} {} l\n", format_coord(x), format_coord(y_pdf));
        page.content_ops.extend_from_slice(ops.as_bytes());
        self
    }

    /// Append a rectangle to the path (PDF `re` operator).
    ///
    /// With [`Origin::TopLeft`], `(x, y)` is the **top-left** corner of the
    /// rectangle and `height` extends downward.
    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) -> &mut Self {
        let r = self.transform_rect(&Rect {
            x,
            y,
            width,
            height,
        });
        let page = self
            .current_page
            .as_mut()
            .expect("rect called with no open page");
        let ops = format!(
            "{} {} {} {} re\n",
            format_coord(r.x),
            format_coord(r.y),
            format_coord(r.width),
            format_coord(r.height),
        );
        page.content_ops.extend_from_slice(ops.as_bytes());
        self
    }

    /// Append a cubic Bezier curve to the path (PDF `c` operator).
    ///
    /// `(x1, y1)` and `(x2, y2)` are the two control points; `(x3, y3)` is
    /// the endpoint. All y-coordinates are transformed according to the
    /// document's origin setting.
    pub fn curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> &mut Self {
        let y1_pdf = self.transform_y(y1);
        let y2_pdf = self.transform_y(y2);
        let y3_pdf = self.transform_y(y3);
        let page = self
            .current_page
            .as_mut()
            .expect("curve_to called with no open page");
        let ops = format!(
            "{} {} {} {} {} {} c\n",
            format_coord(x1),
            format_coord(y1_pdf),
            format_coord(x2),
            format_coord(y2_pdf),
            format_coord(x3),
            format_coord(y3_pdf),
        );
        page.content_ops.extend_from_slice(ops.as_bytes());
        self
    }

    /// Append an arc to the current path.
    ///
    /// The arc is centered at `(cx, cy)` with the given `radius`. Angles follow
    /// standard math convention: 0° = right, counter-clockwise positive.
    /// Use [`Angle::degrees`] or [`Angle::radians`] to construct the angle.
    ///
    /// The arc is approximated with cubic Bezier segments (up to one per 90°).
    /// This method moves to the arc's start point; the caller is responsible for
    /// painting (stroke/fill/fill_stroke).
    ///
    /// With [`Origin::TopLeft`], `(cx, cy)` is measured from the top of the page.
    pub fn arc(&mut self, cx: f64, cy: f64, radius: f64, start: Angle, end: Angle) -> &mut Self {
        let start_rad = start.to_radians();
        let end_rad = end.to_radians();

        // Start point in user space
        let sx = cx + radius * start_rad.cos();
        let sy = cy + radius * start_rad.sin();
        self.move_to(sx, sy);

        for (x1, y1, x2, y2, x3, y3) in arc_bezier_segments(cx, cy, radius, start_rad, end_rad) {
            self.curve_to(x1, y1, x2, y2, x3, y3);
        }
        self
    }

    /// Append a full circle to the current path (closed).
    ///
    /// The circle is centered at `(cx, cy)` with the given `radius`. The path
    /// is automatically closed with the `h` operator; the caller is responsible
    /// for painting (stroke/fill/fill_stroke).
    ///
    /// With [`Origin::TopLeft`], `(cx, cy)` is measured from the top of the page.
    pub fn circle(&mut self, cx: f64, cy: f64, radius: f64) -> &mut Self {
        self.arc(
            cx,
            cy,
            radius,
            Angle::radians(0.0),
            Angle::radians(std::f64::consts::TAU),
        )
        .close_path()
    }

    /// Close the current subpath (PDF `h` operator).
    pub fn close_path(&mut self) -> &mut Self {
        let page = self
            .current_page
            .as_mut()
            .expect("close_path called with no open page");
        page.content_ops.extend_from_slice(b"h\n");
        self
    }

    /// Stroke the current path (PDF `S` operator).
    pub fn stroke(&mut self) -> &mut Self {
        let page = self
            .current_page
            .as_mut()
            .expect("stroke called with no open page");
        page.content_ops.extend_from_slice(b"S\n");
        self
    }

    /// Fill the current path (PDF `f` operator).
    pub fn fill(&mut self) -> &mut Self {
        let page = self
            .current_page
            .as_mut()
            .expect("fill called with no open page");
        page.content_ops.extend_from_slice(b"f\n");
        self
    }

    /// Fill and stroke the current path (PDF `B` operator).
    pub fn fill_stroke(&mut self) -> &mut Self {
        let page = self
            .current_page
            .as_mut()
            .expect("fill_stroke called with no open page");
        page.content_ops.extend_from_slice(b"B\n");
        self
    }

    /// Save the graphics state (PDF `q` operator).
    pub fn save_state(&mut self) -> &mut Self {
        let page = self
            .current_page
            .as_mut()
            .expect("save_state called with no open page");
        page.content_ops.extend_from_slice(b"q\n");
        self
    }

    /// Restore the graphics state (PDF `Q` operator).
    pub fn restore_state(&mut self) -> &mut Self {
        let page = self
            .current_page
            .as_mut()
            .expect("restore_state called with no open page");
        page.content_ops.extend_from_slice(b"Q\n");
        self
    }

    // -------------------------------------------------------
    // Coordinate transform helpers
    // -------------------------------------------------------

    /// Transform a user y-coordinate to PDF space.
    /// With `TopLeft`, flips y: `page_height - y`.
    /// With `BottomLeft`, returns y unchanged.
    fn transform_y(&self, y: f64) -> f64 {
        match self.origin {
            Origin::BottomLeft => y,
            Origin::TopLeft => {
                let page_height = self
                    .current_page
                    .as_ref()
                    .expect("transform_y called with no open page")
                    .height;
                page_height - y
            }
        }
    }

    /// Transform a user-space rect to PDF bottom-left space.
    ///
    /// With `TopLeft`, `(x, y)` is the top-left corner; transforms to
    /// PDF bottom-left: `y_pdf_bottom = page_height - y_user - height`.
    ///
    /// With `BottomLeft`, returns the rect unchanged (`y` is already the
    /// bottom edge in PDF space).
    fn transform_rect(&self, rect: &Rect) -> Rect {
        match self.origin {
            Origin::BottomLeft => *rect,
            Origin::TopLeft => {
                let page_height = self
                    .current_page
                    .as_ref()
                    .expect("transform_rect called with no open page")
                    .height;
                Rect {
                    x: rect.x,
                    y: page_height - rect.y - rect.height,
                    width: rect.width,
                    height: rect.height,
                }
            }
        }
    }

    /// Transform a user-space rect to the "top-edge-in-PDF-space" format
    /// used by the text layout and table engines (where `y` = top edge,
    /// decreasing downward).
    ///
    /// With `TopLeft`, `y_top_pdf = page_height - y_user`.
    /// With `BottomLeft`, returns the rect unchanged (y is already top-edge
    /// in PDF space for layout engines).
    fn transform_rect_top_edge(&self, rect: &Rect) -> Rect {
        match self.origin {
            Origin::BottomLeft => *rect,
            Origin::TopLeft => {
                let page_height = self
                    .current_page
                    .as_ref()
                    .expect("transform_rect_top_edge called with no open page")
                    .height;
                Rect {
                    x: rect.x,
                    y: page_height - rect.y,
                    width: rect.width,
                    height: rect.height,
                }
            }
        }
    }

    /// Current page height, panics if no page is open.
    fn current_page_height(&self) -> f64 {
        self.current_page.as_ref().expect("no open page").height
    }

    /// Build a stream object, optionally compressing the data with FlateDecode.
    fn make_stream(&self, mut dict_entries: Vec<(&str, PdfObject)>, data: Vec<u8>) -> PdfObject {
        if self.compress {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&data).expect("flate2 in-memory write");
            let compressed = encoder.finish().expect("flate2 finish");
            dict_entries.push(("Filter", PdfObject::name("FlateDecode")));
            PdfObject::stream(dict_entries, compressed)
        } else {
            PdfObject::stream(dict_entries, data)
        }
    }

    /// Ensure a builtin font's dictionary object has been written.
    fn ensure_font_written(&mut self, font: BuiltinFont) -> io::Result<ObjId> {
        if let Some(&id) = self.font_obj_ids.get(&font) {
            return Ok(id);
        }
        let id = ObjId(self.next_obj_num, 0);
        self.next_obj_num += 1;
        let obj = PdfObject::dict(vec![
            ("Type", PdfObject::name("Font")),
            ("Subtype", PdfObject::name("Type1")),
            ("BaseFont", PdfObject::name(font.pdf_base_name())),
        ]);
        self.writer.write_object(id, &obj)?;
        self.font_obj_ids.insert(font, id);
        Ok(id)
    }

    /// Pre-allocate ObjIds for a TrueType font if not yet done.
    fn ensure_tt_font_obj_ids(&mut self, idx: usize) -> &TrueTypeFontObjIds {
        if !self.truetype_font_obj_ids.contains_key(&idx) {
            let type0 = ObjId(self.next_obj_num, 0);
            self.next_obj_num += 1;
            let cid_font = ObjId(self.next_obj_num, 0);
            self.next_obj_num += 1;
            let descriptor = ObjId(self.next_obj_num, 0);
            self.next_obj_num += 1;
            let font_file = ObjId(self.next_obj_num, 0);
            self.next_obj_num += 1;
            let tounicode = ObjId(self.next_obj_num, 0);
            self.next_obj_num += 1;
            self.truetype_font_obj_ids.insert(
                idx,
                TrueTypeFontObjIds {
                    type0,
                    cid_font,
                    descriptor,
                    font_file,
                    tounicode,
                },
            );
        }
        &self.truetype_font_obj_ids[&idx]
    }

    /// End the current page. Writes the content stream to the writer
    /// and frees page content from memory. The page dictionary is
    /// deferred until `end_document()` so overlay streams can be added.
    pub fn end_page(&mut self) -> io::Result<()> {
        let page = self
            .current_page
            .take()
            .expect("end_page called with no open page");

        // Write builtin font objects for any not yet written
        for &font in &page.used_fonts {
            self.ensure_font_written(font)?;
        }

        // Pre-allocate ObjIds for TrueType fonts used on this page
        for &idx in &page.used_truetype_fonts {
            self.ensure_tt_font_obj_ids(idx);
        }

        // Write image XObjects for images used on this page
        let used_images: Vec<usize> = page.used_images.iter().copied().collect();
        for idx in &used_images {
            self.write_image_xobject(*idx)?;
        }

        let content_id = ObjId(self.next_obj_num, 0);
        self.next_obj_num += 1;

        // Write content stream immediately (keeps memory usage low)
        let content_stream = self.make_stream(vec![], page.content_ops);
        self.writer.write_object(content_id, &content_stream)?;

        // Pre-allocate ObjIds for form fields on this page
        let field_records: Vec<FormFieldRecord> = page
            .fields
            .into_iter()
            .map(|def| {
                let obj_id = ObjId(self.next_obj_num, 0);
                self.next_obj_num += 1;
                FormFieldRecord {
                    name: def.name,
                    rect: def.rect,
                    obj_id,
                }
            })
            .collect();

        match page.overlay_for {
            None => {
                // New page: pre-allocate the page dict ObjId and store the record.
                // The page dictionary itself is written in write_page_dicts().
                let page_id = ObjId(self.next_obj_num, 0);
                self.next_obj_num += 1;

                self.page_records.push(PageRecord {
                    obj_id: page_id,
                    content_ids: vec![content_id],
                    width: page.width,
                    height: page.height,
                    used_fonts: page.used_fonts,
                    used_truetype_fonts: page.used_truetype_fonts,
                    used_images: page.used_images,
                    fields: field_records,
                });
            }
            Some(idx) => {
                // Overlay: append content stream to existing page record.
                let record = &mut self.page_records[idx];
                record.content_ids.push(content_id);
                record.used_fonts.extend(page.used_fonts);
                record.used_truetype_fonts.extend(page.used_truetype_fonts);
                record.used_images.extend(page.used_images);
                // Overlays don't support adding new fields; field_records is empty for overlays.
            }
        }

        Ok(())
    }

    /// Build the font resource dictionary for a page.
    fn build_font_dict(&self, used_fonts: &[BuiltinFont], used_truetype: &[usize]) -> PdfObject {
        let mut entries: Vec<(String, PdfObject)> = used_fonts
            .iter()
            .map(|f| {
                (
                    f.pdf_name().to_string(),
                    PdfObject::Reference(self.font_obj_ids[f]),
                )
            })
            .collect();

        for &idx in used_truetype {
            let name = self.truetype_fonts[idx].pdf_name.clone();
            let type0_id = self.truetype_font_obj_ids[&idx].type0;
            entries.push((name, PdfObject::Reference(type0_id)));
        }

        PdfObject::Dictionary(entries)
    }

    /// Build the resource dictionary for a page.
    fn build_resource_dict(
        &self,
        used_fonts: &[BuiltinFont],
        used_truetype: &[usize],
        used_images: &[usize],
    ) -> PdfObject {
        let font_dict = self.build_font_dict(used_fonts, used_truetype);

        let xobject_entries: Vec<(String, PdfObject)> = used_images
            .iter()
            .filter_map(|idx| {
                self.image_obj_ids
                    .get(idx)
                    .map(|ids| (ids.pdf_name.clone(), PdfObject::Reference(ids.xobject)))
            })
            .collect();

        let mut resource_entries: Vec<(String, PdfObject)> = vec![("Font".to_string(), font_dict)];
        if !xobject_entries.is_empty() {
            resource_entries.push((
                "XObject".to_string(),
                PdfObject::Dictionary(xobject_entries),
            ));
        }

        PdfObject::Dictionary(resource_entries)
    }

    /// Build the `/Contents` entry: single reference for one stream, array for multiple.
    fn build_contents(content_ids: &[ObjId]) -> PdfObject {
        if content_ids.len() == 1 {
            PdfObject::Reference(content_ids[0])
        } else {
            PdfObject::array(
                content_ids
                    .iter()
                    .map(|id| PdfObject::Reference(*id))
                    .collect(),
            )
        }
    }

    /// Write widget annotation objects for all form fields in a page.
    fn write_widget_annotations(&mut self, page_idx: usize) -> io::Result<()> {
        let page_obj_id = self.page_records[page_idx].obj_id;
        let field_ids: Vec<(String, Rect, ObjId)> = self.page_records[page_idx]
            .fields
            .iter()
            .map(|f| (f.name.clone(), f.rect, f.obj_id))
            .collect();

        for (name, rect, obj_id) in field_ids {
            // Widget annotation: /Rect is [x_ll y_ll x_ur y_ur] in PDF coordinates
            let rect_array = PdfObject::array(vec![
                PdfObject::Real(rect.x),
                PdfObject::Real(rect.y),
                PdfObject::Real(rect.x + rect.width),
                PdfObject::Real(rect.y + rect.height),
            ]);
            let widget = PdfObject::dict(vec![
                ("Type", PdfObject::name("Annot")),
                ("Subtype", PdfObject::name("Widget")),
                ("FT", PdfObject::name("Tx")),
                ("T", PdfObject::literal_string(&name)),
                ("Rect", rect_array),
                ("P", PdfObject::Reference(page_obj_id)),
                ("F", PdfObject::Integer(4)), // Print flag
            ]);
            self.writer.write_object(obj_id, &widget)?;
        }
        Ok(())
    }

    /// Write page dictionaries for all pages. Called from `end_document()`
    /// after all content streams (including overlays) have been written.
    fn write_page_dicts(&mut self) -> io::Result<()> {
        for i in 0..self.page_records.len() {
            self.write_widget_annotations(i)?;

            // Copy out page data to release the borrow before writing
            let obj_id = self.page_records[i].obj_id;
            let content_ids: Vec<ObjId> =
                self.page_records[i].content_ids.iter().copied().collect();
            let width = self.page_records[i].width;
            let height = self.page_records[i].height;
            let used_fonts: Vec<BuiltinFont> =
                self.page_records[i].used_fonts.iter().copied().collect();
            let used_truetype: Vec<usize> = self.page_records[i]
                .used_truetype_fonts
                .iter()
                .copied()
                .collect();
            let used_images: Vec<usize> =
                self.page_records[i].used_images.iter().copied().collect();
            let annot_ids: Vec<ObjId> = self.page_records[i]
                .fields
                .iter()
                .map(|f| f.obj_id)
                .collect();

            let resources = self.build_resource_dict(&used_fonts, &used_truetype, &used_images);
            let contents = Self::build_contents(&content_ids);

            let mut page_entries = vec![
                ("Type", PdfObject::name("Page")),
                ("Parent", PdfObject::Reference(PAGES_OBJ)),
                (
                    "MediaBox",
                    PdfObject::array(vec![
                        PdfObject::Integer(0),
                        PdfObject::Integer(0),
                        PdfObject::Real(width),
                        PdfObject::Real(height),
                    ]),
                ),
                ("Contents", contents),
                ("Resources", resources),
            ];

            if !annot_ids.is_empty() {
                let annots = PdfObject::array(
                    annot_ids
                        .iter()
                        .map(|id| PdfObject::Reference(*id))
                        .collect(),
                );
                page_entries.push(("Annots", annots));
            }

            let page_dict = PdfObject::dict(page_entries);
            self.writer.write_object(obj_id, &page_dict)?;
        }
        Ok(())
    }

    /// Collect all form field ObjIds across all pages.
    fn collect_all_field_ids(&self) -> Vec<ObjId> {
        self.page_records
            .iter()
            .flat_map(|r| r.fields.iter().map(|f| f.obj_id))
            .collect()
    }

    /// Write the /AcroForm dictionary in the catalog if any fields exist.
    /// Returns the ObjId of the AcroForm dict, or None if no fields.
    fn write_acroform(&mut self) -> io::Result<Option<ObjId>> {
        let all_field_ids = self.collect_all_field_ids();
        if all_field_ids.is_empty() {
            return Ok(None);
        }

        let fields_array = PdfObject::array(
            all_field_ids
                .iter()
                .map(|id| PdfObject::Reference(*id))
                .collect(),
        );

        let acroform_id = ObjId(self.next_obj_num, 0);
        self.next_obj_num += 1;

        let acroform = PdfObject::dict(vec![
            ("Fields", fields_array),
            ("NeedAppearances", PdfObject::Boolean(true)),
            // Default appearance: Helvetica 12pt black
            ("DA", PdfObject::literal_string("/Helv 12 Tf 0 g")),
        ]);
        self.writer.write_object(acroform_id, &acroform)?;
        Ok(Some(acroform_id))
    }

    /// Write all TrueType font objects. Called during
    /// end_document, after all pages have been written.
    fn write_truetype_fonts(&mut self) -> io::Result<()> {
        let indices: Vec<usize> = self.truetype_font_obj_ids.keys().copied().collect();

        for idx in indices {
            let obj_ids_type0 = self.truetype_font_obj_ids[&idx].type0;
            let obj_ids_cid = self.truetype_font_obj_ids[&idx].cid_font;
            let obj_ids_desc = self.truetype_font_obj_ids[&idx].descriptor;
            let obj_ids_file = self.truetype_font_obj_ids[&idx].font_file;
            let obj_ids_tounicode = self.truetype_font_obj_ids[&idx].tounicode;

            let font = &self.truetype_fonts[idx];

            // 1. FontFile2 stream (raw .ttf data)
            let original_len = font.font_data.len() as i64;
            let font_file_stream = self.make_stream(
                vec![("Length1", PdfObject::Integer(original_len))],
                font.font_data.clone(),
            );
            self.writer.write_object(obj_ids_file, &font_file_stream)?;

            // 2. FontDescriptor (values scaled to PDF units: 1/1000)
            let descriptor = PdfObject::dict(vec![
                ("Type", PdfObject::name("FontDescriptor")),
                ("FontName", PdfObject::name(&font.postscript_name)),
                ("Flags", PdfObject::Integer(font.flags as i64)),
                (
                    "FontBBox",
                    PdfObject::array(vec![
                        PdfObject::Integer(font.scale_to_pdf(font.bbox[0])),
                        PdfObject::Integer(font.scale_to_pdf(font.bbox[1])),
                        PdfObject::Integer(font.scale_to_pdf(font.bbox[2])),
                        PdfObject::Integer(font.scale_to_pdf(font.bbox[3])),
                    ]),
                ),
                ("ItalicAngle", PdfObject::Real(font.italic_angle)),
                ("Ascent", PdfObject::Integer(font.scale_to_pdf(font.ascent))),
                (
                    "Descent",
                    PdfObject::Integer(font.scale_to_pdf(font.descent)),
                ),
                (
                    "CapHeight",
                    PdfObject::Integer(font.scale_to_pdf(font.cap_height)),
                ),
                ("StemV", PdfObject::Integer(font.scale_to_pdf(font.stem_v))),
                ("FontFile2", PdfObject::Reference(obj_ids_file)),
            ]);
            self.writer.write_object(obj_ids_desc, &descriptor)?;

            // 3. CIDFontType2
            let w_array = font.build_w_array();
            let cid_font = PdfObject::dict(vec![
                ("Type", PdfObject::name("Font")),
                ("Subtype", PdfObject::name("CIDFontType2")),
                ("BaseFont", PdfObject::name(&font.postscript_name)),
                (
                    "CIDSystemInfo",
                    PdfObject::dict(vec![
                        ("Registry", PdfObject::literal_string("Adobe")),
                        ("Ordering", PdfObject::literal_string("Identity")),
                        ("Supplement", PdfObject::Integer(0)),
                    ]),
                ),
                ("FontDescriptor", PdfObject::Reference(obj_ids_desc)),
                ("DW", PdfObject::Integer(font.default_width_pdf())),
                ("W", PdfObject::Array(w_array)),
            ]);
            self.writer.write_object(obj_ids_cid, &cid_font)?;

            // 4. ToUnicode CMap stream
            let tounicode_data = font.build_tounicode_cmap();
            let tounicode = self.make_stream(vec![], tounicode_data);
            self.writer.write_object(obj_ids_tounicode, &tounicode)?;

            // 5. Type0 font (top-level)
            let type0 = PdfObject::dict(vec![
                ("Type", PdfObject::name("Font")),
                ("Subtype", PdfObject::name("Type0")),
                ("BaseFont", PdfObject::name(&font.postscript_name)),
                ("Encoding", PdfObject::name("Identity-H")),
                (
                    "DescendantFonts",
                    PdfObject::array(vec![PdfObject::Reference(obj_ids_cid)]),
                ),
                ("ToUnicode", PdfObject::Reference(obj_ids_tounicode)),
            ]);
            self.writer.write_object(obj_ids_type0, &type0)?;
        }

        Ok(())
    }

    /// Finish the document. Writes page dictionaries, the catalog, pages tree,
    /// info dictionary, xref table, and trailer.
    /// Consumes self -- no further operations are possible.
    pub fn end_document(mut self) -> io::Result<W> {
        // Auto-close any open page
        if self.current_page.is_some() {
            self.end_page()?;
        }

        // Write page dictionaries (deferred so overlays can be accumulated first)
        self.write_page_dicts()?;

        // Write TrueType font objects (deferred until now)
        self.write_truetype_fonts()?;

        // Write AcroForm if any form fields exist
        let acroform_id = self.write_acroform()?;

        // Write info dictionary if any entries exist
        let info_id = if !self.info.is_empty() {
            let id = ObjId(self.next_obj_num, 0);
            self.next_obj_num += 1;
            let entries: Vec<(&str, PdfObject)> = self
                .info
                .iter()
                .map(|(k, v)| (k.as_str(), PdfObject::literal_string(v)))
                .collect();
            let info_obj = PdfObject::dict(entries);
            self.writer.write_object(id, &info_obj)?;
            Some(id)
        } else {
            None
        };

        // Write pages tree (obj 2)
        let kids: Vec<PdfObject> = self
            .page_records
            .iter()
            .map(|r| PdfObject::Reference(r.obj_id))
            .collect();
        let page_count = self.page_records.len() as i64;
        let pages = PdfObject::dict(vec![
            ("Type", PdfObject::name("Pages")),
            ("Kids", PdfObject::Array(kids)),
            ("Count", PdfObject::Integer(page_count)),
        ]);
        self.writer.write_object(PAGES_OBJ, &pages)?;

        // Write catalog (obj 1)
        let mut catalog_entries = vec![
            ("Type", PdfObject::name("Catalog")),
            ("Pages", PdfObject::Reference(PAGES_OBJ)),
        ];
        if let Some(acroform) = acroform_id {
            catalog_entries.push(("AcroForm", PdfObject::Reference(acroform)));
        }
        let catalog = PdfObject::dict(catalog_entries);
        self.writer.write_object(CATALOG_OBJ, &catalog)?;

        // Write xref and trailer
        self.writer.write_xref_and_trailer(CATALOG_OBJ, info_id)?;

        Ok(self.writer.into_inner())
    }
}

/// Decompose an arc into cubic Bezier segments (up to one per 90°).
///
/// Given center `(cx, cy)`, `radius`, and start/end angles in radians
/// (standard math convention: 0 = right, CCW positive), returns a sequence of
/// `(x1, y1, x2, y2, x3, y3)` control-point tuples in user space.
///
/// The magic constant `k = 4/3 * tan(α/4)` approximates the arc to within
/// 0.027% for a 90° segment (see PDF 32000-1:2008, §8.5.2).
fn arc_bezier_segments(
    cx: f64,
    cy: f64,
    radius: f64,
    start_rad: f64,
    end_rad: f64,
) -> Vec<(f64, f64, f64, f64, f64, f64)> {
    const MAX_SEGMENT: f64 = std::f64::consts::FRAC_PI_2; // 90°

    let mut segments = Vec::new();
    let total = end_rad - start_rad;
    let n = (total.abs() / MAX_SEGMENT).ceil().max(1.0) as u32;
    let step = total / n as f64;

    for i in 0..n {
        let a = start_rad + i as f64 * step;
        let b = a + step;
        let k = 4.0 / 3.0 * ((b - a) / 4.0).tan();

        let (cos_a, sin_a) = (a.cos(), a.sin());
        let (cos_b, sin_b) = (b.cos(), b.sin());

        let x1 = cx + radius * (cos_a - k * sin_a);
        let y1 = cy + radius * (sin_a + k * cos_a);
        let x2 = cx + radius * (cos_b + k * sin_b);
        let y2 = cy + radius * (sin_b - k * cos_b);
        let x3 = cx + radius * cos_b;
        let y3 = cy + radius * sin_b;

        segments.push((x1, y1, x2, y2, x3, y3));
    }
    segments
}

/// Format a coordinate value for PDF content streams.
pub(crate) fn format_coord(v: f64) -> String {
    if v == v.floor() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.4}", v);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}
