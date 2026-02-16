use pdf_core::{ImageFit, PdfDocument, Rect};

const TEST_JPEG: &[u8] = include_bytes!("fixtures/test.jpg");
const TEST_PNG: &[u8] = include_bytes!("fixtures/test.png");
const TEST_PNG_ALPHA: &[u8] = include_bytes!("fixtures/test_alpha.png");

fn make_rect() -> Rect {
    Rect {
        x: 72.0,
        y: 72.0,
        width: 200.0,
        height: 150.0,
    }
}

// -------------------------------------------------------
// Loading
// -------------------------------------------------------

#[test]
fn load_jpeg_from_bytes() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_bytes(TEST_JPEG.to_vec());
    assert!(img.is_ok(), "JPEG should load successfully");
}

#[test]
fn load_png_from_bytes() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_bytes(TEST_PNG.to_vec());
    assert!(img.is_ok(), "PNG should load successfully");
}

#[test]
fn load_png_alpha_from_bytes() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_bytes(TEST_PNG_ALPHA.to_vec());
    assert!(img.is_ok(), "RGBA PNG should load successfully");
}

#[test]
fn load_image_from_file() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_file("tests/fixtures/test.png");
    assert!(img.is_ok(), "Loading PNG from file should succeed");
}

#[test]
fn invalid_data_returns_error() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let result = doc.load_image_bytes(vec![0x00, 0x01, 0x02, 0x03]);
    assert!(result.is_err(), "Invalid data should return error");
}

// -------------------------------------------------------
// JPEG output
// -------------------------------------------------------

#[test]
fn jpeg_produces_image_xobject_with_dctdecode() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_bytes(TEST_JPEG.to_vec()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_image(&img, &make_rect(), ImageFit::Fit);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(
        output.contains("/Subtype /Image"),
        "Should contain Image XObject"
    );
    assert!(
        output.contains("/Filter /DCTDecode"),
        "JPEG should use DCTDecode filter"
    );
    assert!(
        output.contains("/ColorSpace /DeviceRGB"),
        "JPEG should have DeviceRGB color space"
    );
}

// -------------------------------------------------------
// PNG output
// -------------------------------------------------------

#[test]
fn png_produces_image_xobject() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_bytes(TEST_PNG.to_vec()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_image(&img, &make_rect(), ImageFit::Fit);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(
        output.contains("/Subtype /Image"),
        "Should contain Image XObject"
    );
    assert!(
        output.contains("/ColorSpace /DeviceRGB"),
        "PNG should have DeviceRGB color space"
    );
}

#[test]
fn rgba_png_produces_smask() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_bytes(TEST_PNG_ALPHA.to_vec()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_image(&img, &make_rect(), ImageFit::Fit);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(
        output.contains("/SMask"),
        "RGBA PNG should produce SMask reference"
    );
    // The SMask should be a DeviceGray image
    let smask_count = output.matches("/ColorSpace /DeviceGray").count();
    assert!(
        smask_count >= 1,
        "SMask should use DeviceGray color space"
    );
}

// -------------------------------------------------------
// Resources
// -------------------------------------------------------

#[test]
fn xobject_dict_in_page_resources() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_bytes(TEST_PNG.to_vec()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_image(&img, &make_rect(), ImageFit::Fit);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(
        output.contains("/XObject"),
        "Page Resources should contain /XObject dict"
    );
    assert!(
        output.contains("/Im1"),
        "XObject dict should reference /Im1"
    );
}

#[test]
fn content_stream_has_image_operators() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_bytes(TEST_PNG.to_vec()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_image(&img, &make_rect(), ImageFit::Fit);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("q\n"), "Should have save state (q)");
    assert!(output.contains("cm\n"), "Should have cm matrix");
    assert!(output.contains("/Im1 Do\n"), "Should have Do operator");
    assert!(output.contains("Q\n"), "Should have restore state (Q)");
}

// -------------------------------------------------------
// Fit modes
// -------------------------------------------------------

