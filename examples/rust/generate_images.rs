use pdf_core::{ImageFit, PdfDocument, Rect};

fn main() {
    std::fs::create_dir_all("examples/output").unwrap();
    let path = "examples/output/rust-images.pdf";
    let mut doc = PdfDocument::create(path).unwrap();
    doc.set_compression(true);
    doc.set_info("Creator", "rust-pdf");
    doc.set_info("Title", "Image Support Demo");

    // Load images
    let jpeg = doc
        .load_image_file("pdf-core/tests/fixtures/test.jpg")
        .unwrap();
    let png = doc
        .load_image_file("pdf-core/tests/fixtures/test.png")
        .unwrap();
    let png_alpha = doc
        .load_image_file("pdf-core/tests/fixtures/test_alpha.png")
        .unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_text("Image Support Demo", 72.0, 750.0);

    // Fit mode — scales to fit, preserves aspect ratio
    doc.place_text("Fit (JPEG)", 72.0, 700.0);
    let r1 = Rect {
        x: 72.0,
        y: 100.0,
        width: 200.0,
        height: 150.0,
    };
    doc.place_image(&jpeg, &r1, ImageFit::Fit);

    // Stretch mode — fills rect exactly, may distort
    doc.place_text("Stretch (PNG)", 320.0, 700.0);
    let r2 = Rect {
        x: 320.0,
        y: 100.0,
        width: 200.0,
        height: 150.0,
    };
    doc.place_image(&png, &r2, ImageFit::Stretch);

    // Fill mode — scales to cover, clips overflow
    doc.place_text("Fill (PNG Alpha)", 72.0, 480.0);
    let r3 = Rect {
        x: 72.0,
        y: 320.0,
        width: 200.0,
        height: 150.0,
    };
    doc.place_image(&png_alpha, &r3, ImageFit::Fill);

    // None mode — natural size (1px = 1pt)
    doc.place_text("None (PNG)", 320.0, 480.0);
    let r4 = Rect {
        x: 320.0,
        y: 320.0,
        width: 200.0,
        height: 150.0,
    };
    doc.place_image(&png, &r4, ImageFit::None);

    // Same image on a second page (demonstrates write-once)
    doc.begin_page(612.0, 792.0);
    doc.place_text("Same JPEG on page 2 (XObject reused)", 72.0, 750.0);
    let r5 = Rect {
        x: 72.0,
        y: 100.0,
        width: 468.0,
        height: 600.0,
    };
    doc.place_image(&jpeg, &r5, ImageFit::Fit);

    doc.end_page().unwrap();
    doc.end_document().unwrap();
    println!("Generated: {}", path);
}
