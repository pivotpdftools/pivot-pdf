/// Object identifier: (object_number, generation_number).
/// Generation is always 0 for new documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjId(pub u32, pub u16);

/// Represents PDF object types per PDF 32000-1:2008 Section 7.3.
#[derive(Debug, Clone)]
pub enum PdfObject {
    /// PDF null object.
    Null,
    /// PDF boolean object.
    Boolean(bool),
    /// PDF integer object.
    Integer(i64),
    /// PDF real (floating-point) object.
    Real(f64),
    /// PDF name object (stored without the leading `/`).
    Name(String),
    /// PDF literal string (stored without the enclosing parens).
    LiteralString(String),
    /// PDF array object.
    Array(Vec<PdfObject>),
    /// Key-value pairs. Uses Vec for deterministic output order.
    Dictionary(Vec<(String, PdfObject)>),
    /// PDF stream object: a dictionary plus a byte sequence.
    Stream {
        /// Stream dictionary entries (excluding `/Length`, which is added on write).
        dict: Vec<(String, PdfObject)>,
        /// Raw stream bytes.
        data: Vec<u8>,
    },
    /// Indirect object reference (`N G R`).
    Reference(ObjId),
}

impl PdfObject {
    /// Construct a `Name` object from a string slice (without the leading `/`).
    pub fn name(s: &str) -> Self {
        PdfObject::Name(s.to_string())
    }

    /// Construct a `LiteralString` object from a string slice.
    pub fn literal_string(s: &str) -> Self {
        PdfObject::LiteralString(s.to_string())
    }

    /// Construct a `Reference` object from an object number and generation number.
    pub fn reference(obj_num: u32, gen: u16) -> Self {
        PdfObject::Reference(ObjId(obj_num, gen))
    }

    /// Construct an `Array` object from a vector of items.
    pub fn array(items: Vec<PdfObject>) -> Self {
        PdfObject::Array(items)
    }

    /// Construct a `Dictionary` object from a slice of `(key, value)` pairs.
    pub fn dict(entries: Vec<(&str, PdfObject)>) -> Self {
        PdfObject::Dictionary(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    /// Construct a `Stream` object from a dictionary and raw byte data.
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
