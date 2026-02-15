use pdf_core::{BuiltinFont, FitResult, FontRef, PdfDocument, Rect, TextFlow, TextStyle};

const DEJAVU_SANS: &[u8] = include_bytes!("fixtures/DejaVuSans.ttf");

/// Helper: check that a byte pattern exists in the buffer.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

// ---- Font parsing and metrics ----

#[test]
fn parse_ttf_and_verify_metrics() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let font_ref = doc.load_font_bytes(DEJAVU_SANS.to_vec()).unwrap();
    // Should return a TrueType font ref
    match font_ref {
        FontRef::TrueType(_) => {}
        _ => panic!("Expected TrueType font ref"),
    }
}

#[test]
fn truetype_font_produces_valid_pdf() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let font_ref = doc.load_font_bytes(DEJAVU_SANS.to_vec()).unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_text_styled(
        "Hello TrueType",
        72.0,
        720.0,
        &TextStyle {
            font: font_ref,
            font_size: 14.0,
        },
    );
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    // Valid PDF structure
    assert!(bytes.starts_with(b"%PDF-1.7\n"));
    assert!(bytes.ends_with(b"%%EOF\n"));

    let output = String::from_utf8_lossy(&bytes);

    // Type0 composite font structure
    assert!(output.contains("/Subtype /Type0"), "Missing Type0 font");
    assert!(
        output.contains("/Subtype /CIDFontType2"),
        "Missing CIDFontType2"
    );
    assert!(
        output.contains("/Type /FontDescriptor"),
        "Missing FontDescriptor"
    );
    assert!(
        output.contains("/Encoding /Identity-H"),
        "Missing Identity-H encoding"
    );

    // ToUnicode CMap
    assert!(output.contains("beginbfchar"), "Missing ToUnicode CMap");

    // Text should be hex-encoded glyph IDs
    assert!(output.contains("> Tj"), "Missing hex-encoded text");

    // Font resource referenced on page
    assert!(
        output.contains("/F15"),
        "Missing TrueType font resource name"
    );
}

#[test]
fn hex_encoding_format() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let font_ref = doc.load_font_bytes(DEJAVU_SANS.to_vec()).unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_text_styled(
        "AB",
        72.0,
        720.0,
        &TextStyle {
            font: font_ref,
            font_size: 12.0,
        },
    );
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    // Text should be hex-encoded: <XXXXXXXX> Tj
    // Each glyph is 4 hex chars, "AB" = 2 glyphs = 8 hex chars
    assert!(output.contains("> Tj"), "Missing hex text operator");
    // Should NOT contain literal string encoding for TT text
    assert!(
        !output.contains("(AB) Tj"),
        "TrueType text should not use literal strings"
    );
}

#[test]
fn mixed_builtin_and_truetype_on_same_page() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let tt_font = doc.load_font_bytes(DEJAVU_SANS.to_vec()).unwrap();

    doc.begin_page(612.0, 792.0);

    // Builtin font text
    doc.place_text_styled(
        "Builtin",
        72.0,
        720.0,
        &TextStyle::builtin(BuiltinFont::Helvetica, 12.0),
    );

    // TrueType font text
    doc.place_text_styled(
        "TrueType",
        72.0,
        700.0,
        &TextStyle {
            font: tt_font,
            font_size: 12.0,
        },
    );

    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    // Both font types in resources
    assert!(output.contains("/F1"), "Missing builtin font");
    assert!(output.contains("/F15"), "Missing TT font");

    // Builtin uses literal, TT uses hex
    assert!(
        output.contains("(Builtin) Tj"),
        "Builtin text should use literal"
    );
    assert!(output.contains("> Tj"), "TT text should use hex");

    // Both font subtypes present
    assert!(output.contains("/Subtype /Type1"));
    assert!(output.contains("/Subtype /Type0"));
}

#[test]
fn textflow_with_truetype() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let tt_font = doc.load_font_bytes(DEJAVU_SANS.to_vec()).unwrap();

    let style = TextStyle {
        font: tt_font,
        font_size: 12.0,
    };

    let mut tf = TextFlow::new();
    tf.add_text("Hello TrueType TextFlow", &style);

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);

    let output = String::from_utf8_lossy(&bytes);
    // Should use hex encoding in textflow too
    assert!(
        output.contains("> Tj"),
        "TextFlow TT should use hex encoding"
    );
    assert!(output.contains("/F15"));
    assert!(output.contains("/Subtype /Type0"));
}

#[test]
fn textflow_mixed_builtin_and_truetype() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let tt_font = doc.load_font_bytes(DEJAVU_SANS.to_vec()).unwrap();

    let normal = TextStyle::default();
    let tt_style = TextStyle {
        font: tt_font,
        font_size: 12.0,
    };

    let mut tf = TextFlow::new();
    tf.add_text("Builtin ", &normal);
    tf.add_text("TrueType", &tt_style);

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 468.0,
        height: 648.0,
    };

    doc.begin_page(612.0, 792.0);
    let result = doc.fit_textflow(&mut tf, &rect).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);

    let output = String::from_utf8_lossy(&bytes);
    // Both font types used
    assert!(output.contains("/F1 12 Tf"));
    assert!(output.contains("/F15 12 Tf"));
    // Builtin literal + TT hex
    assert!(output.contains("(Builtin) Tj"));
    assert!(output.contains("> Tj"));
}

