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

    pub fn stream(dict_entries: Vec<(&str, PdfObject)>, data: Vec<u8>) -> Self {
        PdfObject::Stream {
            dict: dict_entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            data,
        }
    }
}
