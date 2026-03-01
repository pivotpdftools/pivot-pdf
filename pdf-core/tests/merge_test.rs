use pdf_core::{merge_pdfs, MergeOptions, PdfDocument, PdfReader};

/// Helper: create a PDF with `n` blank pages and return the raw bytes.
fn make_pdf(n: usize) -> Vec<u8> {
    let mut doc = PdfDocument::new(Vec::new()).unwrap();
    for _ in 0..n {
        doc.begin_page(612.0, 792.0);
        doc.end_page().unwrap();
    }
    doc.end_document().unwrap()
}

/// Write bytes to a temp file and return the path.
fn write_temp(name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

// ── MergeOptions ─────────────────────────────────────────────────────────────

#[test]
fn merge_options_default_flatten_forms_is_false() {
    let opts = MergeOptions::default();
    assert!(!opts.flatten_forms);
}

// ── merge_pdfs: flatten_forms returns NotSupported ────────────────────────────

#[test]
fn merge_flatten_forms_returns_not_supported() {
    let a = write_temp("merge_not_supported_a.pdf", &make_pdf(1));
    let out = std::env::temp_dir().join("merge_not_supported_out.pdf");
    let opts = MergeOptions {
        flatten_forms: true,
    };
    let result = merge_pdfs(&[&a], &out, opts);
    assert!(
        matches!(result, Err(pdf_core::PdfMergeError::NotSupported)),
        "expected NotSupported, got {:?}",
        result
    );
}

// ── merge_pdfs: basic page counts ─────────────────────────────────────────────

#[test]
fn merge_two_single_page_pdfs_produces_two_pages() {
    let a = write_temp("merge_2x1_a.pdf", &make_pdf(1));
    let b = write_temp("merge_2x1_b.pdf", &make_pdf(1));
    let out = std::env::temp_dir().join("merge_2x1_out.pdf");

    merge_pdfs(&[&a, &b], &out, MergeOptions::default()).unwrap();

    let reader = PdfReader::open(&out).unwrap();
    assert_eq!(reader.page_count(), 2);
}

#[test]
fn merge_three_multi_page_pdfs_produces_correct_total() {
    let a = write_temp("merge_3mp_a.pdf", &make_pdf(2));
    let b = write_temp("merge_3mp_b.pdf", &make_pdf(3));
    let c = write_temp("merge_3mp_c.pdf", &make_pdf(1));
    let out = std::env::temp_dir().join("merge_3mp_out.pdf");

    merge_pdfs(&[&a, &b, &c], &out, MergeOptions::default()).unwrap();

    let reader = PdfReader::open(&out).unwrap();
    assert_eq!(reader.page_count(), 6);
}

#[test]
fn merge_output_is_readable_by_pdf_reader() {
    let a = write_temp("merge_readable_a.pdf", &make_pdf(2));
    let b = write_temp("merge_readable_b.pdf", &make_pdf(2));
    let out = std::env::temp_dir().join("merge_readable_out.pdf");

    let result = merge_pdfs(&[&a, &b], &out, MergeOptions::default());
    assert!(result.is_ok(), "merge_pdfs failed: {:?}", result);

    let reader = PdfReader::open(&out);
    assert!(reader.is_ok(), "PdfReader could not open merged PDF");
    assert_eq!(reader.unwrap().page_count(), 4);
}

#[test]
fn merge_single_source_preserves_page_count() {
    let a = write_temp("merge_single_a.pdf", &make_pdf(3));
    let out = std::env::temp_dir().join("merge_single_out.pdf");

    merge_pdfs(&[&a], &out, MergeOptions::default()).unwrap();

    let reader = PdfReader::open(&out).unwrap();
    assert_eq!(reader.page_count(), 3);
}
