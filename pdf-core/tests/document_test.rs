use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

use pivot_pdf::{
    DocumentOptions, FitResult, ImageFit, Origin, PdfDocument, Rect, TextFlow, TextStyle,
};

#[test]
fn create_empty_document() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("%PDF-1.7"));
    assert!(output.contains("%%EOF"));
}

#[test]
fn set_info_appears_in_output() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
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
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
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

    let mut doc = PdfDocument::new(writer, DocumentOptions::default()).unwrap();
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
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
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
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
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
        let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
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
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
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

    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
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
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
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

#[test]
fn document_options_default_is_bottom_left() {
    let opts = DocumentOptions::default();
    assert_eq!(opts.origin, Origin::BottomLeft);
}

#[test]
fn top_left_origin_transforms_place_text() {
    // Page height 792. Placing text at y=72 with TopLeft should
    // emit y=720 in PDF space (792 - 72 = 720).
    let opts = DocumentOptions {
        origin: Origin::TopLeft,
    };
    let mut doc = PdfDocument::new(Vec::<u8>::new(), opts).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello", 72.0, 72.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(
        output.contains("72 720 Td"),
        "expected '72 720 Td' in output, got: {}",
        output
    );
}

#[test]
fn bottom_left_origin_no_transform() {
    // With BottomLeft (default), y is unchanged.
    let opts = DocumentOptions::default();
    let mut doc = PdfDocument::new(Vec::<u8>::new(), opts).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello", 72.0, 720.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(
        output.contains("72 720 Td"),
        "expected '72 720 Td' in output, got: {}",
        output
    );
}

/// Tests coordinate formatting through the public API.
/// Integer-valued coordinates should appear without decimals,
/// fractional values should retain necessary precision.
#[test]
fn coord_formatting_in_content_stream() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("test", 20.0, 612.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // Integer coords should not have decimal points.
    assert!(output.contains("20 612 Td"));

    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("test", 12.5, 0.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // Fractional coord should retain precision.
    assert!(output.contains("12.5 0 Td"));
}

#[test]
fn top_left_origin_transforms_move_to_and_line_to() {
    // Page height 792. move_to/line_to at y=72 → y=720 in PDF space.
    let opts = DocumentOptions {
        origin: Origin::TopLeft,
    };
    let mut doc = PdfDocument::new(Vec::<u8>::new(), opts).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.move_to(100.0, 72.0);
    doc.line_to(200.0, 72.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("100 720 m\n"), "move_to y should transform");
    assert!(output.contains("200 720 l\n"), "line_to y should transform");
}

#[test]
fn top_left_origin_transforms_rect() {
    // Page height 792. rect at (72, 72, 100, 50) with TopLeft:
    // PDF bottom = 792 - 72 - 50 = 670.
    let opts = DocumentOptions {
        origin: Origin::TopLeft,
    };
    let mut doc = PdfDocument::new(Vec::<u8>::new(), opts).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.rect(72.0, 72.0, 100.0, 50.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(
        output.contains("72 670 100 50 re\n"),
        "rect y should transform"
    );
}

#[test]
fn top_left_origin_transforms_fit_textflow() {
    // Page height 792. Textflow rect at y=72 (from top) → top edge = 720 in PDF.
    let opts = DocumentOptions {
        origin: Origin::TopLeft,
    };
    let mut doc = PdfDocument::new(Vec::<u8>::new(), opts).unwrap();
    doc.begin_page(612.0, 792.0);
    let rect = Rect {
        x: 72.0,
        y: 72.0,
        width: 468.0,
        height: 100.0,
    };
    let mut tf = TextFlow::new();
    tf.add_text("Hello", &TextStyle::default());
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert_eq!(result, FitResult::Stop);
    // Baseline is at top-of-rect minus font-size: 720 - 12 = 708
    assert!(output.contains("72 708 Td"), "textflow y should transform");
}

#[test]
fn top_left_origin_transforms_place_image() {
    // Page height 792. Image rect at (72, 72, 200, 150) with TopLeft:
    // PDF bottom = 792 - 72 - 150 = 570. Image should be placed at y≥570.
    const TEST_PNG: &[u8] = include_bytes!("fixtures/test.png");
    let opts = DocumentOptions {
        origin: Origin::TopLeft,
    };
    let mut doc = PdfDocument::new(Vec::<u8>::new(), opts).unwrap();
    let img = doc.load_image_bytes(TEST_PNG.to_vec()).unwrap();
    doc.begin_page(612.0, 792.0);
    let rect = Rect {
        x: 72.0,
        y: 72.0,
        width: 200.0,
        height: 150.0,
    };
    doc.place_image(&img, &rect, ImageFit::Stretch);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // Stretch: cm = [w 0 0 h x y_bottom] → 200 0 0 150 72 570
    assert!(
        output.contains("200 0 0 150 72 570 cm"),
        "image y should transform: {}",
        output.lines().find(|l| l.contains("cm")).unwrap_or("no cm")
    );
}
