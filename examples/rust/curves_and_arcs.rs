use pivot_pdf::{Angle, Color, DocumentOptions, PdfDocument};

fn main() {
    std::fs::create_dir_all("examples/output").unwrap();
    let path = "examples/output/rust-curves-and-arcs.pdf";
    let mut doc = PdfDocument::create(path, DocumentOptions::default()).unwrap();
    doc.set_compression(true);
    doc.set_info("Creator", "rust-pdf");
    doc.set_info("Title", "Curves and Arcs Demo");
    doc.begin_page(612.0, 792.0);

    doc.place_text("Curves and Arcs Demo", 72.0, 750.0);

    // --- Cubic Bezier curve (S-curve) ---
    doc.save_state();
    doc.set_stroke_color(Color::rgb(0.0, 0.0, 0.8));
    doc.set_line_width(2.0);
    doc.move_to(80.0, 680.0)
        .curve_to(150.0, 730.0, 220.0, 630.0, 290.0, 680.0)
        .stroke();
    doc.restore_state();
    doc.place_text("Cubic Bezier (curve_to)", 300.0, 680.0);

    // --- Quarter arc (pie slice) ---
    doc.save_state();
    doc.set_stroke_color(Color::rgb(0.8, 0.0, 0.0));
    doc.set_line_width(2.0);
    let cx = 130.0_f64;
    let cy = 580.0_f64;
    let r = 60.0_f64;
    // Draw the pie slice: line to edge, arc, close back to center
    doc.move_to(cx, cy).line_to(cx + r, cy);
    doc.arc(cx, cy, r, Angle::degrees(0.0), Angle::degrees(90.0));
    doc.close_path().fill_stroke();
    doc.restore_state();
    doc.place_text("Quarter arc (pie slice)", 220.0, 580.0);

    // --- Half circle (dome) ---
    doc.save_state();
    doc.set_fill_color(Color::rgb(0.9, 0.9, 0.2));
    doc.set_stroke_color(Color::rgb(0.5, 0.5, 0.0));
    doc.set_line_width(1.5);
    doc.arc(
        130.0,
        460.0,
        60.0,
        Angle::degrees(0.0),
        Angle::degrees(180.0),
    );
    doc.close_path().fill_stroke();
    doc.restore_state();
    doc.place_text("Half circle (arc 0°–180°)", 220.0, 480.0);

    // --- Full circles ---
    doc.save_state();
    doc.set_stroke_color(Color::rgb(0.0, 0.6, 0.0));
    doc.set_line_width(2.0);
    doc.circle(130.0, 340.0, 50.0).stroke();
    doc.restore_state();
    doc.place_text("Stroked circle", 200.0, 355.0);

    doc.save_state();
    doc.set_fill_color(Color::rgb(0.8, 0.4, 0.8));
    doc.circle(130.0, 220.0, 50.0).fill();
    doc.restore_state();
    doc.place_text("Filled circle", 200.0, 235.0);

    // --- Concentric rings ---
    doc.save_state();
    doc.set_stroke_color(Color::rgb(0.0, 0.4, 0.8));
    for i in 1..=4_u32 {
        doc.set_line_width(i as f64 * 0.5);
        doc.circle(480.0, 450.0, i as f64 * 30.0).stroke();
    }
    doc.restore_state();
    doc.place_text("Concentric rings", 430.0, 580.0);

    // --- Arc used as a rounded cap ---
    doc.save_state();
    doc.set_stroke_color(Color::rgb(0.5, 0.0, 0.5));
    doc.set_line_width(1.5);
    // Semicircle cap at the end of a line
    doc.move_to(400.0, 200.0).line_to(520.0, 200.0);
    doc.arc(
        520.0,
        200.0,
        20.0,
        Angle::degrees(-90.0),
        Angle::degrees(90.0),
    );
    doc.stroke();
    doc.restore_state();
    doc.place_text("Arc as rounded cap", 390.0, 235.0);

    doc.end_page().unwrap();
    doc.end_document().unwrap();
    println!("Generated: {}", path);
}
