use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use crate::objects::{ObjId, PdfObject};
use crate::textflow::{FitResult, Rect, TextFlow};
use crate::writer::PdfWriter;

const CATALOG_OBJ: ObjId = ObjId(1, 0);
const PAGES_OBJ: ObjId = ObjId(2, 0);
const FONT_HELV_OBJ: ObjId = ObjId(3, 0);
const FONT_HELV_BOLD_OBJ: ObjId = ObjId(4, 0);
const FIRST_PAGE_OBJ_NUM: u32 = 5;

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
}

struct PageBuilder {
    width: f64,
    height: f64,
    content_ops: Vec<u8>,
}

impl PdfDocument<BufWriter<File>> {
    /// Create a new PDF document that writes to a file.
    pub fn create<P: AsRef<Path>>(
        path: P,
    ) -> io::Result<Self> {
        let file = File::create(path)?;
        Self::new(BufWriter::new(file))
    }
}

impl<W: Write> PdfDocument<W> {
    /// Create a new PDF document that writes to the given writer.
    /// Writes the PDF header and shared font object immediately.
    pub fn new(writer: W) -> io::Result<Self> {
        let mut pdf_writer = PdfWriter::new(writer);
        pdf_writer.write_header()?;

        // Write shared Helvetica font (obj 3).
        let font = PdfObject::dict(vec![
            ("Type", PdfObject::name("Font")),
            ("Subtype", PdfObject::name("Type1")),
            (
                "BaseFont",
                PdfObject::name("Helvetica"),
            ),
        ]);
        pdf_writer.write_object(FONT_HELV_OBJ, &font)?;

        // Write shared Helvetica-Bold font (obj 4).
        let font_bold = PdfObject::dict(vec![
            ("Type", PdfObject::name("Font")),
            ("Subtype", PdfObject::name("Type1")),
            (
                "BaseFont",
                PdfObject::name("Helvetica-Bold"),
            ),
        ]);
        pdf_writer.write_object(
            FONT_HELV_BOLD_OBJ,
            &font_bold,
        )?;

        Ok(PdfDocument {
            writer: pdf_writer,
            info: Vec::new(),
            page_obj_ids: Vec::new(),
            current_page: None,
            next_obj_num: FIRST_PAGE_OBJ_NUM,
        })
    }

    /// Set a document info entry (e.g. "Creator", "Title").
    pub fn set_info(
        &mut self,
        key: &str,
        value: &str,
    ) -> &mut Self {
        self.info
            .push((key.to_string(), value.to_string()));
        self
    }

    /// Begin a new page with the given dimensions in points.
    /// If a page is currently open, it is automatically closed.
    pub fn begin_page(
        &mut self,
        width: f64,
        height: f64,
    ) -> &mut Self {
        if self.current_page.is_some() {
            // Auto-close previous page. Ignore write
            // errors here; end_page will catch them.
            let _ = self.end_page();
        }
        self.current_page = Some(PageBuilder {
            width,
            height,
            content_ops: Vec::new(),
        });
        self
    }

    /// Place text at position (x, y) using default 12pt Helvetica.
    /// Coordinates use PDF's default bottom-left origin.
    pub fn place_text(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
    ) -> &mut Self {
        let page = self
            .current_page
            .as_mut()
            .expect("place_text called with no open page");
        let escaped =
            crate::writer::escape_pdf_string(text);
        let ops = format!(
            "BT\n/F1 12 Tf\n{} {} Td\n({}) Tj\nET\n",
            format_coord(x),
            format_coord(y),
            escaped,
        );
        page.content_ops.extend_from_slice(
            ops.as_bytes(),
        );
        self
    }

    /// Fit a TextFlow into a bounding rectangle on the current
    /// page. The flow's cursor advances so subsequent calls
    /// continue where it left off (for multi-page flow).
    pub fn fit_textflow(
        &mut self,
        flow: &mut TextFlow,
        rect: &Rect,
    ) -> io::Result<FitResult> {
        let page = self
            .current_page
            .as_mut()
            .expect(
                "fit_textflow called with no open page",
            );
        let (ops, result) =
            flow.generate_content_ops(rect);
        page.content_ops.extend_from_slice(&ops);
        Ok(result)
    }

    /// End the current page. Writes page objects to the
    /// writer and frees page content from memory.
    pub fn end_page(&mut self) -> io::Result<()> {
        let page = self
            .current_page
            .take()
            .expect("end_page called with no open page");

        let content_id =
            ObjId(self.next_obj_num, 0);
        self.next_obj_num += 1;
        let page_id = ObjId(self.next_obj_num, 0);
        self.next_obj_num += 1;

        // Write content stream.
        let content_stream = PdfObject::stream(
            vec![],
            page.content_ops,
        );
        self.writer
            .write_object(content_id, &content_stream)?;

        // Write page dictionary.
        let page_dict = PdfObject::dict(vec![
            ("Type", PdfObject::name("Page")),
            (
                "Parent",
                PdfObject::Reference(PAGES_OBJ),
            ),
            (
                "MediaBox",
                PdfObject::array(vec![
                    PdfObject::Integer(0),
                    PdfObject::Integer(0),
                    PdfObject::Real(page.width),
                    PdfObject::Real(page.height),
                ]),
            ),
            (
                "Contents",
                PdfObject::Reference(content_id),
            ),
            (
                "Resources",
                PdfObject::dict(vec![(
                    "Font",
                    PdfObject::dict(vec![
                        (
                            "F1",
                            PdfObject::Reference(
                                FONT_HELV_OBJ,
                            ),
                        ),
                        (
                            "F2",
                            PdfObject::Reference(
                                FONT_HELV_BOLD_OBJ,
                            ),
                        ),
                    ]),
                )]),
            ),
        ]);
        self.writer
            .write_object(page_id, &page_dict)?;

        self.page_obj_ids.push(page_id);
        Ok(())
    }

    /// Finish the document. Writes the catalog, pages tree,
    /// info dictionary, xref table, and trailer.
    /// Consumes self — no further operations are possible.
    pub fn end_document(
        mut self,
    ) -> io::Result<W> {
        // Auto-close any open page.
        if self.current_page.is_some() {
            self.end_page()?;
        }

        // Write info dictionary if any entries exist.
        let info_id = if !self.info.is_empty() {
            let id = ObjId(self.next_obj_num, 0);
            self.next_obj_num += 1;
            let entries: Vec<(&str, PdfObject)> =
                self.info
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.as_str(),
                            PdfObject::literal_string(v),
                        )
                    })
                    .collect();
            let info_obj = PdfObject::dict(entries);
            self.writer.write_object(id, &info_obj)?;
            Some(id)
        } else {
            None
        };

        // Write pages tree (obj 2).
        let kids: Vec<PdfObject> = self
            .page_obj_ids
            .iter()
            .map(|id| PdfObject::Reference(*id))
            .collect();
        let page_count =
            self.page_obj_ids.len() as i64;
        let pages = PdfObject::dict(vec![
            ("Type", PdfObject::name("Pages")),
            ("Kids", PdfObject::Array(kids)),
            ("Count", PdfObject::Integer(page_count)),
        ]);
        self.writer
            .write_object(PAGES_OBJ, &pages)?;

        // Write catalog (obj 1).
        let catalog = PdfObject::dict(vec![
            ("Type", PdfObject::name("Catalog")),
            (
                "Pages",
                PdfObject::Reference(PAGES_OBJ),
            ),
        ]);
        self.writer
            .write_object(CATALOG_OBJ, &catalog)?;

        // Write xref and trailer.
        self.writer.write_xref_and_trailer(
            CATALOG_OBJ,
            info_id,
        )?;

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
