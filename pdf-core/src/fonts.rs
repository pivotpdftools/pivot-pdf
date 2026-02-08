/// Font identifier for built-in PDF fonts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontId {
    Helvetica,
    HelveticaBold,
}

impl FontId {
    /// Returns the PDF resource name used in content streams (e.g. "F1").
    pub fn pdf_name(&self) -> &'static str {
        match self {
            FontId::Helvetica => "F1",
            FontId::HelveticaBold => "F2",
        }
    }
}

/// Character widths for Helvetica (ASCII 32..=126) in units of 1/1000 em.
/// Source: Adobe Helvetica AFM data.
const HELVETICA_WIDTHS: [u16; 95] = [
    278, // 32 space
    278, // 33 !
    355, // 34 "
    556, // 35 #
    556, // 36 $
    889, // 37 %
    667, // 38 &
    191, // 39 '
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
    278, // 58 :
    278, // 59 ;
    584, // 60 <
    584, // 61 =
    584, // 62 >
    556, // 63 ?
    1015, // 64 @
    667, // 65 A
    667, // 66 B
    722, // 67 C
    722, // 68 D
    667, // 69 E
    611, // 70 F
    778, // 71 G
    722, // 72 H
    278, // 73 I
    500, // 74 J
    667, // 75 K
    556, // 76 L
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
    278, // 91 [
    278, // 92 backslash
    278, // 93 ]
    469, // 94 ^
    556, // 95 _
    333, // 96 `
    556, // 97 a
    556, // 98 b
    500, // 99 c
    556, // 100 d
    556, // 101 e
    278, // 102 f
    556, // 103 g
    556, // 104 h
    222, // 105 i
    222, // 106 j
    500, // 107 k
    222, // 108 l
    833, // 109 m
    556, // 110 n
    556, // 111 o
    556, // 112 p
    556, // 113 q
    333, // 114 r
    500, // 115 s
    278, // 116 t
    556, // 117 u
    500, // 118 v
    722, // 119 w
    500, // 120 x
    500, // 121 y
    500, // 122 z
    334, // 123 {
    260, // 124 |
    334, // 125 }
    584, // 126 ~
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

/// Default width for characters outside the mapped range (1/1000 em).
const DEFAULT_WIDTH: u16 = 278;

/// Font metrics for built-in PDF fonts.
pub struct FontMetrics;

impl FontMetrics {
    /// Returns the width of a character in 1/1000 em units.
    pub fn char_width(font: FontId, ch: char) -> u16 {
        let code = ch as u32;
        if code < 32 || code > 126 {
            return DEFAULT_WIDTH;
        }
        let index = (code - 32) as usize;
        match font {
            FontId::Helvetica => HELVETICA_WIDTHS[index],
            FontId::HelveticaBold => HELVETICA_BOLD_WIDTHS[index],
        }
    }

    /// Measures the width of a text string in points.
    pub fn measure_text(
        text: &str,
        font: FontId,
        font_size: f64,
    ) -> f64 {
        let total: u32 = text
            .chars()
            .map(|ch| Self::char_width(font, ch) as u32)
            .sum();
        total as f64 * font_size / 1000.0
    }

    /// Returns the line height for a given font size (1.2x multiplier).
    pub fn line_height(_font: FontId, font_size: f64) -> f64 {
        font_size * 1.2
    }
}
