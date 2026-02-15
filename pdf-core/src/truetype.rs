use std::collections::{BTreeMap, BTreeSet};

use crate::objects::PdfObject;

/// A loaded TrueType font with parsed metrics and glyph data.
pub struct TrueTypeFont {
    #[allow(dead_code)] // reserved for font selection UIs
    pub(crate) name: String,
    pub(crate) postscript_name: String,
    pub(crate) font_data: Vec<u8>,
    pub(crate) units_per_em: u16,
    pub(crate) ascent: i16,
    pub(crate) descent: i16,
    pub(crate) bbox: [i16; 4],
    pub(crate) cap_height: i16,
    pub(crate) italic_angle: f64,
    pub(crate) flags: u32,
    pub(crate) stem_v: i16,
    /// Unicode codepoint -> glyph ID
    pub(crate) cmap: BTreeMap<u32, u16>,
    /// Glyph ID -> advance width in font units
    pub(crate) glyph_widths: BTreeMap<u16, u16>,
    pub(crate) default_width: u16,
    /// Glyph IDs that have been used (for subsetting/W array)
    pub(crate) used_glyphs: BTreeSet<u16>,
    /// Glyph ID -> Unicode codepoint (for ToUnicode CMap)
    pub(crate) glyph_to_unicode: BTreeMap<u16, u32>,
    /// PDF resource name (e.g. "F15")
    pub(crate) pdf_name: String,
}

impl TrueTypeFont {
    /// Parse a TrueType font from raw .ttf bytes.
    pub fn from_bytes(data: Vec<u8>, font_num: u32) -> Result<Self, String> {
        let face =
            ttf_parser::Face::parse(&data, 0).map_err(|e| format!("Failed to parse TTF: {}", e))?;

        let units_per_em = face.units_per_em();
        let ascent = face.ascender();
        let descent = face.descender();
        let bbox = face.global_bounding_box();
        let cap_height = face.capital_height().unwrap_or(ascent);
        let italic_angle = face.italic_angle() as f64;

        let flags = compute_flags(&face);
        let stem_v = estimate_stem_v(&face);

        let name = extract_name(&face).unwrap_or_else(|| "Unknown".to_string());
        let postscript_name =
            extract_postscript_name(&face).unwrap_or_else(|| name.replace(' ', ""));

        // Build cmap: Unicode -> GlyphID
        let mut cmap = BTreeMap::new();
        let mut glyph_to_unicode = BTreeMap::new();

        let subtables = face
            .tables()
            .cmap
            .ok_or("Font has no cmap table".to_string())?;
        for subtable in subtables.subtables {
            if !subtable.is_unicode() {
                continue;
            }
            subtable.codepoints(|cp| {
                if let Some(gid) = subtable.glyph_index(cp) {
                    let gid_val = gid.0;
                    cmap.insert(cp, gid_val);
                    glyph_to_unicode.entry(gid_val).or_insert(cp);
                }
            });
        }

        // Build glyph widths from hmtx
        let num_glyphs = face.number_of_glyphs();
        let mut glyph_widths = BTreeMap::new();
        let mut default_width = 0u16;

        for gid in 0..num_glyphs {
            let glyph_id = ttf_parser::GlyphId(gid);
            let width = face.glyph_hor_advance(glyph_id).unwrap_or(0);
            glyph_widths.insert(gid, width);
        }

        // Default width = width of glyph 0 (notdef)
        if let Some(&w) = glyph_widths.get(&0) {
            default_width = w;
        }

        let pdf_name = format!("F{}", font_num);

        Ok(TrueTypeFont {
            name,
            postscript_name,
            font_data: data,
            units_per_em,
            ascent,
            descent,
            bbox: [bbox.x_min, bbox.y_min, bbox.x_max, bbox.y_max],
            cap_height,
            italic_angle,
            flags,
            stem_v,
            cmap,
            glyph_widths,
            default_width,
            used_glyphs: BTreeSet::new(),
            glyph_to_unicode,
            pdf_name,
        })
    }

    /// Scale a raw font unit value to PDF units (1/1000 of text space).
    pub(crate) fn scale_to_pdf(&self, value: i16) -> i64 {
        (value as i64 * 1000) / self.units_per_em as i64
    }

    /// Default width in PDF units (1/1000 of text space).
    pub(crate) fn default_width_pdf(&self) -> i64 {
        (self.default_width as i64 * 1000) / self.units_per_em as i64
    }

    /// Width of a character in PDF units (1/1000 of text space).
    pub fn char_width_pdf(&self, ch: char) -> u16 {
        let gid = self.cmap.get(&(ch as u32)).copied().unwrap_or(0);
        let raw = self
            .glyph_widths
            .get(&gid)
            .copied()
            .unwrap_or(self.default_width);
        ((raw as u32 * 1000) / self.units_per_em as u32) as u16
    }

    /// Measure text width in points.
    pub fn measure_text(&self, text: &str, font_size: f64) -> f64 {
        let total: u32 = text.chars().map(|ch| self.char_width_pdf(ch) as u32).sum();
        total as f64 * font_size / 1000.0
    }

