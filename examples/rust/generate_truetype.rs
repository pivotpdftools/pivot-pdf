use pdf_core::{BuiltinFont, FitResult, PdfDocument, Rect, TextFlow, TextStyle};

fn main() {
    // Use DejaVu Sans from the test fixtures, or pass a
    // path as the first argument.
    let font_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "pdf-core/tests/fixtures/DejaVuSans.ttf".to_string());

    std::fs::create_dir_all("examples/output").unwrap();
    let path = "examples/output/rust-truetype.pdf";
    let mut doc = PdfDocument::create(path).unwrap();
    doc.set_compression(true);
    doc.set_info("Creator", "rust-pdf");
    doc.set_info("Title", "TrueType Font Example");

    // Load a TrueType font
    let tt_font = doc
        .load_font_file(&font_path)
        .expect("Failed to load font file");

    let tt_style = TextStyle {
        font: tt_font,
        font_size: 14.0,
    };
    let tt_small = TextStyle {
        font: tt_font,
        font_size: 11.0,
    };
    let builtin = TextStyle::default();
    let bold = TextStyle::builtin(BuiltinFont::HelveticaBold, 14.0);

    // --- Page 1: Direct text placement ---
    doc.begin_page(612.0, 792.0);

    doc.place_text_styled("TrueType Font Demo", 72.0, 720.0, &bold);
    doc.place_text_styled(
        "This line uses an embedded TrueType font (DejaVu Sans).",
        72.0,
        690.0,
        &tt_style,
    );
    doc.place_text_styled("This line uses builtin Helvetica.", 72.0, 660.0, &builtin);
    doc.place_text_styled(
        "Mixed fonts on the same page work correctly.",
        72.0,
        630.0,
        &tt_small,
    );

    doc.end_page().unwrap();

    // --- Pages 2+: TextFlow with mixed fonts ---
    let mut tf = TextFlow::new();
    tf.add_text(
        "TextFlow with TrueType\n\n",
        &TextStyle {
            font: tt_font,
            font_size: 16.0,
        },
    );
    tf.add_text(
        "This paragraph is set in DejaVu Sans via an \
         embedded TrueType font. The text flows naturally \
         within the bounding box and wraps at word \
         boundaries just like builtin fonts.\n\n",
        &tt_style,
    );
    tf.add_text("Mixing fonts: ", &builtin);
    tf.add_text("this is Helvetica, ", &builtin);
    tf.add_text("and this is DejaVu Sans. ", &tt_style);
    tf.add_text("Both can appear in the same TextFlow.\n\n", &builtin);

    // Add enough text to demonstrate multi-page flow
    for i in 1..=4 {
        tf.add_text(&format!("Section {} ", i), &bold);
        tf.add_text(
            "Lorem ipsum dolor sit amet, consectetur \
             adipiscing elit. Sed do eiusmod tempor \
             incididunt ut labore et dolore magna aliqua. \
             Ut enim ad minim veniam, quis nostrud \
             exercitation ullamco laboris nisi ut aliquip \
             ex ea commodo consequat.\n\n",
            &tt_small,
        );
    }

    tf.add_text("End of document.", &bold);

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut page_count = 1; // already wrote page 1
    loop {
        doc.begin_page(612.0, 792.0);
        let result = doc.fit_textflow(&mut tf, &rect).unwrap();
        doc.end_page().unwrap();
        page_count += 1;

        match result {
            FitResult::Stop => break,
            FitResult::BoxFull => continue,
            FitResult::BoxEmpty => {
                eprintln!("Warning: bounding box too small",);
                break;
            }
        }
    }

    doc.end_document().unwrap();
    println!("Generated: {} ({} pages)", path, page_count,);
}
