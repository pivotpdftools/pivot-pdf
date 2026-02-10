use pdf_core::{
    BuiltinFont, FitResult, PdfDocument, Rect, TextFlow,
    TextStyle,
};

/// Helper: check that a byte pattern exists in the buffer.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|w| w == needle)
}

#[test]
fn simple_text_fits_in_one_box() {
    let mut tf = TextFlow::new();
    tf.add_text("Hello world", &TextStyle::default());

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result =
        doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    assert!(contains(&bytes, b"(Hello) Tj"));
    assert!(contains(&bytes, b"( world) Tj"));
    assert!(contains(&bytes, b"/F1 12 Tf"));
}

#[test]
fn bold_text_uses_f2() {
    let mut tf = TextFlow::new();
    tf.add_text(
        "bold",
        &TextStyle {
            font: BuiltinFont::HelveticaBold,
            ..Default::default()
        },
    );

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result =
        doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    assert!(contains(&bytes, b"/F2 12 Tf"));
    assert!(contains(&bytes, b"(bold) Tj"));
}

#[test]
fn mixed_bold_and_normal() {
    let mut tf = TextFlow::new();
    tf.add_text("Hello ", &TextStyle::default());
    tf.add_text(
        "bold",
        &TextStyle {
            font: BuiltinFont::HelveticaBold,
            ..Default::default()
        },
    );
    tf.add_text(" world", &TextStyle::default());

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result =
        doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    // Should switch between F1 and F2
    assert!(contains(&bytes, b"/F1 12 Tf"));
    assert!(contains(&bytes, b"/F2 12 Tf"));
    assert!(contains(&bytes, b"( bold) Tj"));
}

#[test]
fn box_empty_when_too_small() {
    let mut tf = TextFlow::new();
    tf.add_text("Hello", &TextStyle::default());

    // Height too small for even one line (line_height=14.4)
    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 10.0,
    };

    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result =
        doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    doc.end_document().unwrap();

    assert_eq!(result, FitResult::BoxEmpty);
}

#[test]
fn multi_page_flow() {
    let mut tf = TextFlow::new();
    // Generate enough text to overflow a small box
    let long_text = "word ".repeat(200);
    tf.add_text(&long_text, &TextStyle::default());

    // Small box: fits only a few lines
    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 200.0,
        height: 50.0,
    };

    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    let mut page_count = 0;

    loop {
        doc.begin_page(612.0, 792.0);
        let result =
            doc.fit_textflow(&mut tf, &rect).unwrap();
        doc.end_page().unwrap();
        page_count += 1;

        match result {
            FitResult::Stop => break,
            FitResult::BoxFull => continue,
            FitResult::BoxEmpty => {
                panic!("Box should not be empty");
            }
        }
    }

    let bytes = doc.end_document().unwrap();

    // Should span multiple pages
    assert!(page_count > 1);
    let count_str =
        format!("/Count {}", page_count);
    assert!(contains(
        &bytes,
        count_str.as_bytes()
    ));
}

#[test]
fn newline_forces_line_break() {
    let mut tf = TextFlow::new();
    tf.add_text(
        "Line one\nLine two",
        &TextStyle::default(),
    );

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result =
        doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    // Each word is a separate Tj; lines separated by Td
    assert!(contains(&bytes, b"(Line) Tj"));
    assert!(contains(&bytes, b"( one) Tj"));
    assert!(contains(&bytes, b"( two) Tj"));
    // Should have two Td operations (two lines)
    let output = String::from_utf8_lossy(&bytes);
    let td_count =
        output.matches(" Td\n").count();
    assert_eq!(td_count, 2);
}

#[test]
fn empty_textflow_returns_stop() {
    let mut tf = TextFlow::new();

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result =
        doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
}

#[test]
fn existing_place_text_still_works() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello", 20.0, 20.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(bytes.starts_with(b"%PDF-1.7\n"));
    assert!(bytes.ends_with(b"%%EOF\n"));
    assert!(contains(&bytes, b"(Hello) Tj"));
    assert!(contains(&bytes, b"/F1 12 Tf"));
    assert!(contains(&bytes, b"20 20 Td"));
    assert!(contains(&bytes, b"/BaseFont /Helvetica"));
}

