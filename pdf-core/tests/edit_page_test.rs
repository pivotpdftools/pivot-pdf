use pdf_core::{BuiltinFont, FontRef, ImageFit, PdfDocument, Rect, TextFlow, TextStyle};

// -------------------------------------------------------
// page_count
// -------------------------------------------------------

#[test]
fn page_count_is_zero_before_any_pages() {
    let doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    assert_eq!(doc.page_count(), 0);
}

#[test]
fn page_count_returns_number_of_completed_pages() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    assert_eq!(doc.page_count(), 0);

    doc.begin_page(612.0, 792.0);
    // page_count only counts completed pages
    assert_eq!(doc.page_count(), 0);

    doc.end_page().unwrap();
    assert_eq!(doc.page_count(), 1);

    doc.begin_page(612.0, 792.0);
    doc.end_page().unwrap();
    assert_eq!(doc.page_count(), 2);
}

#[test]
fn page_count_not_incremented_by_open_page() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.end_page().unwrap();
    assert_eq!(doc.page_count(), 1);

    doc.open_page(1).unwrap();
    // open_page edits existing page, not adding a new one
    assert_eq!(doc.page_count(), 1);

    doc.end_page().unwrap();
    assert_eq!(doc.page_count(), 1);
}

// -------------------------------------------------------
// open_page: error cases
// -------------------------------------------------------

#[test]
fn open_page_zero_returns_error() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.end_page().unwrap();

    let result = doc.open_page(0);
    assert!(result.is_err(), "page_num 0 should return error");
}

#[test]
fn open_page_out_of_range_returns_error() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.end_page().unwrap();

    let result = doc.open_page(2);
    assert!(result.is_err(), "page_num 2 with only 1 page should error");
}

#[test]
fn open_page_on_empty_doc_returns_error() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let result = doc.open_page(1);
    assert!(result.is_err());
}

// -------------------------------------------------------
// open_page: overlay content
// -------------------------------------------------------

#[test]
fn open_page_adds_overlay_content_stream() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Main content", 72.0, 700.0);
    doc.end_page().unwrap();

    doc.open_page(1).unwrap();
    doc.place_text("Page 1 of 1", 72.0, 36.0);
    doc.end_page().unwrap();

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    assert!(
        output.contains("(Main content) Tj"),
        "main content should be present"
    );
    assert!(
        output.contains("(Page 1 of 1) Tj"),
        "overlay content should be present"
    );
}

#[test]
fn open_page_contents_is_array_when_overlay_added() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Page body", 72.0, 700.0);
    doc.end_page().unwrap();

    doc.open_page(1).unwrap();
    doc.place_text("Footer", 72.0, 36.0);
    doc.end_page().unwrap();

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    // Two content streams means /Contents should be an array
    assert!(
        output.contains("/Contents ["),
        "two streams should produce /Contents array, got: {}",
        &output[output.find("/Contents").unwrap_or(0)..],
    );
}

#[test]
fn page_without_overlay_has_single_contents_reference() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Solo page", 72.0, 700.0);
    doc.end_page().unwrap();

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    // Single content stream: /Contents should be a direct reference, not an array
    assert!(
        !output.contains("/Contents ["),
        "single stream should not produce /Contents array"
    );
    assert!(output.contains("/Contents "), "should have /Contents entry");
}

// -------------------------------------------------------
// open_page: page dimensions preserved
// -------------------------------------------------------

#[test]
fn open_page_preserves_original_page_dimensions() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    // A5 page (non-letter size to make it detectable)
    doc.begin_page(419.0, 595.0);
    doc.place_text("A5 content", 36.0, 500.0);
    doc.end_page().unwrap();

    doc.open_page(1).unwrap();
    doc.place_text("A5 overlay", 36.0, 36.0);
    doc.end_page().unwrap();

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    assert!(
        output.contains("419"),
        "A5 width (419pt) should appear in MediaBox"
    );
    assert!(
        output.contains("595"),
        "A5 height (595pt) should appear in MediaBox"
    );
}

// -------------------------------------------------------
// open_page: multiple overlays on same page
// -------------------------------------------------------

#[test]
fn multiple_overlays_on_same_page() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Body text", 72.0, 700.0);
    doc.end_page().unwrap();

    doc.open_page(1).unwrap();
    doc.place_text("Overlay one", 72.0, 50.0);
    doc.end_page().unwrap();

    doc.open_page(1).unwrap();
    doc.place_text("Overlay two", 72.0, 36.0);
    doc.end_page().unwrap();

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    assert!(output.contains("(Body text) Tj"));
    assert!(output.contains("(Overlay one) Tj"));
    assert!(output.contains("(Overlay two) Tj"));

    // Three content streams → /Contents array
    assert!(output.contains("/Contents ["));
}

// -------------------------------------------------------
// open_page: auto-close open page
// -------------------------------------------------------

#[test]
fn open_page_auto_closes_open_new_page() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Page 1", 72.0, 700.0);
    doc.end_page().unwrap();

    // Start a second page but don't explicitly close it
    doc.begin_page(612.0, 792.0);
    doc.place_text("Page 2 body", 72.0, 700.0);

    // open_page should auto-close the open page 2
    doc.open_page(1).unwrap();
    doc.place_text("Page 1 overlay", 72.0, 36.0);
    doc.end_page().unwrap();

    doc.end_document().unwrap();
    // page 2 body should have been auto-closed = doc has 2 pages
    // (verified by no panic)
}

