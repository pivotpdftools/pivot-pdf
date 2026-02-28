use pdf_core::{PdfDocument, PdfReadError, PdfReader};

/// Helper: create a PDF with `n` blank pages and return the raw bytes.
fn make_pdf(n: usize) -> Vec<u8> {
    let mut doc = PdfDocument::new(Vec::new()).unwrap();
    for _ in 0..n {
        doc.begin_page(612.0, 792.0);
        doc.end_page().unwrap();
    }
    doc.end_document().unwrap()
}

// --- Task 2 + 5: PdfReader shell with from_bytes ---

#[test]
fn reader_from_bytes_returns_reader() {
    let bytes = make_pdf(1);
    let reader = PdfReader::from_bytes(bytes);
    assert!(reader.is_ok());
}

// --- Task 5: page_count ---

#[test]
fn reader_one_page() {
    let bytes = make_pdf(1);
    let reader = PdfReader::from_bytes(bytes).unwrap();
    assert_eq!(reader.page_count(), 1);
}

#[test]
fn reader_three_pages() {
    let bytes = make_pdf(3);
    let reader = PdfReader::from_bytes(bytes).unwrap();
    assert_eq!(reader.page_count(), 3);
}

#[test]
fn reader_zero_pages() {
    let bytes = make_pdf(0);
    let reader = PdfReader::from_bytes(bytes).unwrap();
    assert_eq!(reader.page_count(), 0);
}

#[test]
fn reader_ten_pages() {
    let bytes = make_pdf(10);
    let reader = PdfReader::from_bytes(bytes).unwrap();
    assert_eq!(reader.page_count(), 10);
}

// --- Task 5: pdf_version ---

#[test]
fn reader_pdf_version() {
    let bytes = make_pdf(1);
    let reader = PdfReader::from_bytes(bytes).unwrap();
    assert_eq!(reader.pdf_version(), "1.7");
}

// --- Task 5: open() ---

#[test]
fn reader_open_file() {
    let bytes = make_pdf(2);
    let path = std::env::temp_dir().join("reader_test_open.pdf");
    std::fs::write(&path, &bytes).unwrap();

    let reader = PdfReader::open(&path).unwrap();
    assert_eq!(reader.page_count(), 2);

    std::fs::remove_file(&path).ok();
}

// --- Task 6: error cases ---

#[test]
fn reader_empty_bytes_returns_error() {
    let result = PdfReader::from_bytes(vec![]);
    assert!(matches!(result, Err(PdfReadError::NotAPdf)));
}

#[test]
fn reader_garbage_bytes_returns_error() {
    let result = PdfReader::from_bytes(b"this is not a pdf at all".to_vec());
    assert!(matches!(result, Err(PdfReadError::NotAPdf)));
}

#[test]
fn reader_truncated_pdf_returns_error() {
    // Only the header, no body or xref
    let result = PdfReader::from_bytes(b"%PDF-1.7\n".to_vec());
    assert!(result.is_err());
}
