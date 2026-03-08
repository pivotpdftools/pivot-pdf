use pivot_pdf::{PdfDocument, Rect};

fn main() {
    std::fs::create_dir_all("examples/output").unwrap();
    let path = "examples/output/rust-form-fields.pdf";
    let mut doc = PdfDocument::create(path).unwrap();
    doc.set_compression(true);
    doc.set_info("Creator", "pivot-pdf");
    doc.set_info("Title", "Form Fields Example");

    doc.begin_page(612.0, 792.0);

    // Labels
    doc.place_text("Full Name:", 72.0, 718.0);
    doc.place_text("Email:", 72.0, 688.0);
    doc.place_text("Phone:", 72.0, 658.0);
    doc.place_text("Comments:", 72.0, 608.0);

    // Fillable text fields (invisible — viewer renders its own chrome)
    // A line is drawn under each field at the bottom edge of its rect.
    doc.set_line_width(0.5);

    let fields: &[(&str, f64, f64, f64, f64)] = &[
        ("full_name", 180.0, 706.0, 300.0, 18.0),
        ("email",     180.0, 676.0, 300.0, 18.0),
        ("phone",     180.0, 646.0, 200.0, 18.0),
        ("comments",  180.0, 576.0, 300.0, 48.0),
    ];

    for &(name, x, y, width, height) in fields {
        doc.add_text_field(name, Rect { x, y, width, height }).unwrap();
        doc.move_to(x, y);
        doc.line_to(x + width, y);
        doc.stroke();
    }

    doc.end_page().unwrap();
    doc.end_document().unwrap();
    println!("Generated: {}", path);
}
