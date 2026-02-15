use pdf_core::objects::{ObjId, PdfObject};
use pdf_core::writer::{escape_pdf_string, PdfWriter};

#[test]
fn header_bytes() {
    let mut buf = Vec::new();
    let mut w = PdfWriter::new(&mut buf);
    w.write_header().unwrap();
    let output = String::from_utf8_lossy(&buf);
    assert!(output.starts_with("%PDF-1.7\n"));
    assert_eq!(buf[9], b'%');
    // Binary bytes >= 128.
    assert!(buf[10] >= 128);
    assert!(buf[11] >= 128);
    assert!(buf[12] >= 128);
    assert!(buf[13] >= 128);
}

#[test]
fn write_name_object() {
    let mut buf = Vec::new();
    let mut w = PdfWriter::new(&mut buf);
    let obj = PdfObject::name("Type");
    w.write_object(ObjId(1, 0), &obj).unwrap();
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("1 0 obj"));
    assert!(output.contains("/Type"));
    assert!(output.contains("endobj"));
}

#[test]
fn write_dictionary() {
    let mut buf = Vec::new();
    let mut w = PdfWriter::new(&mut buf);
    let obj = PdfObject::dict(vec![
        ("Type", PdfObject::name("Catalog")),
        ("Pages", PdfObject::reference(2, 0)),
    ]);
    w.write_object(ObjId(1, 0), &obj).unwrap();
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("<< /Type /Catalog /Pages 2 0 R >>"));
}

#[test]
fn write_array() {
    let mut buf = Vec::new();
    let mut w = PdfWriter::new(&mut buf);
    let obj = PdfObject::array(vec![PdfObject::reference(3, 0), PdfObject::reference(6, 0)]);
    w.write_object(ObjId(1, 0), &obj).unwrap();
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("[3 0 R 6 0 R]"));
}

#[test]
fn write_stream() {
    let mut buf = Vec::new();
    let mut w = PdfWriter::new(&mut buf);
    let data = b"BT /F1 12 Tf ET".to_vec();
    let obj = PdfObject::stream(vec![], data);
    w.write_object(ObjId(4, 0), &obj).unwrap();
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("/Length 15"));
    assert!(output.contains("stream\n"));
    assert!(output.contains("BT /F1 12 Tf ET"));
    assert!(output.contains("\nendstream"));
}

#[test]
fn write_literal_string_escaped() {
    let mut buf = Vec::new();
    let mut w = PdfWriter::new(&mut buf);
    let obj = PdfObject::literal_string("a(b)c\\d");
    w.write_object(ObjId(1, 0), &obj).unwrap();
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("(a\\(b\\)c\\\\d)"));
}

#[test]
fn xref_entry_is_20_bytes() {
    let mut buf = Vec::new();
    let mut w = PdfWriter::new(&mut buf);
    w.write_header().unwrap();
    let obj = PdfObject::name("Catalog");
    w.write_object(ObjId(1, 0), &obj).unwrap();
    w.write_xref_and_trailer(ObjId(1, 0), None).unwrap();

    // Search raw bytes for xref marker.
    let xref_marker = b"xref\n";
    let xref_pos = buf
        .windows(xref_marker.len())
        .position(|w| w == xref_marker)
        .unwrap();
    // After "xref\n0 2\n" comes the entries.
    let entries_start = xref_pos + b"xref\n0 2\n".len();
    let entries = &buf[entries_start..];
    // First entry (obj 0): exactly 20 bytes.
    assert_eq!(entries[19], b'\n');
    assert_eq!(entries[18], b'\r');
    // Second entry (obj 1): next 20 bytes.
    assert_eq!(entries[39], b'\n');
    assert_eq!(entries[38], b'\r');
}

#[test]
fn trailer_has_required_keys() {
    let mut buf = Vec::new();
    let mut w = PdfWriter::new(&mut buf);
    w.write_header().unwrap();
    let cat = PdfObject::name("Catalog");
    w.write_object(ObjId(1, 0), &cat).unwrap();
    let info = PdfObject::dict(vec![("Creator", PdfObject::literal_string("test"))]);
    w.write_object(ObjId(2, 0), &info).unwrap();
    w.write_xref_and_trailer(ObjId(1, 0), Some(ObjId(2, 0)))
        .unwrap();

    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("/Size 3"));
    assert!(output.contains("/Root 1 0 R"));
    assert!(output.contains("/Info 2 0 R"));
    assert!(output.contains("startxref"));
    assert!(output.ends_with("%%EOF\n"));
}

#[test]
fn real_value_formatting() {
    let cases: Vec<(f64, &str)> = vec![
        (612.0, "612.0"),
        (792.0, "792.0"),
        (0.0, "0.0"),
        (12.5, "12.5"),
    ];
    for (val, expected) in cases {
        let mut buf = Vec::new();
        let mut w = PdfWriter::new(&mut buf);
        w.write_object(ObjId(1, 0), &PdfObject::Real(val)).unwrap();
        let output = String::from_utf8_lossy(&buf);
        assert!(
            output.contains(expected),
            "Real({}) should contain '{}', got: {}",
            val,
            expected,
            output
        );
    }
}

#[test]
fn escape_special_chars() {
    assert_eq!(escape_pdf_string("hello"), "hello");
    assert_eq!(escape_pdf_string("a(b)c"), "a\\(b\\)c");
    assert_eq!(escape_pdf_string("back\\slash"), "back\\\\slash");
}
