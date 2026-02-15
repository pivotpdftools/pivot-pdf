/// Index into the document's TrueType font list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TrueTypeFontId(pub usize);

/// Unified font reference: either a builtin PDF font or a loaded
/// TrueType font.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontRef {
    Builtin(BuiltinFont),
    TrueType(TrueTypeFontId),
}

impl From<BuiltinFont> for FontRef {
    fn from(font: BuiltinFont) -> Self {
        FontRef::Builtin(font)
    }
}

/// Font identifier for the 14 standard PDF fonts.
/// These fonts are guaranteed available in all PDF viewers
/// without embedding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BuiltinFont {
    Helvetica,
    HelveticaBold,
    HelveticaOblique,
    HelveticaBoldOblique,
    TimesRoman,
    TimesBold,
    TimesItalic,
    TimesBoldItalic,
    Courier,
    CourierBold,
    CourierOblique,
    CourierBoldOblique,
    Symbol,
    ZapfDingbats,
}

impl BuiltinFont {
    /// Returns the PDF resource name used in content streams
    /// (e.g. "F1"). Fixed mapping by variant order.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            BuiltinFont::Helvetica => "F1",
            BuiltinFont::HelveticaBold => "F2",
            BuiltinFont::HelveticaOblique => "F3",
            BuiltinFont::HelveticaBoldOblique => "F4",
            BuiltinFont::TimesRoman => "F5",
            BuiltinFont::TimesBold => "F6",
            BuiltinFont::TimesItalic => "F7",
            BuiltinFont::TimesBoldItalic => "F8",
            BuiltinFont::Courier => "F9",
            BuiltinFont::CourierBold => "F10",
            BuiltinFont::CourierOblique => "F11",
            BuiltinFont::CourierBoldOblique => "F12",
            BuiltinFont::Symbol => "F13",
            BuiltinFont::ZapfDingbats => "F14",
        }
    }

    /// Returns the PDF BaseFont name (e.g. "Helvetica",
    /// "Times-Roman").
    pub fn pdf_base_name(&self) -> &'static str {
        match self {
            BuiltinFont::Helvetica => "Helvetica",
            BuiltinFont::HelveticaBold => "Helvetica-Bold",
            BuiltinFont::HelveticaOblique => "Helvetica-Oblique",
            BuiltinFont::HelveticaBoldOblique => "Helvetica-BoldOblique",
            BuiltinFont::TimesRoman => "Times-Roman",
            BuiltinFont::TimesBold => "Times-Bold",
            BuiltinFont::TimesItalic => "Times-Italic",
            BuiltinFont::TimesBoldItalic => "Times-BoldItalic",
            BuiltinFont::Courier => "Courier",
            BuiltinFont::CourierBold => "Courier-Bold",
            BuiltinFont::CourierOblique => "Courier-Oblique",
            BuiltinFont::CourierBoldOblique => "Courier-BoldOblique",
            BuiltinFont::Symbol => "Symbol",
            BuiltinFont::ZapfDingbats => "ZapfDingbats",
        }
    }

    /// Look up a BuiltinFont by its PDF base name string.
    /// Returns None if the name doesn't match any variant.
    pub fn from_name(name: &str) -> Option<BuiltinFont> {
        match name {
            "Helvetica" => Some(BuiltinFont::Helvetica),
            "Helvetica-Bold" => Some(BuiltinFont::HelveticaBold),
            "Helvetica-Oblique" => Some(BuiltinFont::HelveticaOblique),
            "Helvetica-BoldOblique" => Some(BuiltinFont::HelveticaBoldOblique),
            "Times-Roman" => Some(BuiltinFont::TimesRoman),
            "Times-Bold" => Some(BuiltinFont::TimesBold),
            "Times-Italic" => Some(BuiltinFont::TimesItalic),
            "Times-BoldItalic" => Some(BuiltinFont::TimesBoldItalic),
            "Courier" => Some(BuiltinFont::Courier),
            "Courier-Bold" => Some(BuiltinFont::CourierBold),
            "Courier-Oblique" => Some(BuiltinFont::CourierOblique),
            "Courier-BoldOblique" => Some(BuiltinFont::CourierBoldOblique),
            "Symbol" => Some(BuiltinFont::Symbol),
            "ZapfDingbats" => Some(BuiltinFont::ZapfDingbats),
            _ => None,
        }
    }
}