#[test]
fn place_text_and_textflow_on_same_page() {
    let mut tf = TextFlow::new();
    tf.add_text("Flowed text", &TextStyle::default());

    let rect = Rect {
        x: 72.0,
        y: 400.0,
        width: 468.0,
        height: 200.0,
    };

    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Title", 72.0, 720.0);
    let result =
        doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    assert!(contains(&bytes, b"(Title) Tj"));
    assert!(contains(&bytes, b"(Flowed) Tj"));
    assert!(contains(&bytes, b"( text) Tj"));
}

#[test]
fn word_wrapping_respects_box_width() {
    let mut tf = TextFlow::new();
    // "Hello world" at 12pt Helvetica:
    // H=722, e=556, l=222, l=222, o=556 = 2278 => 27.336pt
    // space=278 => 3.336pt
    // w=722, o=556, r=333, l=222, d=556 = 2389 => 28.668pt
    // Total: ~59.34pt
    // Use a width that fits "Hello" but not "Hello world"
    tf.add_text(
        "Hello world",
        &TextStyle::default(),
    );

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 40.0, // Only fits "Hello" (27.3pt)
        height: 648.0,
    };

    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result =
        doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    // Words should be on separate lines
    assert!(contains(&bytes, b"(Hello) Tj"));
    // "world" is first on its line, so no leading space
    assert!(contains(&bytes, b"(world) Tj"));
    // Should have two Td positioning commands (two lines)
    let output = String::from_utf8_lossy(&bytes);
    let td_count =
        output.matches(" Td\n").count();
    assert_eq!(td_count, 2);
}

#[test]
fn space_preserved_between_text_flows() {
    let mut tf = TextFlow::new();
    let normal = TextStyle::default();
    tf.add_text("this is bold ", &normal);
    tf.add_text("and this is not", &normal);

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result =
        doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    // "bold" ends span 1, "and" starts span 2.
    // The trailing space after "bold " must be preserved
    // so "and" has a leading space.
    assert!(contains(&bytes, b"( bold) Tj"));
    assert!(
        contains(&bytes, b"( and) Tj"),
        "Expected '( and) Tj' but space between spans \
         was lost. Output: {}",
        String::from_utf8_lossy(&bytes),
    );
}

#[test]
fn bold_font_in_pdf_output() {
    let mut tf = TextFlow::new();
    tf.add_text("normal ", &TextStyle::default());
    tf.add_text(
        "bold",
        &TextStyle {
            font: BuiltinFont::HelveticaBold,
            ..Default::default()
        },
    );

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    // Both fonts should be in the PDF
    assert!(contains(
        &bytes,
        b"/BaseFont /Helvetica-Bold"
    ));
    assert!(contains(
        &bytes,
        b"/BaseFont /Helvetica"
    ));
    // Resources should reference both
    assert!(contains(&bytes, b"/F1"));
    assert!(contains(&bytes, b"/F2"));
}

#[test]
fn times_font_in_textflow() {
    let mut tf = TextFlow::new();
    tf.add_text(
        "Times text",
        &TextStyle {
            font: BuiltinFont::TimesRoman,
            ..Default::default()
        },
    );

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result =
        doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    assert!(contains(&bytes, b"/F5 12 Tf"));
    assert!(contains(&bytes, b"(Times) Tj"));
}

#[test]
fn courier_font_in_textflow() {
    let mut tf = TextFlow::new();
    tf.add_text(
        "Code",
        &TextStyle {
            font: BuiltinFont::Courier,
            ..Default::default()
        },
    );

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result =
        doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    assert!(contains(&bytes, b"/F9 12 Tf"));
    assert!(contains(&bytes, b"(Code) Tj"));
}

#[test]
fn place_text_styled_uses_correct_font() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text_styled(
        "Styled",
        72.0,
        720.0,
        &TextStyle {
            font: BuiltinFont::TimesBold,
            font_size: 18.0,
        },
    );
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(contains(&bytes, b"/F6 18 Tf"));
    assert!(contains(&bytes, b"(Styled) Tj"));
}
