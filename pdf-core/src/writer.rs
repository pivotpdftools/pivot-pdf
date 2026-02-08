use std::io::{self, Write};

use crate::objects::{ObjId, PdfObject};

/// Low-level PDF binary writer. Serializes PDF objects to any
/// `Write` target while tracking byte offsets for the xref table.
pub struct PdfWriter<W: Write> {
    writer: W,
    offset: usize,
    xref_entries: Vec<(u32, usize)>,
}

impl<W: Write> PdfWriter<W> {
    pub fn new(writer: W) -> Self {
        PdfWriter {
            writer,
            offset: 0,
            xref_entries: Vec::new(),
        }
    }

    /// Write raw bytes, tracking the byte offset.
    fn write_bytes(&mut self, data: &[u8]) -> io::Result<()> {
        self.writer.write_all(data)?;
        self.offset += data.len();
        Ok(())
    }

    /// Write a formatted string, tracking the byte offset.
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.write_bytes(s.as_bytes())
    }

    /// Write the PDF 1.7 header and binary comment.
    pub fn write_header(&mut self) -> io::Result<()> {
        self.write_str("%PDF-1.7\n")?;
        // Binary comment: 4 bytes >= 128 for binary detection.
        self.write_bytes(b"%\xe2\xe3\xcf\xd3\n")?;
        Ok(())
    }

    /// Write an indirect object, recording its byte offset for xref.
    pub fn write_object(
        &mut self,
        id: ObjId,
        obj: &PdfObject,
    ) -> io::Result<()> {
        self.xref_entries.push((id.0, self.offset));
        self.write_str(
            &format!("{} {} obj\n", id.0, id.1),
        )?;
        self.write_pdf_object(obj)?;
        self.write_str("\nendobj\n")?;
        Ok(())
    }

    /// Serialize a PdfObject to its PDF text representation.
    fn write_pdf_object(
        &mut self,
        obj: &PdfObject,
    ) -> io::Result<()> {
        match obj {
            PdfObject::Null => self.write_str("null"),
            PdfObject::Boolean(b) => {
                if *b {
                    self.write_str("true")
                } else {
                    self.write_str("false")
                }
            }
            PdfObject::Integer(n) => {
                self.write_str(&n.to_string())
            }
            PdfObject::Real(f) => {
                let s = format_real(*f);
                self.write_str(&s)
            }
            PdfObject::Name(name) => {
                self.write_str("/")?;
                self.write_str(name)
            }
            PdfObject::LiteralString(s) => {
                self.write_str("(")?;
                self.write_str(&escape_pdf_string(s))?;
                self.write_str(")")
            }
            PdfObject::Array(items) => {
                self.write_str("[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.write_str(" ")?;
                    }
                    self.write_pdf_object(item)?;
                }
                self.write_str("]")
            }
            PdfObject::Dictionary(entries) => {
                self.write_str("<<")?;
                for (key, val) in entries {
                    self.write_str(" /")?;
                    self.write_str(key)?;
                    self.write_str(" ")?;
                    self.write_pdf_object(val)?;
                }
                self.write_str(" >>")
            }
            PdfObject::Stream { dict, data } => {
                self.write_str("<<")?;
                for (key, val) in dict {
                    self.write_str(" /")?;
                    self.write_str(key)?;
                    self.write_str(" ")?;
                    self.write_pdf_object(val)?;
                }
                self.write_str(" /Length ")?;
                self.write_str(&data.len().to_string())?;
                self.write_str(" >>\nstream\n")?;
                self.write_bytes(data)?;
                self.write_str("\nendstream")
            }
            PdfObject::Reference(id) => {
                self.write_str(
                    &format!("{} {} R", id.0, id.1),
                )
            }
        }
    }

    /// Current byte offset in the output.
    pub fn current_offset(&self) -> usize {
        self.offset
    }

    /// Write xref table, trailer, startxref, and %%EOF.
    pub fn write_xref_and_trailer(
        &mut self,
        root_id: ObjId,
        info_id: Option<ObjId>,
    ) -> io::Result<()> {
        let xref_offset = self.offset;

        // Sort xref entries by object number.
        self.xref_entries
            .sort_by_key(|&(num, _)| num);

        let max_obj = self
            .xref_entries
            .last()
            .map(|&(num, _)| num)
            .unwrap_or(0);
        let size = max_obj + 1;

        self.write_str("xref\n")?;
        self.write_str(&format!("0 {}\n", size))?;

        // Object 0: free entry head (exactly 20 bytes).
        self.write_bytes(
            b"0000000000 65535 f\r\n",
        )?;

        // Build a map for quick lookup.
        let mut offset_map =
            std::collections::HashMap::new();
        for &(num, off) in &self.xref_entries {
            offset_map.insert(num, off);
        }

        // Write entries for objects 1..max_obj.
        for obj_num in 1..size {
            if let Some(&off) = offset_map.get(&obj_num) {
                let entry = format!(
                    "{:010} {:05} n\r\n",
                    off, 0
                );
                self.write_bytes(entry.as_bytes())?;
            } else {
                // Free entry for gaps.
                self.write_bytes(
                    b"0000000000 00000 f\r\n",
                )?;
            }
        }

        // Trailer.
        self.write_str("trailer\n")?;
        self.write_str(&format!(
            "<< /Size {} /Root {} {} R",
            size, root_id.0, root_id.1,
        ))?;
        if let Some(info) = info_id {
            self.write_str(&format!(
                " /Info {} {} R",
                info.0, info.1,
            ))?;
        }
        self.write_str(" >>\n")?;

        self.write_str("startxref\n")?;
        self.write_str(&format!("{}\n", xref_offset))?;
        self.write_str("%%EOF\n")?;

        Ok(())
    }

    /// Return the inner writer, consuming this PdfWriter.
    pub fn into_inner(self) -> W {
        self.writer
    }
}

/// Escape special characters in a PDF literal string.
pub fn escape_pdf_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '(' => result.push_str("\\("),
            ')' => result.push_str("\\)"),
            _ => result.push(c),
        }
    }
    result
}

/// Format a float for PDF output: no trailing zeros,
/// no scientific notation.
fn format_real(f: f64) -> String {
    if f == f.floor() && f.abs() < 1e15 {
        format!("{:.1}", f)
    } else {
        let s = format!("{:.6}", f);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}
