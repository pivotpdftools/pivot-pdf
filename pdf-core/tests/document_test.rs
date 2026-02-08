use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

use pdf_core::PdfDocument;

#[test]
fn create_empty_document() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("%PDF-1.7"));
    assert!(output.contains("%%EOF"));
}

#[test]
fn set_info_appears_in_output() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.set_info("Creator", "rust-pdf");
    doc.set_info("Title", "Test Doc");
    doc.begin_page(612.0, 792.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("(rust-pdf)"));
    assert!(output.contains("(Test Doc)"));
}

#[test]
fn place_text_in_content_stream() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello", 20.0, 20.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("(Hello) Tj"));
    assert!(output.contains("/F1 12 Tf"));
    assert!(output.contains("20 20 Td"));
}

/// Verifies that end_page flushes page data to the writer
/// incrementally, rather than buffering everything until
/// end_document.
#[test]
fn end_page_flushes_to_writer() {
    struct TrackingWriter {
        byte_count: Rc<RefCell<usize>>,
        inner: Vec<u8>,
    }

    impl Write for TrackingWriter {
        fn write(
            &mut self,
            buf: &[u8],
        ) -> io::Result<usize> {
            let n = self.inner.write(buf)?;
            *self.byte_count.borrow_mut() += n;
            Ok(n)
        }
        fn flush(&mut self) -> io::Result<()> {
            self.inner.flush()
        }
    }

    let counter = Rc::new(RefCell::new(0usize));
    let writer = TrackingWriter {
        byte_count: counter.clone(),
        inner: Vec::new(),
    };

    let mut doc = PdfDocument::new(writer).unwrap();
    let after_init = *counter.borrow();

    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello", 20.0, 20.0);

    // Page data is in memory, not yet written.
    assert_eq!(*counter.borrow(), after_init);

    doc.end_page().unwrap();

    // After end_page, page data has been flushed.
    assert!(*counter.borrow() > after_init);
}

#[test]
fn auto_close_page_on_begin_page() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Page 1", 20.0, 20.0);
    // begin_page again without end_page.
    doc.begin_page(612.0, 792.0);
    doc.place_text("Page 2", 20.0, 20.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("/Count 2"));
}

#[test]
fn auto_close_page_on_end_document() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello", 20.0, 20.0);
    // end_document without end_page.
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("/Count 1"));
    assert!(output.contains("(Hello) Tj"));
}

/// Tests coordinate formatting through the public API.
/// Integer-valued coordinates should appear without decimals,
/// fractional values should retain necessary precision.
#[test]
fn coord_formatting_in_content_stream() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("test", 20.0, 612.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // Integer coords should not have decimal points.
    assert!(output.contains("20 612 Td"));

    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("test", 12.5, 0.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // Fractional coord should retain precision.
    assert!(output.contains("12.5 0 Td"));
}
