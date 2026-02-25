use pdf_core::{BuiltinFont, FitResult, PdfDocument, Rect, TextFlow, TextStyle};

fn main() {
    std::fs::create_dir_all("examples/output").unwrap();
    let path = "examples/output/rust-textflow.pdf";
    let mut doc = PdfDocument::create(path).unwrap();
    doc.set_compression(true);
    doc.set_info("Creator", "rust-pdf");
    doc.set_info("Title", "TextFlow Example");

    let bold = TextStyle::builtin(BuiltinFont::HelveticaBold, 12.0);
    let normal = TextStyle::default();
    let times = TextStyle::builtin(BuiltinFont::TimesRoman, 11.0);
    let courier = TextStyle::builtin(BuiltinFont::Courier, 10.0);

    let mut tf = TextFlow::new();
    tf.add_text("TextFlow Demo\n\n", &bold);
    tf.add_text(
        "This document demonstrates the TextFlow \
         feature of the rust-pdf library. Text is \
         automatically wrapped within a bounding box \
         and flows across multiple pages when the box \
         is full.\n\n",
        &normal,
    );

    // Demonstrate Times-Roman
    tf.add_text(
        "This paragraph is set in Times-Roman at 11pt. \
         The quick brown fox jumps over the lazy dog.\n\n",
        &times,
    );

    // Demonstrate Courier
    tf.add_text(
        "This line is in Courier at 10pt (monospaced).\n\n",
        &courier,
    );

    // Generate several paragraphs of text to fill multiple
    // pages.
    for i in 1..=6 {
        tf.add_text(&format!("Section {}\n", i), &bold);
        tf.add_text(
            "Lorem ipsum dolor sit amet, consectetur \
             adipiscing elit. Sed do eiusmod tempor \
             incididunt ut labore et dolore magna aliqua. \
             Ut enim ad minim veniam, quis nostrud \
             exercitation ullamco laboris nisi ut aliquip \
             ex ea commodo consequat. Duis aute irure \
             dolor in reprehenderit in voluptate velit \
             esse cillum dolore eu fugiat nulla pariatur. \
             Excepteur sint occaecat cupidatat non \
             proident, sunt in culpa qui officia deserunt \
             mollit anim id est laborum.\n\n",
            &normal,
        );
        tf.add_text(" this is bold ", &bold);
        tf.add_text(
            "Curabitur pretium tincidunt lacus. Nulla \
             gravida orci a odio. Nullam varius, turpis \
             et commodo pharetra, est eros bibendum elit, \
             nec luctus magna felis sollicitudin mauris. \
             Integer in mauris eu nibh euismod gravida. \
             Duis ac tellus et risus vulputate vehicula. \
             Donec lobortis risus a elit. Etiam tempor. \
             Ut ullamcorper, ligula ut dictum pharetra, \
             nisi nunc fringilla magna, in commodo elit \
             erat nec turpis. Ut pharetra augue nec \
             augue.\n\n",
            &normal,
        );
    }

    tf.add_text("End of document.", &bold);

    // 1-inch margins on US Letter (612x792pt).
    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut page_count = 0;
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
