use pdf_core::{BuiltinFont, FitResult, PdfDocument, Rect, TextFlow, TextStyle, WordBreak};

/// Helper: check that a byte pattern exists in the buffer.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
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

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
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
        &TextStyle::builtin(BuiltinFont::HelveticaBold, 12.0),
    );

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
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
        &TextStyle::builtin(BuiltinFont::HelveticaBold, 12.0),
    );
    tf.add_text(" world", &TextStyle::default());

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    assert!(contains(&bytes, b"/F1 12 Tf"));
    assert!(contains(&bytes, b"/F2 12 Tf"));
    assert!(contains(&bytes, b"( bold) Tj"));
}

#[test]
fn box_empty_when_too_small() {
    let mut tf = TextFlow::new();
    tf.add_text("Hello", &TextStyle::default());

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 10.0,
    };

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    doc.end_document().unwrap();

    assert_eq!(result, FitResult::BoxEmpty);
}

#[test]
fn multi_page_flow() {
    let mut tf = TextFlow::new();
    let long_text = "word ".repeat(200);
    tf.add_text(&long_text, &TextStyle::default());

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 200.0,
        height: 50.0,
    };

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
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
                panic!("Box should not be empty");
            }
        }
    }

    let bytes = doc.end_document().unwrap();

    assert!(page_count > 1);
    let count_str = format!("/Count {}", page_count);
    assert!(contains(&bytes, count_str.as_bytes()));
}

#[test]
fn newline_forces_line_break() {
    let mut tf = TextFlow::new();
    tf.add_text("Line one\nLine two", &TextStyle::default());

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    assert!(contains(&bytes, b"(Line) Tj"));
    assert!(contains(&bytes, b"( one) Tj"));
    assert!(contains(&bytes, b"( two) Tj"));
    let output = String::from_utf8_lossy(&bytes);
    let td_count = output.matches(" Td\n").count();
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

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
}

#[test]
fn existing_place_text_still_works() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
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

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Title", 72.0, 720.0);
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
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
    tf.add_text("Hello world", &TextStyle::default());

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 40.0,
        height: 648.0,
    };

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    assert!(contains(&bytes, b"(Hello) Tj"));
    assert!(contains(&bytes, b"(world) Tj"));
    let output = String::from_utf8_lossy(&bytes);
    let td_count = output.matches(" Td\n").count();
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

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
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
        &TextStyle::builtin(BuiltinFont::HelveticaBold, 12.0),
    );

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(contains(&bytes, b"/BaseFont /Helvetica-Bold"));
    assert!(contains(&bytes, b"/BaseFont /Helvetica"));
    assert!(contains(&bytes, b"/F1"));
    assert!(contains(&bytes, b"/F2"));
}

#[test]
fn times_font_in_textflow() {
    let mut tf = TextFlow::new();
    tf.add_text(
        "Times text",
        &TextStyle::builtin(BuiltinFont::TimesRoman, 12.0),
    );

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    assert!(contains(&bytes, b"/F5 12 Tf"));
    assert!(contains(&bytes, b"(Times) Tj"));
}

#[test]
fn courier_font_in_textflow() {
    let mut tf = TextFlow::new();
    tf.add_text("Code", &TextStyle::builtin(BuiltinFont::Courier, 12.0));

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    assert!(contains(&bytes, b"/F9 12 Tf"));
    assert!(contains(&bytes, b"(Code) Tj"));
}

#[test]
fn place_text_styled_uses_correct_font() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text_styled(
        "Styled",
        72.0,
        720.0,
        &TextStyle::builtin(BuiltinFont::TimesBold, 18.0),
    );
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(contains(&bytes, b"/F6 18 Tf"));
    assert!(contains(&bytes, b"(Styled) Tj"));
}

// -------------------------------------------------------
// Word-break tests
// -------------------------------------------------------

/// A narrow box where the long word must be broken.
fn narrow_rect() -> Rect {
    Rect {
        x: 72.0,
        y: 720.0,
        width: 60.0,
        height: 200.0,
    }
}

fn make_doc() -> PdfDocument<Vec<u8>> {
    PdfDocument::new(Vec::<u8>::new()).unwrap()
}

