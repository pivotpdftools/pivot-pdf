use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use crate::objects::{ObjId, PdfObject};
use crate::writer::PdfWriter;

const CATALOG_OBJ: ObjId = ObjId(1, 0);
const PAGES_OBJ: ObjId = ObjId(2, 0);
const FONT_OBJ: ObjId = ObjId(3, 0);
const FIRST_PAGE_OBJ_NUM: u32 = 4;

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
        pdf_writer.write_object(FONT_OBJ, &font)?;

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
                    PdfObject::dict(vec![(
                        "F1",
                        PdfObject::Reference(
                            FONT_OBJ,
                        ),
                    )]),
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
fn format_coord(v: f64) -> String {
    if v == v.floor() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.4}", v);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_empty_document() {
        let mut doc =
            PdfDocument::new(Vec::<u8>::new())
                .unwrap();
        doc.begin_page(612.0, 792.0);
        doc.end_page().unwrap();
        let bytes = doc.end_document().unwrap();
        let output =
            String::from_utf8_lossy(&bytes);
        assert!(output.contains("%PDF-1.7"));
        assert!(output.contains("%%EOF"));
    }

    #[test]
    fn set_info_appears_in_output() {
        let mut doc =
            PdfDocument::new(Vec::<u8>::new())
                .unwrap();
        doc.set_info("Creator", "rust-pdf");
        doc.set_info("Title", "Test Doc");
        doc.begin_page(612.0, 792.0);
        doc.end_page().unwrap();
        let bytes = doc.end_document().unwrap();
        let output =
            String::from_utf8_lossy(&bytes);
        assert!(output.contains("(rust-pdf)"));
        assert!(output.contains("(Test Doc)"));
    }

    #[test]
    fn place_text_in_content_stream() {
        let mut doc =
            PdfDocument::new(Vec::<u8>::new())
                .unwrap();
        doc.begin_page(612.0, 792.0);
        doc.place_text("Hello", 20.0, 20.0);
        doc.end_page().unwrap();
        let bytes = doc.end_document().unwrap();
        let output =
            String::from_utf8_lossy(&bytes);
        assert!(output.contains("(Hello) Tj"));
        assert!(output.contains("/F1 12 Tf"));
        assert!(output.contains("20 20 Td"));
    }

    #[test]
    fn end_page_flushes_to_writer() {
        let mut doc =
            PdfDocument::new(Vec::<u8>::new())
                .unwrap();
        let size_after_header =
            doc.writer.current_offset();

        doc.begin_page(612.0, 792.0);
        doc.place_text("Hello", 20.0, 20.0);

        // Before end_page, size should be same
        // (page data in memory, not flushed).
        assert_eq!(
            doc.writer.current_offset(),
            size_after_header,
        );

        doc.end_page().unwrap();

        // After end_page, more bytes written.
        assert!(
            doc.writer.current_offset()
                > size_after_header,
        );
    }

    #[test]
    fn auto_close_page_on_begin_page() {
        let mut doc =
            PdfDocument::new(Vec::<u8>::new())
                .unwrap();
        doc.begin_page(612.0, 792.0);
        doc.place_text("Page 1", 20.0, 20.0);
        // begin_page again without end_page.
        doc.begin_page(612.0, 792.0);
        doc.place_text("Page 2", 20.0, 20.0);
        doc.end_page().unwrap();
        let bytes = doc.end_document().unwrap();
        let output =
            String::from_utf8_lossy(&bytes);
        assert!(output.contains("/Count 2"));
    }

    #[test]
    fn auto_close_page_on_end_document() {
        let mut doc =
            PdfDocument::new(Vec::<u8>::new())
                .unwrap();
        doc.begin_page(612.0, 792.0);
        doc.place_text("Hello", 20.0, 20.0);
        // end_document without end_page.
        let bytes = doc.end_document().unwrap();
        let output =
            String::from_utf8_lossy(&bytes);
        assert!(output.contains("/Count 1"));
        assert!(output.contains("(Hello) Tj"));
    }

    #[test]
    fn format_coord_values() {
        assert_eq!(format_coord(20.0), "20");
        assert_eq!(format_coord(612.0), "612");
        assert_eq!(format_coord(0.0), "0");
        assert_eq!(format_coord(12.5), "12.5");
    }
}
