use pivot_pdf::{DocumentOptions, Origin, PdfDocument, Rect};

fn field_rect() -> Rect {
    Rect {
        x: 72.0,
        y: 700.0,
        width: 200.0,
        height: 18.0,
    }
}

// -------------------------------------------------------
// Task 2: add_text_field API and error cases
// -------------------------------------------------------

#[test]
fn add_text_field_returns_ok() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.add_text_field("full_name", field_rect()).unwrap();
    doc.end_page().unwrap();
    doc.end_document().unwrap();
}

#[test]
fn add_text_field_no_active_page_returns_error() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    let result = doc.add_text_field("full_name", field_rect());
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("no active page"),
        "expected 'no active page' in error: {}",
        err
    );
}

#[test]
fn add_text_field_duplicate_name_returns_error() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.add_text_field("full_name", field_rect()).unwrap();
    let result = doc.add_text_field("full_name", field_rect());
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("full_name") && err.contains("duplicate"),
        "expected duplicate error mentioning field name: {}",
        err
    );
    doc.end_page().unwrap();
    doc.end_document().unwrap();
}

#[test]
fn add_text_field_duplicate_across_pages_returns_error() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.add_text_field("email", field_rect()).unwrap();
    doc.end_page().unwrap();
    // Second page — same name should fail
    doc.begin_page(612.0, 792.0);
    let result = doc.add_text_field("email", field_rect());
    assert!(result.is_err());
    doc.end_page().unwrap();
    doc.end_document().unwrap();
}

// -------------------------------------------------------
// Task 3 & 4: widget annotations appear in PDF output
// -------------------------------------------------------

#[test]
fn text_field_produces_widget_annotation() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.add_text_field("full_name", field_rect()).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    assert!(
        output.contains("/Subtype /Widget"),
        "should contain Widget annotation"
    );
    assert!(output.contains("/FT /Tx"), "should declare text field type");
    assert!(output.contains("/Annots"), "page dict should have /Annots");
    assert!(output.contains("(full_name)"), "should contain field name");
}

#[test]
fn text_field_rect_appears_in_output() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.add_text_field(
        "email",
        Rect {
            x: 72.0,
            y: 700.0,
            width: 200.0,
            height: 18.0,
        },
    )
    .unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    // Rect in PDF is [x y x+width y+height] i.e. [72 700 272 718]
    assert!(output.contains("/Rect"), "widget should contain /Rect key");
}

#[test]
fn multiple_fields_on_same_page() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.add_text_field(
        "first_name",
        Rect {
            x: 72.0,
            y: 700.0,
            width: 200.0,
            height: 18.0,
        },
    )
    .unwrap();
    doc.add_text_field(
        "last_name",
        Rect {
            x: 72.0,
            y: 670.0,
            width: 200.0,
            height: 18.0,
        },
    )
    .unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    assert!(output.contains("(first_name)"));
    assert!(output.contains("(last_name)"));
}

#[test]
fn fields_on_different_pages() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.add_text_field("name", field_rect()).unwrap();
    doc.end_page().unwrap();
    doc.begin_page(612.0, 792.0);
    doc.add_text_field("address", field_rect()).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    assert!(output.contains("(name)"));
    assert!(output.contains("(address)"));
}

// -------------------------------------------------------
// Task 5: AcroForm catalog entry
// -------------------------------------------------------

#[test]
fn acroform_appears_in_catalog() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.add_text_field("field1", field_rect()).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    assert!(
        output.contains("/AcroForm"),
        "catalog should contain /AcroForm"
    );
    assert!(
        output.contains("/Fields"),
        "AcroForm should contain /Fields"
    );
    assert!(
        output.contains("/NeedAppearances true"),
        "AcroForm should have NeedAppearances"
    );
    assert!(
        output.contains("/DA"),
        "AcroForm should have default appearance /DA"
    );
}

#[test]
fn no_acroform_when_no_fields() {
    let mut doc = PdfDocument::new(Vec::<u8>::new(), DocumentOptions::default()).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.place_text("No fields here", 72.0, 720.0);
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);

    assert!(
        !output.contains("/AcroForm"),
        "catalog should not contain /AcroForm when no fields"
    );
}

#[test]
fn top_left_origin_transforms_add_text_field() {
    // Page 612x792. Field rect (72, 72, 200, 18) with TopLeft origin:
    // PDF Rect = [x_ll, y_ll, x_ur, y_ur]
    //           = [72, page_h - y - h, 72 + 200, page_h - y]
    //           = [72, 792 - 72 - 18, 272, 792 - 72]
    //           = [72, 702, 272, 720]
    let opts = DocumentOptions {
        origin: Origin::TopLeft,
    };
    let mut doc = PdfDocument::new(Vec::<u8>::new(), opts).unwrap();
    doc.begin_page(612.0, 792.0);
    doc.add_text_field(
        "email",
        Rect {
            x: 72.0,
            y: 72.0,
            width: 200.0,
            height: 18.0,
        },
    )
    .unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    // Widget /Rect should be [72 702 272 720] in PDF space (values may have .0 suffix).
    assert!(
        output.contains("72.0 702.0 272.0 720.0") || output.contains("72 702 272 720"),
        "field rect should be transformed: {}",
        output
            .lines()
            .find(|l| l.contains("Rect"))
            .unwrap_or("no Rect")
    );
}