    /// Line height for a given font size using ascent - descent.
    pub fn line_height(&self, font_size: f64) -> f64 {
        let height = (self.ascent as i32 - self.descent as i32) as f64 / self.units_per_em as f64;
        height * font_size
    }

    /// Look up the glyph ID for a character and record it as used.
    pub fn glyph_id(&mut self, ch: char) -> u16 {
        let gid = self.cmap.get(&(ch as u32)).copied().unwrap_or(0);
        self.used_glyphs.insert(gid);
        gid
    }

    /// Encode text as hex glyph IDs: `<00480065006C006C006F>`.
    pub fn encode_text_hex(&mut self, text: &str) -> String {
        let mut hex = String::with_capacity(text.len() * 5 + 2);
        hex.push('<');
        for ch in text.chars() {
            let gid = self.glyph_id(ch);
            hex.push_str(&format!("{:04X}", gid));
        }
        hex.push('>');
        hex
    }

    /// Build the PDF /W array for used glyphs.
    /// Format: `[cid [w1 w2 ...] cid [w1 w2 ...] ...]`
    pub fn build_w_array(&self) -> Vec<PdfObject> {
        let mut result = Vec::new();
        // BTreeSet iterates in sorted order already
        let sorted_glyphs: Vec<u16> = self.used_glyphs.iter().copied().collect();

        let mut i = 0;
        while i < sorted_glyphs.len() {
            let start = sorted_glyphs[i];
            let mut widths = Vec::new();

            // Collect consecutive glyph IDs
            let mut j = i;
            while j < sorted_glyphs.len() && sorted_glyphs[j] == start + (j - i) as u16 {
                let gid = sorted_glyphs[j];
                let raw = self
                    .glyph_widths
                    .get(&gid)
                    .copied()
                    .unwrap_or(self.default_width);
                let pdf_w = ((raw as u32 * 1000) / self.units_per_em as u32) as i64;
                widths.push(PdfObject::Integer(pdf_w));
                j += 1;
            }

            result.push(PdfObject::Integer(start as i64));
            result.push(PdfObject::Array(widths));
            i = j;
        }

        result
    }

    /// Build the ToUnicode CMap stream bytes.
    pub fn build_tounicode_cmap(&self) -> Vec<u8> {
        let mut cmap = String::new();
        cmap.push_str(
            "/CIDInit /ProcSet findresource begin\n\
             12 dict begin\n\
             begincmap\n\
             /CIDSystemInfo\n\
             << /Registry (Adobe)\n\
             /Ordering (UCS)\n\
             /Supplement 0\n\
             >> def\n\
             /CMapName /Adobe-Identity-UCS def\n\
             /CMapType 2 def\n\
             1 begincodespacerange\n\
             <0000> <FFFF>\n\
             endcodespacerange\n",
        );

        // Collect mappings for used glyphs
        let mappings: Vec<(u16, u32)> = self
            .used_glyphs
            .iter()
            .filter_map(|&gid| self.glyph_to_unicode.get(&gid).map(|&cp| (gid, cp)))
            .collect();

        // Write in chunks of 100 (PDF limit per beginbfchar)
        for chunk in mappings.chunks(100) {
            cmap.push_str(&format!("{} beginbfchar\n", chunk.len()));
            for &(gid, cp) in chunk {
                cmap.push_str(&format!("<{:04X}> <{:04X}>\n", gid, cp));
            }
            cmap.push_str("endbfchar\n");
        }

        cmap.push_str(
            "endcmap\n\
             CMapName currentdict /CMap defineresource pop\n\
             end\n\
             end\n",
        );

        cmap.into_bytes()
    }
}

/// Extract the font family name from the name table.
fn extract_name(face: &ttf_parser::Face) -> Option<String> {
    face.names()
        .into_iter()
        .find(|name| name.name_id == ttf_parser::name_id::FAMILY && name.is_unicode())
        .and_then(|name| name.to_string())
}

/// Extract the PostScript name from the name table.
fn extract_postscript_name(face: &ttf_parser::Face) -> Option<String> {
    face.names()
        .into_iter()
        .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME && name.is_unicode())
        .and_then(|name| name.to_string())
}

/// Compute PDF font descriptor flags from the font tables.
fn compute_flags(face: &ttf_parser::Face) -> u32 {
    let mut flags = 0u32;

    // Bit 1 (value 1): FixedPitch
    if face.is_monospaced() {
        flags |= 1;
    }

    // Bit 3 (value 4): Symbolic (set if no standard encoding)
    // Bit 6 (value 32): Nonsymbolic
    // Most Latin TrueType fonts are nonsymbolic
    flags |= 32;

    // Bit 7 (value 64): Italic
    if face.is_italic() {
        flags |= 64;
    }

    flags
}

/// Estimate StemV from the font's weight class.
fn estimate_stem_v(face: &ttf_parser::Face) -> i16 {
    let weight = face.weight().to_number();
    // Rough approximation: StemV ~ 10 + 220 * (weight/1000)^2
    let w = weight as f64 / 1000.0;
    (10.0 + 220.0 * w * w) as i16
}
