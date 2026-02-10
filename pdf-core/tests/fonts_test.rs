use pdf_core::fonts::{BuiltinFont, FontMetrics};

#[test]
fn helvetica_space_width() {
    assert_eq!(
        FontMetrics::char_width(BuiltinFont::Helvetica, ' '),
        278,
    );
}

#[test]
fn helvetica_bold_space_width() {
    assert_eq!(
        FontMetrics::char_width(
            BuiltinFont::HelveticaBold,
            ' ',
        ),
        278,
    );
}

#[test]
fn helvetica_uppercase_a() {
    assert_eq!(
        FontMetrics::char_width(BuiltinFont::Helvetica, 'A'),
        667,
    );
}

#[test]
fn helvetica_bold_uppercase_a() {
    assert_eq!(
        FontMetrics::char_width(
            BuiltinFont::HelveticaBold,
            'A',
        ),
        722,
    );
}

#[test]
fn unmapped_char_returns_default() {
    // Non-ASCII character should return 278
    assert_eq!(
        FontMetrics::char_width(
            BuiltinFont::Helvetica,
            '\u{00E9}',
        ),
        278,
    );
    // Control character
    assert_eq!(
        FontMetrics::char_width(BuiltinFont::Helvetica, '\n'),
        278,
    );
}

#[test]
fn measure_text_hello() {
    // H=722, e=556, l=222, l=222, o=556 => total = 2278
    // At 12pt: 2278 * 12 / 1000 = 27.336
    let width = FontMetrics::measure_text(
        "Hello",
        BuiltinFont::Helvetica,
        12.0,
    );
    assert!((width - 27.336).abs() < 0.001);
}

#[test]
fn measure_text_empty() {
    let width = FontMetrics::measure_text(
        "",
        BuiltinFont::Helvetica,
        12.0,
    );
    assert!((width - 0.0).abs() < 0.001);
}

#[test]
fn measure_text_bold_wider() {
    let normal = FontMetrics::measure_text(
        "Hello",
        BuiltinFont::Helvetica,
        12.0,
    );
    let bold = FontMetrics::measure_text(
        "Hello",
        BuiltinFont::HelveticaBold,
        12.0,
    );
    assert!(bold > normal);
}

#[test]
fn line_height_at_12pt() {
    let h = FontMetrics::line_height(
        BuiltinFont::Helvetica,
        12.0,
    );
    assert!((h - 14.4).abs() < 0.001);
}

#[test]
fn pdf_name_returns_correct_ids() {
    assert_eq!(BuiltinFont::Helvetica.pdf_name(), "F1");
    assert_eq!(BuiltinFont::HelveticaBold.pdf_name(), "F2");
    assert_eq!(
        BuiltinFont::HelveticaOblique.pdf_name(),
        "F3",
    );
    assert_eq!(
        BuiltinFont::HelveticaBoldOblique.pdf_name(),
        "F4",
    );
    assert_eq!(BuiltinFont::TimesRoman.pdf_name(), "F5");
    assert_eq!(BuiltinFont::TimesBold.pdf_name(), "F6");
    assert_eq!(BuiltinFont::TimesItalic.pdf_name(), "F7");
    assert_eq!(
        BuiltinFont::TimesBoldItalic.pdf_name(),
        "F8",
    );
    assert_eq!(BuiltinFont::Courier.pdf_name(), "F9");
    assert_eq!(BuiltinFont::CourierBold.pdf_name(), "F10");
    assert_eq!(
        BuiltinFont::CourierOblique.pdf_name(),
        "F11",
    );
    assert_eq!(
        BuiltinFont::CourierBoldOblique.pdf_name(),
        "F12",
    );
    assert_eq!(BuiltinFont::Symbol.pdf_name(), "F13");
    assert_eq!(BuiltinFont::ZapfDingbats.pdf_name(), "F14");
}