/// Character widths for Helvetica (ASCII 32..=126) in units of 1/1000 em.
/// Source: Adobe Helvetica AFM data.
const HELVETICA_WIDTHS: [u16; 95] = [
    278,  // 32 space
    278,  // 33 !
    355,  // 34 "
    556,  // 35 #
    556,  // 36 $
    889,  // 37 %
    667,  // 38 &
    191,  // 39 '
    333,  // 40 (
    333,  // 41 )
    389,  // 42 *
    584,  // 43 +
    278,  // 44 ,
    333,  // 45 -
    278,  // 46 .
    278,  // 47 /
    556,  // 48 0
    556,  // 49 1
    556,  // 50 2
    556,  // 51 3
    556,  // 52 4
    556,  // 53 5
    556,  // 54 6
    556,  // 55 7
    556,  // 56 8
    556,  // 57 9
    278,  // 58 :
    278,  // 59 ;
    584,  // 60 <
    584,  // 61 =
    584,  // 62 >
    556,  // 63 ?
    1015, // 64 @
    667,  // 65 A
    667,  // 66 B
    722,  // 67 C
    722,  // 68 D
    667,  // 69 E
    611,  // 70 F
    778,  // 71 G
    722,  // 72 H
    278,  // 73 I
    500,  // 74 J
    667,  // 75 K
    556,  // 76 L
    833,  // 77 M
    722,  // 78 N
    778,  // 79 O
    667,  // 80 P
    778,  // 81 Q
    722,  // 82 R
    667,  // 83 S
    611,  // 84 T
    722,  // 85 U
    667,  // 86 V
    944,  // 87 W
    667,  // 88 X
    667,  // 89 Y
    611,  // 90 Z
    278,  // 91 [
    278,  // 92 backslash
    278,  // 93 ]
    469,  // 94 ^
    556,  // 95 _
    333,  // 96 `
    556,  // 97 a
    556,  // 98 b
    500,  // 99 c
    556,  // 100 d
    556,  // 101 e
    278,  // 102 f
    556,  // 103 g
    556,  // 104 h
    222,  // 105 i
    222,  // 106 j
    500,  // 107 k
    222,  // 108 l
    833,  // 109 m
    556,  // 110 n
    556,  // 111 o
    556,  // 112 p
    556,  // 113 q
    333,  // 114 r
    500,  // 115 s
    278,  // 116 t
    556,  // 117 u
    500,  // 118 v
    722,  // 119 w
    500,  // 120 x
    500,  // 121 y
    500,  // 122 z
    334,  // 123 {
    260,  // 124 |
    334,  // 125 }
    584,  // 126 ~
];

/// Character widths for Helvetica-Bold (ASCII 32..=126) in 1/1000 em.
/// Source: Adobe Helvetica-Bold AFM data.
const HELVETICA_BOLD_WIDTHS: [u16; 95] = [
    278, // 32 space
    333, // 33 !
    474, // 34 "
    556, // 35 #
    556, // 36 $
    889, // 37 %
    722, // 38 &
    238, // 39 '
    333, // 40 (
    333, // 41 )
    389, // 42 *
    584, // 43 +
    278, // 44 ,
    333, // 45 -
    278, // 46 .
    278, // 47 /
    556, // 48 0
    556, // 49 1
    556, // 50 2
    556, // 51 3
    556, // 52 4
    556, // 53 5
    556, // 54 6
    556, // 55 7
    556, // 56 8
    556, // 57 9
    333, // 58 :
    333, // 59 ;
    584, // 60 <
    584, // 61 =
    584, // 62 >
    611, // 63 ?
    975, // 64 @
    722, // 65 A
    722, // 66 B
    722, // 67 C
    722, // 68 D
    667, // 69 E
    611, // 70 F
    778, // 71 G
    722, // 72 H
    278, // 73 I
    556, // 74 J
    722, // 75 K
    611, // 76 L
    833, // 77 M
    722, // 78 N
    778, // 79 O
    667, // 80 P
    778, // 81 Q
    722, // 82 R
    667, // 83 S
    611, // 84 T
    722, // 85 U
    667, // 86 V
    944, // 87 W
    667, // 88 X
    667, // 89 Y
    611, // 90 Z
    333, // 91 [
    278, // 92 backslash
    333, // 93 ]
    584, // 94 ^
    556, // 95 _
    333, // 96 `
    556, // 97 a
    611, // 98 b
    556, // 99 c
    611, // 100 d
    556, // 101 e
    333, // 102 f
    611, // 103 g
    611, // 104 h
    278, // 105 i
    278, // 106 j
    556, // 107 k
    278, // 108 l
    889, // 109 m
    611, // 110 n
    611, // 111 o
    611, // 112 p
    611, // 113 q
    389, // 114 r
    556, // 115 s
    333, // 116 t
    611, // 117 u
    556, // 118 v
    778, // 119 w
    556, // 120 x
    556, // 121 y
    500, // 122 z
    389, // 123 {
    280, // 124 |
    389, // 125 }
    584, // 126 ~
];

