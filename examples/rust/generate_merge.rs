/// Example: Merge two previously generated PDFs and report the page count.
///
/// Merges the files produced by `generate_tables` and `generate_invoice` into
/// a single PDF, then reads it back and prints the total page count.
///
/// Run `generate_tables` and `generate_invoice` first, then:
///   cargo run --example generate_merge -p pdf-examples
///
/// Output: examples/output/rust-merged.pdf
use pivot_pdf::{merge_pdfs, MergeOptions, PdfReader};

fn main() {
    let inputs = [
        "examples/output/rust-tables.pdf",
        "examples/output/rust-invoice.pdf",
    ];
    let output = "examples/output/rust-merged.pdf";

    // Verify source files exist before attempting the merge.
    for path in &inputs {
        if !std::path::Path::new(path).exists() {
            eprintln!("Missing input: {path}");
            eprintln!("Hint: run `generate_tables` and `generate_invoice` examples first.");
            std::process::exit(1);
        }
    }

    println!("Merging:");
    for path in &inputs {
        let reader = PdfReader::open(path).expect("failed to open source PDF");
        println!("  {path}  ({} pages)", reader.page_count());
    }

    match merge_pdfs(&inputs, output, MergeOptions::default()) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }

    let reader = PdfReader::open(output).expect("failed to read merged PDF");
    println!("\nOutput: {output}");
    println!("Pages:  {}", reader.page_count());
}
