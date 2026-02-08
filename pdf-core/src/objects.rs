/// Object identifier: (object_number, generation_number).
/// Generation is always 0 for new documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjId(pub u32, pub u16);

/// Represents PDF object types per PDF 32000-1:2008 Section 7.3.
#[derive(Debug, Clone)]
pub enum PdfObject {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    /// PDF name object (stored without the leading `/`).
    Name(String),
    /// PDF literal string (stored without the enclosing parens).
    LiteralString(String),
    Array(Vec<PdfObject>),
    /// Key-value pairs. Uses Vec for deterministic output order.
    Dictionary(Vec<(String, PdfObject)>),
    Stream {
        dict: Vec<(String, PdfObject)>,
        data: Vec<u8>,
    },
    Reference(ObjId),
}

impl PdfObject {
    pub fn name(s: &str) -> Self {
        PdfObject::Name(s.to_string())
    }

    pub fn literal_string(s: &str) -> Self {
        PdfObject::LiteralString(s.to_string())
    }

    pub fn reference(obj_num: u32, gen: u16) -> Self {
        PdfObject::Reference(ObjId(obj_num, gen))
    }

    pub fn array(items: Vec<PdfObject>) -> Self {
        PdfObject::Array(items)
    }

    pub fn dict(entries: Vec<(&str, PdfObject)>) -> Self {
        PdfObject::Dictionary(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    pub fn stream(
        dict_entries: Vec<(&str, PdfObject)>,
        data: Vec<u8>,
    ) -> Self {
        PdfObject::Stream {
            dict: dict_entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let obj = PdfObject::array(vec![
            PdfObject::reference(3, 0),
            PdfObject::reference(6, 0),
        ]);
        match obj {
            PdfObject::Array(items) => assert_eq!(items.len(), 2),
            _ => panic!("expected Array"),
        }
    }

    #[test]
    fn stream_constructor() {
        let data = b"BT /F1 12 Tf ET".to_vec();
        let obj = PdfObject::stream(
            vec![("Filter", PdfObject::name("None"))],
            data.clone(),
        );
        match obj {
            PdfObject::Stream { dict, data: d } => {
                assert_eq!(dict.len(), 1);
                assert_eq!(d, data);
            }
            _ => panic!("expected Stream"),
        }
    }
}
