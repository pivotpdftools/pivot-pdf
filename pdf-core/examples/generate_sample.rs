use pdf_core::PdfDocument;

fn main() {
    let path = "sample_output.pdf";
    let mut doc = PdfDocument::create(path).unwrap();
    doc.set_info("Creator", "rust-pdf");
    doc.set_info("Title", "A Test Document");
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello, PDF!", 72.0, 720.0);
    doc.place_text("Created by rust-pdf library.", 72.0, 700.0);
    doc.end_page().unwrap();
    doc.end_document().unwrap();
    println!("Generated: {}", path);
}