#[test]
fn open_page_auto_close_produces_correct_page_count() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.end_page().unwrap();

    // Open page 2 without explicitly closing it first
    doc.begin_page(612.0, 792.0);
    // page_count is still 1 (page 2 not yet ended)
    assert_eq!(doc.page_count(), 1);

    // open_page auto-closes page 2 and increments count
    doc.open_page(1).unwrap();
    assert_eq!(doc.page_count(), 2);

    doc.end_page().unwrap();

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("/Count 2"));
}

// -------------------------------------------------------
// end_document auto-closes open edit page
// -------------------------------------------------------

#[test]
fn end_document_auto_closes_open_edit_page() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Main", 72.0, 700.0);
    doc.end_page().unwrap();

    doc.open_page(1).unwrap();
    doc.place_text("Footer added via open_page", 72.0, 36.0);
    // Don't call end_page; end_document should auto-close

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("(Footer added via open_page) Tj"));
}

// -------------------------------------------------------
// Page numbering use case (integration test)
// -------------------------------------------------------

#[test]
fn page_numbering_use_case() {
    let style = TextStyle {
        font: FontRef::Builtin(BuiltinFont::Helvetica),
        font_size: 10.0,
    };

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();

    // Write 3 pages of content
    for i in 1..=3 {
        doc.begin_page(612.0, 792.0);
        let mut flow = TextFlow::new();
        flow.add_text(&format!("Content for page {}", i), &style);
        let rect = Rect {
            x: 72.0,
            y: 720.0,
            width: 468.0,
            height: 648.0,
        };
        doc.fit_textflow(&mut flow, &rect).unwrap();
        doc.end_page().unwrap();
    }

    let total = doc.page_count();
    assert_eq!(total, 3);

    // Add page number footer to each page using place_text (writes full string as one literal)
    for i in 1..=total {
        doc.open_page(i).unwrap();
        doc.place_text(&format!("Page {} of {}", i, total), 72.0, 36.0);
        doc.end_page().unwrap();
    }

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    // place_text writes the whole string as a single PDF literal
    assert!(output.contains("(Page 1 of 3) Tj"), "page 1 footer missing");
    assert!(output.contains("(Page 2 of 3) Tj"), "page 2 footer missing");
    assert!(output.contains("(Page 3 of 3) Tj"), "page 3 footer missing");
    // Each page has overlay → /Contents array present
    assert!(output.contains("/Contents ["));
}

// -------------------------------------------------------
// Overlays on multiple different pages
// -------------------------------------------------------

#[test]
fn overlay_on_multiple_different_pages() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();

    for i in 1..=3 {
        doc.begin_page(612.0, 792.0);
        doc.place_text(&format!("Page {} body", i), 72.0, 700.0);
        doc.end_page().unwrap();
    }

    // Add overlays to pages 2 and 3 (not in order)
    doc.open_page(3).unwrap();
    doc.place_text("Overlay on page 3", 72.0, 36.0);
    doc.end_page().unwrap();

    doc.open_page(2).unwrap();
    doc.place_text("Overlay on page 2", 72.0, 36.0);
    doc.end_page().unwrap();

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    assert!(output.contains("(Page 1 body) Tj"), "page 1 body missing");
    assert!(output.contains("(Page 2 body) Tj"), "page 2 body missing");
    assert!(output.contains("(Page 3 body) Tj"), "page 3 body missing");
    assert!(
        output.contains("(Overlay on page 2) Tj"),
        "page 2 overlay missing"
    );
    assert!(
        output.contains("(Overlay on page 3) Tj"),
        "page 3 overlay missing"
    );
    // Page 1 has no overlay so no /Contents array for it; pages 2 and 3 do
    assert!(
        output.contains("/Contents ["),
        "pages with overlays should have array"
    );
}

// -------------------------------------------------------
// Image resources merged from overlays
// -------------------------------------------------------

#[test]
fn overlay_images_included_in_page_resources() {
    const TEST_JPEG: &[u8] = include_bytes!("fixtures/test.jpg");

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    // Main page has no image
    doc.begin_page(612.0, 792.0);
    doc.place_text("Main text", 72.0, 700.0);
    doc.end_page().unwrap();

    // Load image and place it in an overlay
    let image_id = doc.load_image_bytes(TEST_JPEG.to_vec()).unwrap();
    doc.open_page(1).unwrap();
    doc.place_image(
        &image_id,
        &Rect {
            x: 72.0,
            y: 100.0,
            width: 100.0,
            height: 100.0,
        },
        ImageFit::Fit,
    );
    doc.end_page().unwrap();

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    // The image XObject name (Im1) should appear in the page's resources
    assert!(
        output.contains("/XObject"),
        "XObject resource dict should be present"
    );
    assert!(
        output.contains("/Im1"),
        "image name Im1 should be in resources"
    );
    assert!(
        output.contains("/Im1 Do"),
        "image should be painted with Do operator"
    );
}

// -------------------------------------------------------
// Font resources merged from overlays
// -------------------------------------------------------

#[test]
fn overlay_fonts_included_in_page_resources() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    // Main page uses Helvetica only
    doc.place_text("Main text", 72.0, 700.0);
    doc.end_page().unwrap();

    // Overlay uses Courier
    let courier_style = TextStyle {
        font: FontRef::Builtin(BuiltinFont::Courier),
        font_size: 10.0,
    };
    doc.open_page(1).unwrap();
    doc.place_text_styled("Footer in Courier", 72.0, 36.0, &courier_style);
    doc.end_page().unwrap();

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    // Both fonts should be in the output
    assert!(
        output.contains("/Helvetica"),
        "Helvetica should be referenced"
    );
    assert!(output.contains("/Courier"), "Courier should be referenced");
}