#[test]
fn break_all_splits_long_word_across_lines() {
    // "WWWWWWWWWW" at 12pt Helvetica is much wider than 60pt.
    // With BreakAll (default), it should be split into pieces that each fit.
    let style = TextStyle::default(); // 12pt Helvetica
    let mut tf = TextFlow::new();
    tf.add_text("WWWWWWWWWW", &style);
    // word_break defaults to BreakAll — no explicit set needed.

    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &narrow_rect()).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    // All text was placed (no overflow).
    assert_eq!(result, FitResult::Stop);
    // Multiple Td operators mean multiple lines were emitted.
    assert!(
        contains(&bytes, b"0 -"),
        "expected multi-line Td operators from word break"
    );
}

#[test]
fn break_all_result_is_stop_not_box_empty() {
    // Before word-break was implemented, a word wider than the box returned
    // BoxEmpty. Now it should split the word and return Stop.
    let mut tf = TextFlow::new();
    tf.add_text("superlongwordwithoutspaces", &TextStyle::default());

    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &narrow_rect()).unwrap();
    doc.end_page().unwrap();
    doc.end_document().unwrap();

    assert_ne!(
        result,
        FitResult::BoxEmpty,
        "word break should prevent BoxEmpty"
    );
    assert_eq!(result, FitResult::Stop);
}

#[test]
fn hyphenate_mode_inserts_hyphen_at_break() {
    let style = TextStyle::default();
    let mut tf = TextFlow::new();
    tf.word_break = WordBreak::Hyphenate;
    tf.add_text("WWWWWWWWWW", &style);

    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &narrow_rect()).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    // A hyphen at the end of a PDF literal string looks like `-)`.
    // Checking for `-) Tj` avoids false positives from negative coordinates.
    assert!(
        contains(&bytes, b"-) Tj"),
        "hyphenate mode should emit a hyphen at break points"
    );
}

#[test]
fn normal_mode_does_not_break_word() {
    // With WordBreak::Normal, wide words are emitted as-is (overflow).
    // The box is too narrow for "WWWW" at 12pt but the result should still
    // complete (BoxEmpty is returned because no text can fit at all when
    // the first word is wider than the box and nothing has been placed yet).
    let mut tf = TextFlow::new();
    tf.word_break = WordBreak::Normal;
    tf.add_text("WWWWWWWWWW", &TextStyle::default());

    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    // Use a very narrow rect so the word definitely cannot fit.
    let tiny_rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 10.0,
        height: 200.0,
    };
    let result = doc.fit_textflow(&mut tf, &tiny_rect).unwrap();
    doc.end_page().unwrap();
    doc.end_document().unwrap();

    // Without word-break the flow cannot place the word in a 10pt-wide box.
    assert_eq!(result, FitResult::BoxEmpty);
}

#[test]
fn word_break_does_not_affect_normal_words() {
    // Short words that fit on a line should be placed unchanged.
    let mut tf = TextFlow::new();
    tf.add_text("Hello world", &TextStyle::default());

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 200.0,
    };
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    assert!(contains(&bytes, b"(Hello) Tj"));
    assert!(contains(&bytes, b"( world) Tj"));
}

#[test]
fn break_all_multi_page_cursor_is_consistent() {
    // A very long word that forces a page break mid-word should resume
    // correctly on the next page with the remaining characters.
    let mut tf = TextFlow::new();
    // 26 W's — much wider than the narrow box; forces many lines.
    tf.add_text("WWWWWWWWWWWWWWWWWWWWWWWWWW", &TextStyle::default());

    // A box that only fits ~2 lines of text.
    let small_box = Rect {
        x: 72.0,
        y: 720.0,
        width: 60.0,
        height: 30.0,
    };

    let mut doc = make_doc();

    doc.begin_page(612.0, 792.0);
    let r1 = doc.fit_textflow(&mut tf, &small_box).unwrap();
    doc.end_page().unwrap();

    doc.begin_page(612.0, 792.0);
    let r2 = doc.fit_textflow(&mut tf, &small_box).unwrap();
    doc.end_page().unwrap();

    doc.begin_page(612.0, 792.0);
    let r3 = doc.fit_textflow(&mut tf, &small_box).unwrap();
    doc.end_page().unwrap();
    doc.end_document().unwrap();

    // At least the first call should return BoxFull (more text remains),
    // and eventually a Stop should be produced.
    assert_eq!(
        r1,
        FitResult::BoxFull,
        "first page should be full, not all placed"
    );
    let finished = r2 == FitResult::Stop || r3 == FitResult::Stop;
    assert!(finished, "text should eventually be fully placed");
}
