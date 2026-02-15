use pdf_core::objects::{ObjId, PdfObject};

#[test]
fn obj_id_equality() {
    let a = ObjId(1, 0);
    let b = ObjId(1, 0);
    let c = ObjId(2, 0);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn name_constructor() {
    let obj = PdfObject::name("Type");
    match obj {
        PdfObject::Name(s) => assert_eq!(s, "Type"),
        _ => panic!("expected Name"),
    }
}

#[test]
fn literal_string_constructor() {
    let obj = PdfObject::literal_string("Hello");
    match obj {
        PdfObject::LiteralString(s) => assert_eq!(s, "Hello"),
        _ => panic!("expected LiteralString"),
    }
}

#[test]
fn reference_constructor() {
    let obj = PdfObject::reference(5, 0);
    match obj {
        PdfObject::Reference(id) => {
            assert_eq!(id, ObjId(5, 0));
        }
        _ => panic!("expected Reference"),
    }
}

#[test]
fn dict_constructor() {
    let obj = PdfObject::dict(vec![
        ("Type", PdfObject::name("Catalog")),
        ("Pages", PdfObject::reference(2, 0)),
    ]);
    match obj {
        PdfObject::Dictionary(entries) => {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, "Type");
            assert_eq!(entries[1].0, "Pages");
        }
        _ => panic!("expected Dictionary"),
    }
}

#[test]
fn array_constructor() {
    let obj = PdfObject::array(vec![PdfObject::reference(3, 0), PdfObject::reference(6, 0)]);
    match obj {
        PdfObject::Array(items) => assert_eq!(items.len(), 2),
        _ => panic!("expected Array"),
    }
}

#[test]
fn stream_constructor() {
    let data = b"BT /F1 12 Tf ET".to_vec();
    let obj = PdfObject::stream(vec![("Filter", PdfObject::name("None"))], data.clone());
    match obj {
        PdfObject::Stream { dict, data: d } => {
            assert_eq!(dict.len(), 1);
            assert_eq!(d, data);
        }
        _ => panic!("expected Stream"),
    }
}
