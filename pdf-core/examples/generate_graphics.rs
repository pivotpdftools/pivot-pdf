use pdf_core::{Color, PdfDocument};

fn main() {
    let path = "graphics_output.pdf";
    let mut doc = PdfDocument::create(path).unwrap();
    doc.set_info("Creator", "rust-pdf");
    doc.set_info("Title", "Line Graphics Demo");
    doc.begin_page(612.0, 792.0);

    // Stroked rectangle (page border)
    doc.set_stroke_color(Color::rgb(0.0, 0.0, 0.0));
    doc.set_line_width(1.0);
    doc.rect(72.0, 72.0, 468.0, 648.0);
    doc.stroke();

    // Filled rectangle (light gray background box)
    doc.set_fill_color(Color::gray(0.9));
    doc.rect(100.0, 600.0, 200.0, 50.0);
    doc.fill();

    // Diagonal line
    doc.set_stroke_color(Color::rgb(0.0, 0.0, 1.0));
    doc.set_line_width(2.0);
    doc.move_to(100.0, 500.0);
    doc.line_to(300.0, 550.0);
    doc.stroke();

    // Triangle with fill and stroke
    doc.save_state();
    doc.set_fill_color(Color::rgb(1.0, 0.0, 0.0));
    doc.set_stroke_color(Color::rgb(0.0, 0.0, 0.0));
    doc.set_line_width(1.5);
    doc.move_to(350.0, 400.0)
        .line_to(450.0, 400.0)
        .line_to(400.0, 480.0)
        .close_path()
        .fill_stroke();
    doc.restore_state();

    // Nested rectangles using save/restore to isolate state
    doc.save_state();
    doc.set_stroke_color(Color::rgb(0.0, 0.5, 0.0));
    doc.set_line_width(3.0);
    doc.rect(150.0, 200.0, 300.0, 150.0);
    doc.stroke();

    doc.set_fill_color(Color::rgb(0.8, 0.9, 0.8));
    doc.rect(180.0, 230.0, 240.0, 90.0);
    doc.fill();
    doc.restore_state();

    // Add a label
    doc.place_text("Line Graphics Demo", 72.0, 740.0);

    doc.end_page().unwrap();
    doc.end_document().unwrap();
    println!("Generated: {}", path);
}