/// Character widths for Times-Roman (ASCII 32..=126) in 1/1000 em.
/// Source: Adobe Times-Roman AFM data.
const TIMES_ROMAN_WIDTHS: [u16; 95] = [
    250, // 32 space
    333, // 33 !
    408, // 34 "
    500, // 35 #
    500, // 36 $
    833, // 37 %
    778, // 38 &
    180, // 39 '
    333, // 40 (
    333, // 41 )
    500, // 42 *
    564, // 43 +
    250, // 44 ,
    333, // 45 -
    250, // 46 .
    278, // 47 /
    500, // 48 0
    500, // 49 1
    500, // 50 2
    500, // 51 3
    500, // 52 4
    500, // 53 5
    500, // 54 6
    500, // 55 7
    500, // 56 8
    500, // 57 9
    278, // 58 :
    278, // 59 ;
    564, // 60 <
    564, // 61 =
    564, // 62 >
    444, // 63 ?
    921, // 64 @
    722, // 65 A
    667, // 66 B
    667, // 67 C
    722, // 68 D
    611, // 69 E
    556, // 70 F
    722, // 71 G
    722, // 72 H
    333, // 73 I
    389, // 74 J
    722, // 75 K
    611, // 76 L
    889, // 77 M
    722, // 78 N
    722, // 79 O
    556, // 80 P
    722, // 81 Q
    667, // 82 R
    556, // 83 S
    611, // 84 T
    722, // 85 U
    722, // 86 V
    944, // 87 W
    722, // 88 X
    722, // 89 Y
    611, // 90 Z
    333, // 91 [
    278, // 92 backslash
    333, // 93 ]
    469, // 94 ^
    500, // 95 _
    333, // 96 `
    444, // 97 a
    500, // 98 b
    444, // 99 c
    500, // 100 d
    444, // 101 e
    333, // 102 f
    500, // 103 g
    500, // 104 h
    278, // 105 i
    278, // 106 j
    500, // 107 k
    278, // 108 l
    778, // 109 m
    500, // 110 n
    500, // 111 o
    500, // 112 p
    500, // 113 q
    333, // 114 r
    389, // 115 s
    278, // 116 t
    500, // 117 u
    500, // 118 v
    722, // 119 w
    500, // 120 x
    500, // 121 y
    444, // 122 z
    480, // 123 {
    200, // 124 |
    480, // 125 }
    541, // 126 ~
];