#[test]
fn fit_mode_preserves_aspect_ratio() {
    // Image is 100x80, rect is 200x150
    // Scale to fit: min(200/100, 150/80) = min(2.0, 1.875) = 1.875
    // Width: 100*1.875 = 187.5, Height: 80*1.875 = 150
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_bytes(TEST_PNG.to_vec()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_image(&img, &make_rect(), ImageFit::Fit);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // The cm matrix should have proportional width/height
    assert!(
        output.contains("187.5 0 0 150"),
        "Fit mode should scale to 187.5x150, got: {}",
        output
            .lines()
            .find(|l| l.contains("cm"))
            .unwrap_or("no cm found")
    );
}

#[test]
fn fill_mode_has_clipping() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_bytes(TEST_PNG.to_vec()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_image(&img, &make_rect(), ImageFit::Fill);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(
        output.contains("re W n\n"),
        "Fill mode should have clip rect (re W n)"
    );
}

#[test]
fn stretch_mode_uses_exact_rect_dimensions() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_bytes(TEST_PNG.to_vec()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_image(&img, &make_rect(), ImageFit::Stretch);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // Stretch uses exact rect dimensions: 200x150
    assert!(
        output.contains("200 0 0 150"),
        "Stretch mode should use exact dimensions 200x150, got: {}",
        output
            .lines()
            .find(|l| l.contains("cm"))
            .unwrap_or("no cm found")
    );
}

#[test]
fn none_mode_uses_natural_size() {
    // Image is 100x80, rect is 200x150
    // None mode: 1px = 1pt, so image is placed at 100x80
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_bytes(TEST_PNG.to_vec()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_image(&img, &make_rect(), ImageFit::None);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // cm matrix should use natural dimensions: 100x80
    assert!(
        output.contains("100 0 0 80"),
        "None mode should use natural size 100x80, got: {}",
        output
            .lines()
            .find(|l| l.contains("cm"))
            .unwrap_or("no cm found")
    );
    // None mode should NOT have clipping
    assert!(
        !output.contains("re W n"),
        "None mode should not clip"
    );
}

// -------------------------------------------------------
// Compression
// -------------------------------------------------------

#[test]
fn png_gets_flatedecode_when_compressed() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.set_compression(true);
    let img = doc.load_image_bytes(TEST_PNG.to_vec()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_image(&img, &make_rect(), ImageFit::Fit);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(
        output.contains("/Filter /FlateDecode"),
        "PNG image should get FlateDecode when compression is enabled"
    );
}

#[test]
fn jpeg_keeps_only_dctdecode() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.set_compression(true);
    let img = doc.load_image_bytes(TEST_JPEG.to_vec()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_image(&img, &make_rect(), ImageFit::Fit);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(
        output.contains("/Filter /DCTDecode"),
        "JPEG should keep DCTDecode"
    );
    // JPEG stream should NOT get double-compressed with FlateDecode
    // Count: DCTDecode appears once (for the image), FlateDecode may appear
    // for the page content stream but not for the JPEG image.
    let dct_count = output.matches("/DCTDecode").count();
    assert_eq!(dct_count, 1, "Only one DCTDecode filter expected");
}

// -------------------------------------------------------
// Multi-page / deduplication
// -------------------------------------------------------

#[test]
fn same_image_on_multiple_pages_written_once() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_bytes(TEST_PNG.to_vec()).unwrap();

    // Place same image on two pages
    doc.begin_page(612.0, 792.0);
    doc.place_image(&img, &make_rect(), ImageFit::Fit);
    doc.end_page().unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_image(&img, &make_rect(), ImageFit::Stretch);
    doc.end_page().unwrap();

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    // Both pages should reference the same XObject
    let im1_count = output.matches("/Im1").count();
    assert!(
        im1_count >= 2,
        "Both pages should reference /Im1 (found {} references)",
        im1_count,
    );

    // The image XObject should only be written once
    // Look for /Subtype /Image which appears in the XObject definition
    let image_obj_count = output.matches("/Subtype /Image").count();
    assert_eq!(
        image_obj_count, 1,
        "Image XObject should be written only once"
    );
}

// -------------------------------------------------------
// Mixed content
// -------------------------------------------------------

#[test]
fn mixed_text_and_images_have_font_and_xobject_resources() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img = doc.load_image_bytes(TEST_PNG.to_vec()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello", 72.0, 720.0);
    doc.place_image(&img, &make_rect(), ImageFit::Fit);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    assert!(
        output.contains("/Font"),
        "Resources should contain Font dict"
    );
    assert!(
        output.contains("/XObject"),
        "Resources should contain XObject dict"
    );
    assert!(output.contains("(Hello) Tj"), "Content should have text");
    assert!(
        output.contains("/Im1 Do"),
        "Content should have image operator"
    );
}

// -------------------------------------------------------
// Method chaining
// -------------------------------------------------------

#[test]
fn place_image_returns_self_for_chaining() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let img1 = doc.load_image_bytes(TEST_PNG.to_vec()).unwrap();
    let img2 = doc.load_image_bytes(TEST_JPEG.to_vec()).unwrap();

    let rect1 = Rect {
        x: 72.0,
        y: 72.0,
        width: 200.0,
        height: 150.0,
    };
    let rect2 = Rect {
        x: 300.0,
        y: 72.0,
        width: 200.0,
        height: 150.0,
    };

    doc.begin_page(612.0, 792.0);
    doc.place_image(&img1, &rect1, ImageFit::Fit)
        .place_image(&img2, &rect2, ImageFit::Stretch);

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("/Im1 Do"), "First image should be placed");
    assert!(output.contains("/Im2 Do"), "Second image should be placed");
}

// -------------------------------------------------------
// Full workflow
// -------------------------------------------------------

#[test]
fn full_workflow_produces_valid_pdf() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.set_info("Creator", "images-test");
    doc.set_compression(true);

    let jpeg = doc.load_image_bytes(TEST_JPEG.to_vec()).unwrap();
    let png = doc.load_image_bytes(TEST_PNG.to_vec()).unwrap();
    let png_alpha = doc.load_image_bytes(TEST_PNG_ALPHA.to_vec()).unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_text("Images Test", 72.0, 750.0);

    let r1 = Rect {
        x: 72.0,
        y: 100.0,
        width: 200.0,
        height: 150.0,
    };
    let r2 = Rect {
        x: 300.0,
        y: 100.0,
        width: 200.0,
        height: 150.0,
    };
    let r3 = Rect {
        x: 72.0,
        y: 300.0,
        width: 200.0,
        height: 150.0,
    };

    doc.place_image(&jpeg, &r1, ImageFit::Fit);
    doc.place_image(&png, &r2, ImageFit::Stretch);
    doc.place_image(&png_alpha, &r3, ImageFit::Fill);

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    assert!(output.starts_with("%PDF-1.7"), "Valid PDF header");
    assert!(output.contains("%%EOF"), "Valid PDF trailer");
    assert!(output.contains("/Type /Catalog"), "Has catalog");
    assert!(output.contains("/Count 1"), "Has one page");
    assert!(output.contains("(images-test)"), "Has info");
}