#[test]
fn pdf_base_name_returns_correct_names() {
    assert_eq!(
        BuiltinFont::Helvetica.pdf_base_name(),
        "Helvetica",
    );
    assert_eq!(
        BuiltinFont::HelveticaBold.pdf_base_name(),
        "Helvetica-Bold",
    );
    assert_eq!(
        BuiltinFont::TimesRoman.pdf_base_name(),
        "Times-Roman",
    );
    assert_eq!(
        BuiltinFont::Courier.pdf_base_name(),
        "Courier",
    );
    assert_eq!(
        BuiltinFont::Symbol.pdf_base_name(),
        "Symbol",
    );
    assert_eq!(
        BuiltinFont::ZapfDingbats.pdf_base_name(),
        "ZapfDingbats",
    );
}

#[test]
fn from_name_roundtrips() {
    assert_eq!(
        BuiltinFont::from_name("Helvetica"),
        Some(BuiltinFont::Helvetica),
    );
    assert_eq!(
        BuiltinFont::from_name("Helvetica-Bold"),
        Some(BuiltinFont::HelveticaBold),
    );
    assert_eq!(
        BuiltinFont::from_name("Times-Roman"),
        Some(BuiltinFont::TimesRoman),
    );
    assert_eq!(
        BuiltinFont::from_name("Courier"),
        Some(BuiltinFont::Courier),
    );
    assert_eq!(
        BuiltinFont::from_name("ZapfDingbats"),
        Some(BuiltinFont::ZapfDingbats),
    );
    assert_eq!(BuiltinFont::from_name("NotAFont"), None);
}

#[test]
fn times_roman_widths() {
    // Times-Roman 'A' = 722
    assert_eq!(
        FontMetrics::char_width(
            BuiltinFont::TimesRoman,
            'A',
        ),
        722,
    );
    // Times-Roman space = 250
    assert_eq!(
        FontMetrics::char_width(
            BuiltinFont::TimesRoman,
            ' ',
        ),
        250,
    );
}

#[test]
fn times_bold_widths() {
    // Times-Bold 'A' = 722
    assert_eq!(
        FontMetrics::char_width(
            BuiltinFont::TimesBold,
            'A',
        ),
        722,
    );
    // Times-Bold 'a' = 500
    assert_eq!(
        FontMetrics::char_width(
            BuiltinFont::TimesBold,
            'a',
        ),
        500,
    );
}

#[test]
fn courier_uniform_width() {
    // All Courier variants should return 600 for any character
    assert_eq!(
        FontMetrics::char_width(BuiltinFont::Courier, 'A'),
        600,
    );
    assert_eq!(
        FontMetrics::char_width(BuiltinFont::Courier, ' '),
        600,
    );
    assert_eq!(
        FontMetrics::char_width(
            BuiltinFont::CourierBold,
            'W',
        ),
        600,
    );
    assert_eq!(
        FontMetrics::char_width(
            BuiltinFont::CourierOblique,
            'i',
        ),
        600,
    );
    // Non-ASCII also returns 600 for Courier
    assert_eq!(
        FontMetrics::char_width(
            BuiltinFont::Courier,
            '\u{00E9}',
        ),
        600,
    );
}

#[test]
fn helvetica_oblique_shares_widths() {
    // Oblique variants share widths with their upright form
    assert_eq!(
        FontMetrics::char_width(
            BuiltinFont::HelveticaOblique,
            'A',
        ),
        FontMetrics::char_width(
            BuiltinFont::Helvetica,
            'A',
        ),
    );
    assert_eq!(
        FontMetrics::char_width(
            BuiltinFont::HelveticaBoldOblique,
            'A',
        ),
        FontMetrics::char_width(
            BuiltinFont::HelveticaBold,
            'A',
        ),
    );
}

#[test]
fn symbol_uses_default_width() {
    assert_eq!(
        FontMetrics::char_width(BuiltinFont::Symbol, 'A'),
        278,
    );
    assert_eq!(
        FontMetrics::char_width(
            BuiltinFont::ZapfDingbats,
            'A',
        ),
        278,
    );
}
