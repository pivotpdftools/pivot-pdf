/// Example: Read an existing PDF and report the number of pages.
///
/// Reads the file produced by `generate_tables` and prints the page count.
/// Run `generate_tables` first to create the input file.
///
/// Run with:
///   cargo run --example read_tables -p pdf-examples
use pdf_core::PdfReader;

fn main() {
    let path = "examples/output/rust-tables.pdf";

    let reader = match PdfReader::open(path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {e}");
            eprintln!("Hint: run `cargo run --example generate_tables -p pdf-examples` first.");
            std::process::exit(1);
        }
    };

    println!("File:    {path}");
    println!("Version: PDF {}", reader.pdf_version());
    println!("Pages:   {}", reader.page_count());
}
