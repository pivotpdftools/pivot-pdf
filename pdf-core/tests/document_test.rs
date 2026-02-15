use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

use pdf_core::{PdfDocument, TextStyle};

#[test]
fn create_empty_document() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("%PDF-1.7"));
    assert!(output.contains("%%EOF"));
}

#[test]
fn set_info_appears_in_output() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
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
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
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
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
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
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
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
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello", 20.0, 20.0);
    // end_document without end_page.
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("/Count 1"));
    assert!(output.contains("(Hello) Tj"));
}

#[test]
fn compressed_pdf_is_smaller_than_uncompressed() {
    let make_pdf = |compress: bool| -> Vec<u8> {
        let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
        doc.set_compression(compress);
        for i in 0..10 {
            doc.begin_page(612.0, 792.0);
            for y in (0..20).rev() {
                doc.place_text(
                    &format!("Page {} line {} — repetitive content for compression", i, y),
                    20.0,
                    700.0 - (y as f64 * 30.0),
                );
            }
            doc.end_page().unwrap();
        }
        doc.end_document().unwrap()
    };

    let uncompressed = make_pdf(false);
    let compressed = make_pdf(true);
    assert!(
        compressed.len() < uncompressed.len(),
        "compressed ({}) should be smaller than uncompressed ({})",
        compressed.len(),
        uncompressed.len(),
    );
}

#[test]
fn compressed_pdf_contains_flatedecode_filter() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.set_compression(true);
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello", 20.0, 20.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(
        output.contains("/Filter /FlateDecode"),
        "compressed output should contain FlateDecode filter",
    );
}

#[test]
fn compressed_truetype_font_has_filter_and_length1() {
    const DEJAVU_SANS: &[u8] = include_bytes!("fixtures/DejaVuSans.ttf");

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.set_compression(true);
    let font_ref = doc.load_font_bytes(DEJAVU_SANS.to_vec()).unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_text_styled(
        "Test",
        72.0,
        720.0,
        &TextStyle {
            font: font_ref,
            font_size: 12.0,
        },
    );
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    assert!(
        output.contains("/Length1"),
        "FontFile2 should have Length1 entry"
    );
    assert!(
        output.contains("/Filter /FlateDecode"),
        "compressed streams should have FlateDecode filter",
    );
}

#[test]
fn uncompressed_pdf_has_no_flatedecode_filter() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello", 20.0, 20.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(
        !output.contains("FlateDecode"),
        "uncompressed output should not contain FlateDecode",
    );
}

/// Tests coordinate formatting through the public API.
/// Integer-valued coordinates should appear without decimals,
/// fractional values should retain necessary precision.
#[test]
fn coord_formatting_in_content_stream() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("test", 20.0, 612.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // Integer coords should not have decimal points.
    assert!(output.contains("20 612 Td"));

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("test", 12.5, 0.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // Fractional coord should retain precision.
    assert!(output.contains("12.5 0 Td"));
}
