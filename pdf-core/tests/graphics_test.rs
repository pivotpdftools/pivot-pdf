use pdf_core::{Color, PdfDocument};

#[test]
fn stroke_line_produces_operators() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.move_to(100.0, 200.0);
    doc.line_to(300.0, 400.0);
    doc.stroke();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("100 200 m\n"));
    assert!(output.contains("300 400 l\n"));
    assert!(output.contains("S\n"));
}

#[test]
fn set_stroke_color_operator() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.set_stroke_color(Color::rgb(1.0, 0.0, 0.0));
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("1 0 0 RG\n"));
}

#[test]
fn set_fill_color_operator() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.set_fill_color(Color::rgb(0.0, 0.5, 1.0));
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("0 0.5 1 rg\n"));
}

#[test]
fn set_line_width_operator() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.set_line_width(2.5);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("2.5 w\n"));
}

#[test]
fn rect_operator() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.rect(50.0, 50.0, 200.0, 100.0);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("50 50 200 100 re\n"));
}

#[test]
fn close_path_operator() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.move_to(0.0, 0.0);
    doc.line_to(100.0, 0.0);
    doc.line_to(50.0, 100.0);
    doc.close_path();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("h\n"));
}

#[test]
fn fill_operator() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.rect(10.0, 10.0, 50.0, 50.0);
    doc.fill();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("f\n"));
}

#[test]
fn fill_stroke_operator() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.rect(10.0, 10.0, 50.0, 50.0);
    doc.fill_stroke();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("B\n"));
}

#[test]
fn save_restore_state() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.save_state();
    doc.set_line_width(5.0);
    doc.restore_state();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("q\n"));
    assert!(output.contains("Q\n"));
}

#[test]
fn gray_color() {
    let c = Color::gray(0.5);
    assert_eq!(c.r, 0.5);
    assert_eq!(c.g, 0.5);
    assert_eq!(c.b, 0.5);
}

#[test]
fn graphics_with_text() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello", 72.0, 720.0);
    doc.set_stroke_color(Color::rgb(0.0, 0.0, 1.0));
    doc.rect(72.0, 700.0, 100.0, 20.0);
    doc.stroke();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("(Hello) Tj"));
    assert!(output.contains("0 0 1 RG\n"));
    assert!(output.contains("72 700 100 20 re\n"));
    assert!(output.contains("S\n"));
}

#[test]
fn method_chaining() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.save_state()
        .set_stroke_color(Color::rgb(1.0, 0.0, 0.0))
        .set_line_width(2.0)
        .move_to(10.0, 10.0)
        .line_to(100.0, 100.0)
        .stroke()
        .restore_state();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("q\n"));
    assert!(output.contains("1 0 0 RG\n"));
    assert!(output.contains("2 w\n"));
    assert!(output.contains("10 10 m\n"));
    assert!(output.contains("100 100 l\n"));
    assert!(output.contains("S\n"));
    assert!(output.contains("Q\n"));
}

#[test]
fn full_workflow_valid_pdf() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.set_info("Creator", "graphics-test");
    doc.begin_page(612.0, 792.0);

    // Draw a stroked rectangle
    doc.set_stroke_color(Color::rgb(0.0, 0.0, 0.0));
    doc.set_line_width(1.0);
    doc.rect(72.0, 72.0, 468.0, 648.0);
    doc.stroke();

    // Draw a filled rectangle
    doc.set_fill_color(Color::rgb(0.9, 0.9, 0.9));
    doc.rect(100.0, 100.0, 200.0, 50.0);
    doc.fill();

    // Draw a triangle with fill+stroke
    doc.save_state();
    doc.set_fill_color(Color::rgb(1.0, 0.0, 0.0));
    doc.set_stroke_color(Color::rgb(0.0, 0.0, 0.0));
    doc.move_to(300.0, 300.0);
    doc.line_to(400.0, 300.0);
    doc.line_to(350.0, 400.0);
    doc.close_path();
    doc.fill_stroke();
    doc.restore_state();

    // Add text
    doc.place_text("Graphics Test", 72.0, 740.0);

    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    // Valid PDF structure
    assert!(output.starts_with("%PDF-1.7"));
    assert!(output.contains("%%EOF"));
    assert!(output.contains("/Type /Catalog"));
    assert!(output.contains("/Type /Pages"));
    assert!(output.contains("/Count 1"));
    assert!(output.contains("(graphics-test)"));
}