/// Character widths for Times-Bold (ASCII 32..=126) in 1/1000 em.
/// Source: Adobe Times-Bold AFM data.
const TIMES_BOLD_WIDTHS: [u16; 95] = [
    250,  // 32 space
    333,  // 33 !
    555,  // 34 "
    500,  // 35 #
    500,  // 36 $
    1000, // 37 %
    833,  // 38 &
    278,  // 39 '
    333,  // 40 (
    333,  // 41 )
    500,  // 42 *
    570,  // 43 +
    250,  // 44 ,
    333,  // 45 -
    250,  // 46 .
    278,  // 47 /
    500,  // 48 0
    500,  // 49 1
    500,  // 50 2
    500,  // 51 3
    500,  // 52 4
    500,  // 53 5
    500,  // 54 6
    500,  // 55 7
    500,  // 56 8
    500,  // 57 9
    333,  // 58 :
    333,  // 59 ;
    570,  // 60 <
    570,  // 61 =
    570,  // 62 >
    500,  // 63 ?
    930,  // 64 @
    722,  // 65 A
    667,  // 66 B
    722,  // 67 C
    722,  // 68 D
    667,  // 69 E
    611,  // 70 F
    778,  // 71 G
    778,  // 72 H
    389,  // 73 I
    500,  // 74 J
    778,  // 75 K
    667,  // 76 L
    944,  // 77 M
    722,  // 78 N
    778,  // 79 O
    611,  // 80 P
    778,  // 81 Q
    722,  // 82 R
    556,  // 83 S
    667,  // 84 T
    722,  // 85 U
    722,  // 86 V
    1000, // 87 W
    722,  // 88 X
    722,  // 89 Y
    667,  // 90 Z
    333,  // 91 [
    278,  // 92 backslash
    333,  // 93 ]
    581,  // 94 ^
    500,  // 95 _
    333,  // 96 `
    500,  // 97 a
    556,  // 98 b
    444,  // 99 c
    556,  // 100 d
    444,  // 101 e
    333,  // 102 f
    500,  // 103 g
    556,  // 104 h
    278,  // 105 i
    333,  // 106 j
    556,  // 107 k
    278,  // 108 l
    833,  // 109 m
    556,  // 110 n
    500,  // 111 o
    556,  // 112 p
    556,  // 113 q
    444,  // 114 r
    389,  // 115 s
    333,  // 116 t
    556,  // 117 u
    500,  // 118 v
    722,  // 119 w
    500,  // 120 x
    500,  // 121 y
    444,  // 122 z
    394,  // 123 {
    220,  // 124 |
    394,  // 125 }
    520,  // 126 ~
];

/// Character widths for Times-Italic (ASCII 32..=126) in 1/1000 em.
/// Source: Adobe Times-Italic AFM data.
const TIMES_ITALIC_WIDTHS: [u16; 95] = [
    250, // 32 space
    333, // 33 !
    420, // 34 "
    500, // 35 #
    500, // 36 $
    833, // 37 %
    778, // 38 &
    214, // 39 '
    333, // 40 (
    333, // 41 )
    500, // 42 *
    675, // 43 +
    250, // 44 ,
    333, // 45 -
    250, // 46 .
    278, // 47 /
    500, // 48 0
    500, // 49 1
    500, // 50 2
    500, // 51 3
    500, // 52 4
    500, // 53 5
    500, // 54 6
    500, // 55 7
    500, // 56 8
    500, // 57 9
    333, // 58 :
    333, // 59 ;
    675, // 60 <
    675, // 61 =
    675, // 62 >
    500, // 63 ?
    920, // 64 @
    611, // 65 A
    611, // 66 B
    667, // 67 C
    722, // 68 D
    611, // 69 E
    611, // 70 F
    722, // 71 G
    722, // 72 H
    333, // 73 I
    444, // 74 J
    667, // 75 K
    556, // 76 L
    833, // 77 M
    667, // 78 N
    722, // 79 O
    611, // 80 P
    722, // 81 Q
    611, // 82 R
    500, // 83 S
    556, // 84 T
    722, // 85 U
    611, // 86 V
    833, // 87 W
    611, // 88 X
    556, // 89 Y
    556, // 90 Z
    389, // 91 [
    278, // 92 backslash
    389, // 93 ]
    422, // 94 ^
    500, // 95 _
    333, // 96 `
    500, // 97 a
    500, // 98 b
    444, // 99 c
    500, // 100 d
    444, // 101 e
    278, // 102 f
    500, // 103 g
    500, // 104 h
    278, // 105 i
    278, // 106 j
    444, // 107 k
    278, // 108 l
    722, // 109 m
    500, // 110 n
    500, // 111 o
    500, // 112 p
    500, // 113 q
    389, // 114 r
    389, // 115 s
    278, // 116 t
    500, // 117 u
    444, // 118 v
    667, // 119 w
    444, // 120 x
    444, // 121 y
    389, // 122 z
    400, // 123 {
    275, // 124 |
    400, // 125 }
    541, // 126 ~
];