#[test]
fn truetype_multi_page_textflow() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let tt_font = doc.load_font_bytes(DEJAVU_SANS.to_vec()).unwrap();

    let style = TextStyle {
        font: tt_font,
        font_size: 12.0,
    };

    let mut tf = TextFlow::new();
    let long_text = "word ".repeat(200);
    tf.add_text(&long_text, &style);

    let rect = Rect {
        x: 72.0,
        y: 720.0,
        width: 200.0,
        height: 50.0,
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
                panic!("Box should not be empty");
            }
        }
    }

    let bytes = doc.end_document().unwrap();

    assert!(page_count > 1);
    // Font objects should appear only once despite multi-page use
    let output = String::from_utf8_lossy(&bytes);
    let type0_count = output.matches("/Subtype /Type0").count();
    assert_eq!(type0_count, 1, "Type0 font should be written once");
}

#[test]
fn font_descriptor_has_required_fields() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let font_ref = doc.load_font_bytes(DEJAVU_SANS.to_vec()).unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_text_styled(
        "Test",
        72.0,
        720.0,
        &TextStyle {
            font: font_ref,
            font_size: 12.0,
        },
    );
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    assert!(output.contains("/FontName"));
    assert!(output.contains("/Flags"));
    assert!(output.contains("/FontBBox"));
    assert!(output.contains("/ItalicAngle"));
    assert!(output.contains("/Ascent"));
    assert!(output.contains("/Descent"));
    assert!(output.contains("/CapHeight"));
    assert!(output.contains("/StemV"));
    assert!(output.contains("/FontFile2"));
}

#[test]
fn tounicode_cmap_present() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let font_ref = doc.load_font_bytes(DEJAVU_SANS.to_vec()).unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_text_styled(
        "Hello",
        72.0,
        720.0,
        &TextStyle {
            font: font_ref,
            font_size: 12.0,
        },
    );
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    assert!(output.contains("/ToUnicode"));
    assert!(output.contains("begincmap"));
    assert!(output.contains("endcmap"));
    assert!(output.contains("beginbfchar"));
    assert!(output.contains("endbfchar"));
    assert!(output.contains("/CMapName /Adobe-Identity-UCS"));
}

#[test]
fn w_array_present() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let font_ref = doc.load_font_bytes(DEJAVU_SANS.to_vec()).unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_text_styled(
        "Hi",
        72.0,
        720.0,
        &TextStyle {
            font: font_ref,
            font_size: 12.0,
        },
    );
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    // /W array should be present in CIDFont dict
    assert!(output.contains("/W ["), "Missing /W array in CIDFont");
}

#[test]
fn font_file_embedded() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let font_ref = doc.load_font_bytes(DEJAVU_SANS.to_vec()).unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_text_styled(
        "X",
        72.0,
        720.0,
        &TextStyle {
            font: font_ref,
            font_size: 12.0,
        },
    );
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    // The raw TTF data starts with a specific 4-byte signature
    // (0x00010000 for TrueType)
    let ttf_header = &[0x00, 0x01, 0x00, 0x00];
    assert!(
        contains(&bytes, ttf_header),
        "Embedded font file should contain TTF header"
    );
}

#[test]
fn load_font_file_from_path() {
    let path = "tests/fixtures/DejaVuSans.ttf";
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    let font_ref = doc.load_font_file(path).unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_text_styled(
        "From file",
        72.0,
        720.0,
        &TextStyle {
            font: font_ref,
            font_size: 12.0,
        },
    );
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(bytes.starts_with(b"%PDF-1.7\n"));
    let output = String::from_utf8_lossy(&bytes);
    assert!(output.contains("/Subtype /Type0"));
}

#[test]
fn multiple_truetype_fonts() {
    let mut doc = PdfDocument::new(Vec::<u8>::new()).unwrap();
    // Load the same font data twice to simulate two fonts
    let font1 = doc.load_font_bytes(DEJAVU_SANS.to_vec()).unwrap();
    let font2 = doc.load_font_bytes(DEJAVU_SANS.to_vec()).unwrap();

    assert_ne!(font1, font2);

    doc.begin_page(612.0, 792.0);
    doc.place_text_styled(
        "Font One",
        72.0,
        720.0,
        &TextStyle {
            font: font1,
            font_size: 12.0,
        },
    );
    doc.place_text_styled(
        "Font Two",
        72.0,
        700.0,
        &TextStyle {
            font: font2,
            font_size: 14.0,
        },
    );
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    // Two separate Type0 fonts
    let type0_count = output.matches("/Subtype /Type0").count();
    assert_eq!(type0_count, 2);

    // Both font resources on the page
    assert!(output.contains("/F15"));
    assert!(output.contains("/F16"));
}
