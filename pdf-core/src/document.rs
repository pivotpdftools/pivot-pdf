use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::fonts::{BuiltinFont, FontRef, TrueTypeFontId};
use crate::graphics::Color;
use crate::images::{self, ImageData, ImageFit, ImageFormat, ImageId};
use crate::objects::{ObjId, PdfObject};
use crate::tables::{Row, Table, TableCursor};
use crate::textflow::{FitResult, Rect, TextFlow, TextStyle};
use crate::truetype::TrueTypeFont;
use crate::writer::PdfWriter;

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
    page_obj_ids: Vec<ObjId>,
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
}

struct PageBuilder {
    width: f64,
    height: f64,
    content_ops: Vec<u8>,
    used_fonts: BTreeSet<BuiltinFont>,
    used_truetype_fonts: BTreeSet<usize>,
    used_images: BTreeSet<usize>,
}

impl PdfDocument<BufWriter<File>> {
    /// Create a new PDF document that writes to a file.
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::create(path)?;
        Self::new(BufWriter::new(file))
    }
}

impl<W: Write> PdfDocument<W> {
    /// Create a new PDF document that writes to the given writer.
    /// Writes the PDF header immediately.
    pub fn new(writer: W) -> io::Result<Self> {
        let mut pdf_writer = PdfWriter::new(writer);
        pdf_writer.write_header()?;

        Ok(PdfDocument {
            writer: pdf_writer,
            info: Vec::new(),
            page_obj_ids: Vec::new(),
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
        });
        self
    }

    /// Place text at position (x, y) using default 12pt Helvetica.
    /// Coordinates use PDF's default bottom-left origin.
    pub fn place_text(&mut self, text: &str, x: f64, y: f64) -> &mut Self {
        let page = self
            .current_page
            .as_mut()
            .expect("place_text called with no open page");
        page.used_fonts.insert(BuiltinFont::Helvetica);
        let escaped = crate::writer::escape_pdf_string(text);
        let ops = format!(
            "BT\n/F1 12 Tf\n{} {} Td\n({}) Tj\nET\n",
            format_coord(x),
            format_coord(y),
            escaped,
        );
        page.content_ops.extend_from_slice(ops.as_bytes());
        self
    }

