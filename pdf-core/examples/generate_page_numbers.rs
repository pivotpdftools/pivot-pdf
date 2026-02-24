/// Example: Page numbering using `open_page()`.
///
/// Demonstrates the "Page X of Y" pattern where the total page count is
/// unknown until all pages have been written:
///
/// 1. Write all pages of content using the standard textflow loop.
/// 2. Call `page_count()` to get the total.
/// 3. Loop back over pages using `open_page(i)` to add footer overlays.
///
/// Run with:
///   cargo run --example generate_page_numbers
///
/// Opens output at: output/rust-page-numbers.pdf
use pdf_core::{BuiltinFont, FontRef, FitResult, PdfDocument, Rect, TextFlow, TextStyle};

const PAGE_WIDTH: f64 = 612.0;
const PAGE_HEIGHT: f64 = 792.0;
const MARGIN: f64 = 72.0;
const CONTENT_WIDTH: f64 = PAGE_WIDTH - 2.0 * MARGIN;
const CONTENT_HEIGHT: f64 = PAGE_HEIGHT - 2.0 * MARGIN - 40.0; // leave room for footer

fn content_rect() -> Rect {
    Rect {
        x: MARGIN,
        y: PAGE_HEIGHT - MARGIN,
        width: CONTENT_WIDTH,
        height: CONTENT_HEIGHT,
    }
}

fn main() {
    std::fs::create_dir_all("output").unwrap();
    let path = "output/rust-page-numbers.pdf";
    let mut doc = PdfDocument::create(path).expect("create PDF");
    doc.set_compression(true);
    doc.set_info("Title", "Page Numbering Example");
    doc.set_info("Creator", "rust-pdf generate_page_numbers example");

    let body_style = TextStyle {
        font: FontRef::Builtin(BuiltinFont::TimesRoman),
        font_size: 12.0,
    };
    let footer_style = TextStyle {
        font: FontRef::Builtin(BuiltinFont::Helvetica),
        font_size: 9.0,
    };

    // Build a multi-page textflow with sample content
    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
        Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
        Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris. \
        Duis aute irure dolor in reprehenderit in voluptate velit esse cillum. \
        Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia. ";

    let mut flow = TextFlow::new();
    // Add enough content to fill several pages
    for i in 1..=8 {
        flow.add_text(&format!("Section {}. ", i), &TextStyle {
            font: FontRef::Builtin(BuiltinFont::HelveticaBold),
            font_size: 12.0,
        });
        for _ in 0..4 {
            flow.add_text(lorem, &body_style);
        }
        flow.add_text("\n\n", &body_style);
    }

    // --- Pass 1: write all content pages ---
    loop {
        doc.begin_page(PAGE_WIDTH, PAGE_HEIGHT);
        match doc.fit_textflow(&mut flow, &content_rect()).expect("fit_textflow") {
            FitResult::Stop => {
                doc.end_page().expect("end_page");
                break;
            }
            FitResult::BoxFull => {
                doc.end_page().expect("end_page");
            }
            FitResult::BoxEmpty => {
                doc.end_page().expect("end_page");
                break;
            }
        }
    }

    // --- Pass 2: add "Page X of Y" footer to every page ---
    let total = doc.page_count();
    println!("Total pages: {}", total);

    for i in 1..=total {
        doc.open_page(i).expect("open_page");
        doc.place_text_styled(
            &format!("Page {} of {}", i, total),
            MARGIN,
            28.0,
            &footer_style,
        );
        doc.end_page().expect("end_page after overlay");
    }

    doc.end_document().expect("end_document");
    println!("Written to {}", path);
}
