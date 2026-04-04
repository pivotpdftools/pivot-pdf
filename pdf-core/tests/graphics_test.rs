use pivot_pdf::{Angle, Color, DocumentOptions, PdfDocument};

#[test]
fn stroke_line_produces_operators() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
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
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.set_stroke_color(Color::rgb(1.0, 0.0, 0.0));
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("1 0 0 RG\n"));
}

#[test]
fn set_fill_color_operator() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.set_fill_color(Color::rgb(0.0, 0.5, 1.0));
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("0 0.5 1 rg\n"));
}

#[test]
fn set_line_width_operator() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.set_line_width(2.5);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("2.5 w\n"));
}

#[test]
fn rect_operator() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.rect(50.0, 50.0, 200.0, 100.0);
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("50 50 200 100 re\n"));
}

#[test]
fn close_path_operator() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
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
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.rect(10.0, 10.0, 50.0, 50.0);
    doc.fill();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("f\n"));
}

#[test]
fn fill_stroke_operator() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.rect(10.0, 10.0, 50.0, 50.0);
    doc.fill_stroke();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("B\n"));
}

#[test]
fn save_restore_state() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
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
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
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
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
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
fn circle_emits_four_curves_and_closes() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.circle(200.0, 300.0, 50.0).stroke();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // Full circle needs 4 Bezier segments
    assert_eq!(
        output.matches(" c\n").count(),
        4,
        "circle needs 4 bezier segments"
    );
    // Path must be closed
    assert!(output.contains("h\n"), "circle must close the path");
}

#[test]
fn arc_quarter_circle_emits_move_and_curve() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    // 0° to 90° arc at (0,0) radius 100 — start point is (100, 0)
    doc.arc(0.0, 0.0, 100.0, Angle::degrees(0.0), Angle::degrees(90.0))
        .stroke();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // starts with move_to to the start point
    assert!(output.contains("100 0 m\n"), "expected move_to start point");
    // emits at least one cubic bezier
    assert!(output.contains(" c\n"), "expected bezier curve operator");
    // does NOT close the path automatically
    assert!(!output.contains("h\n"), "arc should not close path");
}

#[test]
fn arc_half_circle_emits_two_curves() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    // 0° to 180° arc — needs 2 segments
    doc.arc(0.0, 0.0, 100.0, Angle::degrees(0.0), Angle::degrees(180.0))
        .stroke();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert_eq!(
        output.matches(" c\n").count(),
        2,
        "half circle needs 2 bezier segments"
    );
}

#[test]
fn arc_topleft_origin_transforms_center() {
    use pivot_pdf::{DocumentOptions, Origin};
    let opts = DocumentOptions {
        origin: Origin::TopLeft,
    };
    let mut doc = PdfDocument::new(Vec::<u8>::new(), opts).unwrap();
    doc.begin_page(612.0, 792.0);
    // center at (100, 100) TopLeft → y_pdf = 792-100 = 692
    // 0° arc start x = cx + r = 200, y_pdf = 692 (point y stays same as center y since sin(0)=0)
    doc.arc(
        100.0,
        100.0,
        100.0,
        Angle::degrees(0.0),
        Angle::degrees(90.0),
    )
    .stroke();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // start point: x=200, y=792-100=692 (sin(0)=0, so y_pdf = cy_pdf + r*sin(0) = 692)
    assert!(
        output.contains("200 692 m\n"),
        "start point should be in PDF space"
    );
}

#[test]
fn curve_to_emits_c_operator() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.move_to(50.0, 100.0)
        .curve_to(80.0, 200.0, 120.0, 200.0, 150.0, 100.0)
        .stroke();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("50 100 m\n"));
    assert!(output.contains("80 200 120 200 150 100 c\n"));
    assert!(output.contains("S\n"));
}

#[test]
fn curve_to_topleft_origin() {
    use pivot_pdf::{DocumentOptions, Origin};
    let opts = DocumentOptions {
        origin: Origin::TopLeft,
    };
    let mut doc = PdfDocument::new(Vec::<u8>::new(), opts).unwrap();
    doc.begin_page(612.0, 792.0);
    // With TopLeft: y_pdf = page_height - y_user = 792 - y
    doc.curve_to(80.0, 200.0, 120.0, 200.0, 150.0, 100.0)
        .stroke();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // y1 = 792-200=592, y2 = 792-200=592, y3 = 792-100=692
    assert!(output.contains("80 592 120 592 150 692 c\n"));
}

#[test]
fn angle_degrees_to_radians() {
    let a = Angle::degrees(180.0);
    assert!((a.to_radians() - std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn angle_radians_roundtrip() {
    let a = Angle::radians(std::f64::consts::FRAC_PI_2);
    assert!((a.to_radians() - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
}

#[test]
fn full_workflow_valid_pdf() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
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