    /// Place text at position (x, y) with the given style.
    /// Coordinates use PDF's default bottom-left origin.
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
            format_coord(y),
            text_op,
        );
        page.content_ops.extend_from_slice(ops.as_bytes());
        self
    }

    /// Fit a TextFlow into a bounding rectangle on the current
    /// page. The flow's cursor advances so subsequent calls
    /// continue where it left off (for multi-page flow).
    pub fn fit_textflow(&mut self, flow: &mut TextFlow, rect: &Rect) -> io::Result<FitResult> {
        let (ops, result, used_fonts) = flow.generate_content_ops(rect, &mut self.truetype_fonts);

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
        let (ops, result, used_fonts) =
            table.generate_row_ops(row, cursor, &mut self.truetype_fonts);

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
    pub fn place_image(
        &mut self,
        image: &ImageId,
        rect: &Rect,
        fit: ImageFit,
    ) -> &mut Self {
        let idx = image.0;
        let img = &self.images[idx];
        let page_height = self
            .current_page
            .as_ref()
            .expect("place_image called with no open page")
            .height;

        let placement =
            images::calculate_placement(img.width, img.height, rect, fit, page_height);

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

        self.image_obj_ids.insert(idx, ImageObjIds {
            xobject,
            smask,
            pdf_name,
        });
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
        if let (Some(smask_obj_id), Some(smask_data)) =
            (smask_id, img.smask_data.as_ref())
        {
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
            ImageFormat::Png => {
                self.make_stream(entries, img.data.clone())
            }
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
    pub fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        let page = self
            .current_page
            .as_mut()
            .expect("move_to called with no open page");
        let ops = format!("{} {} m\n", format_coord(x), format_coord(y));
        page.content_ops.extend_from_slice(ops.as_bytes());
        self
    }

    /// Draw a line from the current point (PDF `l` operator).
    pub fn line_to(&mut self, x: f64, y: f64) -> &mut Self {
        let page = self
            .current_page
            .as_mut()
            .expect("line_to called with no open page");
        let ops = format!("{} {} l\n", format_coord(x), format_coord(y));
        page.content_ops.extend_from_slice(ops.as_bytes());
        self
    }

    /// Append a rectangle to the path (PDF `re` operator).
    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) -> &mut Self {
        let page = self
            .current_page
            .as_mut()
            .expect("rect called with no open page");
        let ops = format!(
            "{} {} {} {} re\n",
            format_coord(x),
            format_coord(y),
            format_coord(width),
            format_coord(height),
        );
        page.content_ops.extend_from_slice(ops.as_bytes());
        self
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

    /// End the current page. Writes page objects to the
    /// writer and frees page content from memory.
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
        let page_id = ObjId(self.next_obj_num, 0);
        self.next_obj_num += 1;

        // Write content stream
        let content_stream = self.make_stream(vec![], page.content_ops);
        self.writer.write_object(content_id, &content_stream)?;

        // Build font resource entries for builtin fonts
        let font_entries: Vec<(&str, PdfObject)> = page
            .used_fonts
            .iter()
            .map(|f| (f.pdf_name(), PdfObject::Reference(self.font_obj_ids[f])))
            .collect();

        // Add TrueType font entries (reference the Type0 obj)
        // We need owned strings for the PDF names
        let tt_entries: Vec<(String, PdfObject)> = page
            .used_truetype_fonts
            .iter()
            .map(|&idx| {
                let obj_ids = &self.truetype_font_obj_ids[&idx];
                let name = self.truetype_fonts[idx].pdf_name.clone();
                (name, PdfObject::Reference(obj_ids.type0))
            })
            .collect();

        // Combine into a single dict. Since PdfObject::dict takes
        // &str, we need to build the Dictionary variant directly.
        let mut all_font_entries: Vec<(String, PdfObject)> = font_entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        all_font_entries.extend(tt_entries);

        let font_dict = PdfObject::Dictionary(all_font_entries);

        // Build XObject dict for images used on this page
        let xobject_entries: Vec<(String, PdfObject)> = used_images
            .iter()
            .filter_map(|idx| {
                self.image_obj_ids.get(idx).map(|ids| {
                    (ids.pdf_name.clone(), PdfObject::Reference(ids.xobject))
                })
            })
            .collect();

        // Build Resources dict with Font and optional XObject
        let mut resource_entries: Vec<(String, PdfObject)> =
            vec![("Font".to_string(), font_dict)];
        if !xobject_entries.is_empty() {
            resource_entries.push((
                "XObject".to_string(),
                PdfObject::Dictionary(xobject_entries),
            ));
        }

        // Write page dictionary
        let page_dict = PdfObject::dict(vec![
            ("Type", PdfObject::name("Page")),
            ("Parent", PdfObject::Reference(PAGES_OBJ)),
            (
                "MediaBox",
                PdfObject::array(vec![
                    PdfObject::Integer(0),
                    PdfObject::Integer(0),
                    PdfObject::Real(page.width),
                    PdfObject::Real(page.height),
                ]),
            ),
            ("Contents", PdfObject::Reference(content_id)),
            ("Resources", PdfObject::Dictionary(resource_entries)),
        ]);
        self.writer.write_object(page_id, &page_dict)?;

        self.page_obj_ids.push(page_id);
        Ok(())
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

    /// Finish the document. Writes the catalog, pages tree,
    /// info dictionary, xref table, and trailer.
    /// Consumes self -- no further operations are possible.
    pub fn end_document(mut self) -> io::Result<W> {
        // Auto-close any open page
        if self.current_page.is_some() {
            self.end_page()?;
        }

        // Write TrueType font objects (deferred until now)
        self.write_truetype_fonts()?;

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
            .page_obj_ids
            .iter()
            .map(|id| PdfObject::Reference(*id))
            .collect();
        let page_count = self.page_obj_ids.len() as i64;
        let pages = PdfObject::dict(vec![
            ("Type", PdfObject::name("Pages")),
            ("Kids", PdfObject::Array(kids)),
            ("Count", PdfObject::Integer(page_count)),
        ]);
        self.writer.write_object(PAGES_OBJ, &pages)?;

        // Write catalog (obj 1)
        let catalog = PdfObject::dict(vec![
            ("Type", PdfObject::name("Catalog")),
            ("Pages", PdfObject::Reference(PAGES_OBJ)),
        ]);
        self.writer.write_object(CATALOG_OBJ, &catalog)?;

        // Write xref and trailer
        self.writer.write_xref_and_trailer(CATALOG_OBJ, info_id)?;

        Ok(self.writer.into_inner())
    }
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