/// Character widths for Times-BoldItalic (ASCII 32..=126) in 1/1000 em.
/// Source: Adobe Times-BoldItalic AFM data.
const TIMES_BOLD_ITALIC_WIDTHS: [u16; 95] = [
    250, // 32 space
    389, // 33 !
    555, // 34 "
    500, // 35 #
    500, // 36 $
    833, // 37 %
    778, // 38 &
    278, // 39 '
    333, // 40 (
    333, // 41 )
    500, // 42 *
    570, // 43 +
    250, // 44 ,
    333, // 45 -
    250, // 46 .
    278, // 47 /
    500, // 48 0
    500, // 49 1
    500, // 50 2
    500, // 51 3
    500, // 52 4
    500, // 53 5
    500, // 54 6
    500, // 55 7
    500, // 56 8
    500, // 57 9
    333, // 58 :
    333, // 59 ;
    570, // 60 <
    570, // 61 =
    570, // 62 >
    500, // 63 ?
    832, // 64 @
    667, // 65 A
    667, // 66 B
    667, // 67 C
    722, // 68 D
    667, // 69 E
    667, // 70 F
    722, // 71 G
    778, // 72 H
    389, // 73 I
    500, // 74 J
    667, // 75 K
    611, // 76 L
    889, // 77 M
    722, // 78 N
    722, // 79 O
    611, // 80 P
    722, // 81 Q
    667, // 82 R
    556, // 83 S
    611, // 84 T
    722, // 85 U
    667, // 86 V
    889, // 87 W
    667, // 88 X
    611, // 89 Y
    611, // 90 Z
    333, // 91 [
    278, // 92 backslash
    333, // 93 ]
    570, // 94 ^
    500, // 95 _
    333, // 96 `
    500, // 97 a
    500, // 98 b
    444, // 99 c
    500, // 100 d
    444, // 101 e
    333, // 102 f
    500, // 103 g
    556, // 104 h
    278, // 105 i
    278, // 106 j
    500, // 107 k
    278, // 108 l
    778, // 109 m
    556, // 110 n
    500, // 111 o
    556, // 112 p
    556, // 113 q
    389, // 114 r
    389, // 115 s
    278, // 116 t
    556, // 117 u
    444, // 118 v
    667, // 119 w
    500, // 120 x
    444, // 121 y
    389, // 122 z
    348, // 123 {
    220, // 124 |
    348, // 125 }
    570, // 126 ~
];

/// Courier uses a uniform width of 600 for all characters.
const COURIER_WIDTH: u16 = 600;

/// Default width for characters outside the mapped range (1/1000 em).
const DEFAULT_WIDTH: u16 = 278;

/// Font metrics for built-in PDF fonts.
pub struct FontMetrics;

impl FontMetrics {
    /// Returns the width of a character in 1/1000 em units.
    pub fn char_width(font: BuiltinFont, ch: char) -> u16 {
        // Courier variants are monospaced
        match font {
            BuiltinFont::Courier
            | BuiltinFont::CourierBold
            | BuiltinFont::CourierOblique
            | BuiltinFont::CourierBoldOblique => {
                return COURIER_WIDTH;
            }
            // Symbol/ZapfDingbats use default width (Phase 1)
            BuiltinFont::Symbol | BuiltinFont::ZapfDingbats => {
                return DEFAULT_WIDTH;
            }
            _ => {}
        }

        let code = ch as u32;
        if code < 32 || code > 126 {
            return DEFAULT_WIDTH;
        }
        let index = (code - 32) as usize;
        match font {
            BuiltinFont::Helvetica | BuiltinFont::HelveticaOblique => HELVETICA_WIDTHS[index],
            BuiltinFont::HelveticaBold | BuiltinFont::HelveticaBoldOblique => {
                HELVETICA_BOLD_WIDTHS[index]
            }
            BuiltinFont::TimesRoman => TIMES_ROMAN_WIDTHS[index],
            BuiltinFont::TimesBold => TIMES_BOLD_WIDTHS[index],
            BuiltinFont::TimesItalic => TIMES_ITALIC_WIDTHS[index],
            BuiltinFont::TimesBoldItalic => TIMES_BOLD_ITALIC_WIDTHS[index],
            // Already handled above
            _ => DEFAULT_WIDTH,
        }
    }

    /// Measures the width of a text string in points.
    pub fn measure_text(text: &str, font: BuiltinFont, font_size: f64) -> f64 {
        let total: u32 = text
            .chars()
            .map(|ch| Self::char_width(font, ch) as u32)
            .sum();
        total as f64 * font_size / 1000.0
    }

    /// Returns the line height for a given font size
    /// (1.2x multiplier).
    pub fn line_height(_font: BuiltinFont, font_size: f64) -> f64 {
        font_size * 1.2
    }
}
