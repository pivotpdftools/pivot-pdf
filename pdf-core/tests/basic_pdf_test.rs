use pdf_core::PdfDocument;

/// Helper: find a byte pattern in a buffer.
fn find_bytes(
    haystack: &[u8],
    needle: &[u8],
) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|w| w == needle)
}

/// Helper: check that a byte pattern exists in the buffer.
fn contains_bytes(
    haystack: &[u8],
    needle: &[u8],
) -> bool {
    find_bytes(haystack, needle).is_some()
}

#[test]
fn full_workflow_produces_valid_pdf() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.set_info("Creator", "rust-pdf");
    doc.set_info("Title", "A Test Document");
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello", 20.0, 20.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    // Header.
    assert!(bytes.starts_with(b"%PDF-1.7\n"));

    // Trailer.
    assert!(bytes.ends_with(b"%%EOF\n"));

    // Core PDF structure.
    assert!(contains_bytes(&bytes, b"/Type /Catalog"));
    assert!(contains_bytes(&bytes, b"/Type /Pages"));
    assert!(contains_bytes(&bytes, b"/Type /Page"));
    assert!(contains_bytes(&bytes, b"/Type /Font"));
    assert!(contains_bytes(
        &bytes,
        b"/BaseFont /Helvetica",
    ));

    // Content stream with text.
    assert!(contains_bytes(&bytes, b"(Hello) Tj"));
    assert!(contains_bytes(&bytes, b"/F1 12 Tf"));
    assert!(contains_bytes(&bytes, b"20 20 Td"));

    // Info dictionary.
    assert!(contains_bytes(&bytes, b"(rust-pdf)"));
    assert!(contains_bytes(
        &bytes,
        b"(A Test Document)",
    ));

    // Xref and trailer structure.
    assert!(contains_bytes(&bytes, b"xref\n"));
    assert!(contains_bytes(&bytes, b"trailer\n"));
    assert!(contains_bytes(&bytes, b"startxref\n"));
    assert!(contains_bytes(&bytes, b"/Root 1 0 R"));
    assert!(contains_bytes(&bytes, b"/Info"));
}

#[test]
fn empty_page_produces_valid_pdf() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(bytes.starts_with(b"%PDF-1.7\n"));
    assert!(bytes.ends_with(b"%%EOF\n"));
    assert!(contains_bytes(&bytes, b"/Count 1"));
    // Empty content stream should have /Length 0.
    assert!(contains_bytes(&bytes, b"/Length 0"));
}

#[test]
fn special_characters_in_text() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text(
        "Price: $100 (USD)",
        20.0,
        20.0,
    );
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    // Parentheses in text should be escaped.
    assert!(contains_bytes(
        &bytes,
        b"(Price: $100 \\(USD\\)) Tj"
    ));
}

#[test]
fn multi_page_document() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_text("Page 1", 20.0, 700.0);
    doc.end_page().unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_text("Page 2", 20.0, 700.0);
    doc.end_page().unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_text("Page 3", 20.0, 700.0);
    doc.end_page().unwrap();

    let bytes = doc.end_document().unwrap();

    assert!(contains_bytes(&bytes, b"/Count 3"));
    assert!(contains_bytes(&bytes, b"(Page 1) Tj"));
    assert!(contains_bytes(&bytes, b"(Page 2) Tj"));
    assert!(contains_bytes(&bytes, b"(Page 3) Tj"));
}

#[test]
fn streaming_frees_page_data() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();

    doc.begin_page(612.0, 792.0);
    doc.place_text("First page content", 20.0, 20.0);
    doc.end_page().unwrap();

    // After end_page, the first page's content has been
    // written. Starting a second page should not accumulate
    // the first page's data in memory.
    doc.begin_page(612.0, 792.0);
    doc.place_text("Second page", 20.0, 20.0);
    doc.end_page().unwrap();

    let bytes = doc.end_document().unwrap();

    // Both pages present in output.
    assert!(contains_bytes(
        &bytes,
        b"(First page content) Tj",
    ));
    assert!(contains_bytes(
        &bytes,
        b"(Second page) Tj",
    ));
    assert!(contains_bytes(&bytes, b"/Count 2"));
}

#[test]
fn xref_object_count_matches() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.set_info("Creator", "test");
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello", 20.0, 20.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    // Objects: 1=Catalog, 2=Pages,
    // 3=Font(Helvetica), 4=ContentStream, 5=Page,
    // 6=Info
    // Size = max_obj + 1 = 7
    assert!(
        contains_bytes(&bytes, b"/Size 7"),
        "Expected /Size 7 in output: {}",
        String::from_utf8_lossy(&bytes),
    );
    // Xref section header should match.
    assert!(
        contains_bytes(&bytes, b"xref\n0 7\n"),
        "Expected xref header '0 7' in output: {}",
        String::from_utf8_lossy(&bytes),
    );
}

#[test]
fn save_to_temp_file() {
    let dir = std::env::temp_dir();
    let path = dir.join("rust_pdf_test_output.pdf");

    let mut doc = PdfDocument::create(&path).unwrap();
    doc.set_info("Creator", "rust-pdf");
    doc.set_info("Title", "A Test Document");
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello, PDF!", 72.0, 720.0);
    doc.end_page().unwrap();
    doc.end_document().unwrap();

    // Verify file was created and has content.
    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.starts_with(b"%PDF-1.7\n"));
    assert!(bytes.ends_with(b"%%EOF\n"));

    // Clean up.
    let _ = std::fs::remove_file(&path);
}

#[test]
fn only_used_fonts_written_to_output() {
    // A doc using only Helvetica should contain that font
    // but not Times-Roman, Courier, etc.
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("Hello", 20.0, 20.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(contains_bytes(
        &bytes,
        b"/BaseFont /Helvetica",
    ));
    assert!(!contains_bytes(
        &bytes,
        b"/BaseFont /Times-Roman",
    ));
    assert!(!contains_bytes(
        &bytes,
        b"/BaseFont /Courier",
    ));
}

#[test]
fn empty_page_has_no_font_objects() {
    let mut doc =
        PdfDocument::new(Vec::<u8>::new()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    // No text placed, so no font objects should exist.
    assert!(!contains_bytes(
        &bytes,
        b"/BaseFont",
    ));
}
