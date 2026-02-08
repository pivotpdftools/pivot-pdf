use pdf_core::fonts::{FontId, FontMetrics};

#[test]
fn helvetica_space_width() {
    assert_eq!(FontMetrics::char_width(FontId::Helvetica, ' '), 278);
}

#[test]
fn helvetica_bold_space_width() {
    assert_eq!(
        FontMetrics::char_width(FontId::HelveticaBold, ' '),
        278
    );
}

#[test]
fn helvetica_uppercase_a() {
    assert_eq!(FontMetrics::char_width(FontId::Helvetica, 'A'), 667);
}

#[test]
fn helvetica_bold_uppercase_a() {
    assert_eq!(
        FontMetrics::char_width(FontId::HelveticaBold, 'A'),
        722
    );
}

#[test]
fn unmapped_char_returns_default() {
    // Non-ASCII character should return 278
    assert_eq!(FontMetrics::char_width(FontId::Helvetica, '\u{00E9}'), 278);
    // Control character
    assert_eq!(FontMetrics::char_width(FontId::Helvetica, '\n'), 278);
}

#[test]
fn measure_text_hello() {
    // H=722, e=556, l=222, l=222, o=556 => total = 2278
    // At 12pt: 2278 * 12 / 1000 = 27.336
    let width = FontMetrics::measure_text(
        "Hello",
        FontId::Helvetica,
        12.0,
    );
    assert!((width - 27.336).abs() < 0.001);
}

#[test]
fn measure_text_empty() {
    let width = FontMetrics::measure_text(
        "",
        FontId::Helvetica,
        12.0,
    );
    assert!((width - 0.0).abs() < 0.001);
}

#[test]
fn measure_text_bold_wider() {
    let normal = FontMetrics::measure_text(
        "Hello",
        FontId::Helvetica,
        12.0,
    );
    let bold = FontMetrics::measure_text(
        "Hello",
        FontId::HelveticaBold,
        12.0,
    );
    assert!(bold > normal);
}

#[test]
fn line_height_at_12pt() {
    let h = FontMetrics::line_height(FontId::Helvetica, 12.0);
    assert!((h - 14.4).abs() < 0.001);
}

#[test]
fn pdf_name_returns_correct_ids() {
    assert_eq!(FontId::Helvetica.pdf_name(), "F1");
    assert_eq!(FontId::HelveticaBold.pdf_name(), "F2");
}
