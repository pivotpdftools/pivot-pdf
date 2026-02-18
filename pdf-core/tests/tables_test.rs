use pdf_core::{
    BuiltinFont, Cell, CellOverflow, CellStyle, Color, FitResult, FontRef, PdfDocument, Rect,
    Row, Table, TableCursor,
};

/// Check whether a byte pattern exists in the buffer.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

fn make_doc() -> PdfDocument<Vec<u8>> {
    PdfDocument::new(Vec::<u8>::new()).unwrap()
}

fn full_rect() -> Rect {
    Rect { x: 72.0, y: 720.0, width: 468.0, height: 648.0 }
}

fn two_col_table() -> Table {
    Table::new(vec![234.0, 234.0])
}

fn data_row(a: &str, b: &str) -> Row {
    Row::new(vec![Cell::new(a), Cell::new(b)])
}

// -------------------------------------------------------
// Basic placement
// -------------------------------------------------------

#[test]
fn single_row_returns_stop() {
    let table = two_col_table();
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    let result = doc.fit_row(&table, &data_row("A", "B"), &mut cursor).unwrap();
    doc.end_page().unwrap();
    doc.end_document().unwrap();
    assert_eq!(result, FitResult::Stop);
}

#[test]
fn single_row_produces_valid_pdf() {
    let table = two_col_table();
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    doc.fit_row(&table, &data_row("Name", "Value"), &mut cursor).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(contains(&bytes, b"%PDF-1.7"));
    assert!(contains(&bytes, b"%%EOF"));
    assert!(contains(&bytes, b"(Name) Tj"));
    assert!(contains(&bytes, b"(Value) Tj"));
}

#[test]
fn multiple_rows_on_one_page() {
    let table = two_col_table();
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    for i in 0..5 {
        let result = doc
            .fit_row(&table, &data_row(&format!("R{}", i), "data"), &mut cursor)
            .unwrap();
        assert_eq!(result, FitResult::Stop);
    }
    doc.end_page().unwrap();
    doc.end_document().unwrap();
}

// -------------------------------------------------------
// is_first_row state
// -------------------------------------------------------

#[test]
fn is_first_row_true_before_any_placement() {
    let cursor = TableCursor::new(&full_rect());
    assert!(cursor.is_first_row());
}

#[test]
fn is_first_row_false_after_successful_placement() {
    let table = two_col_table();
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    doc.fit_row(&table, &data_row("A", "B"), &mut cursor).unwrap();
    doc.end_page().unwrap();
    doc.end_document().unwrap();
    assert!(!cursor.is_first_row());
}

#[test]
fn reset_restores_is_first_row() {
    let table = two_col_table();
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    doc.fit_row(&table, &data_row("A", "B"), &mut cursor).unwrap();
    doc.end_page().unwrap();
    doc.end_document().unwrap();

    assert!(!cursor.is_first_row());
    cursor.reset(&full_rect());
    assert!(cursor.is_first_row());
}

// -------------------------------------------------------
// FitResult semantics
// -------------------------------------------------------

#[test]
fn box_empty_when_rect_too_small() {
    // Row height ~18pt, rect height only 5pt
    let tiny = Rect { x: 72.0, y: 720.0, width: 468.0, height: 5.0 };
    let table = two_col_table();
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&tiny);
    let result = doc.fit_row(&table, &data_row("X", "Y"), &mut cursor).unwrap();
    doc.end_page().unwrap();
    doc.end_document().unwrap();

    assert_eq!(result, FitResult::BoxEmpty);
    // Nothing placed: cursor still at first_row
    assert!(cursor.is_first_row());
}

#[test]
fn box_full_when_page_has_content_and_row_does_not_fit() {
    // Rect tall enough for exactly ~3 rows (each ~18pt, rect 50pt)
    let short_rect = Rect { x: 72.0, y: 720.0, width: 468.0, height: 50.0 };
    let table = two_col_table();
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&short_rect);

    // Place rows until BoxFull
    let mut placed = 0;
    let mut got_box_full = false;
    for _ in 0..10 {
        match doc.fit_row(&table, &data_row("Row", "Data"), &mut cursor).unwrap() {
            FitResult::Stop    => placed += 1,
            FitResult::BoxFull => { got_box_full = true; break; }
            FitResult::BoxEmpty => panic!("unexpected BoxEmpty"),
        }
    }
    doc.end_page().unwrap();
    doc.end_document().unwrap();

    assert!(placed > 0, "expected at least one row placed before BoxFull");
    assert!(got_box_full);
    // After BoxFull, is_first_row is false (rows were placed)
    assert!(!cursor.is_first_row());
}

// -------------------------------------------------------
// Multi-page streaming loop
// -------------------------------------------------------

#[test]
fn multi_page_streaming_loop() {
    let small_rect = Rect { x: 72.0, y: 720.0, width: 468.0, height: 60.0 };
    let table = two_col_table();
    let rows: Vec<Row> = (0..15)
        .map(|i| data_row(&format!("Row {}", i), "data"))
        .collect();

    let mut doc = make_doc();
    let mut cursor = TableCursor::new(&small_rect);
    let mut iter = rows.iter().peekable();
    let mut pages = 0;

    while iter.peek().is_some() {
        doc.begin_page(612.0, 792.0);
        pages += 1;
        while let Some(row) = iter.peek() {
            match doc.fit_row(&table, row, &mut cursor).unwrap() {
                FitResult::Stop    => { iter.next(); }
                FitResult::BoxFull => break,
                FitResult::BoxEmpty => { iter.next(); break; }
            }
        }
        doc.end_page().unwrap();
        if iter.peek().is_some() {
            cursor.reset(&small_rect);
        }
    }

    let bytes = doc.end_document().unwrap();
    assert!(pages >= 2, "expected multi-page, got {}", pages);
    assert!(contains(&bytes, b"%PDF-1.7"));
}

#[test]
fn header_repeated_on_each_page_via_is_first_row() {
    let small_rect = Rect { x: 72.0, y: 720.0, width: 468.0, height: 60.0 };
    let table = two_col_table();
    let header = data_row("Name", "Value");
    let data: Vec<Row> = (0..12)
        .map(|i| data_row(&format!("Item {}", i), &format!("{}", i * 100)))
        .collect();

    let mut doc = make_doc();
    let mut cursor = TableCursor::new(&small_rect);
    let mut iter = data.iter().peekable();
    let mut pages = 0;

    while iter.peek().is_some() {
        doc.begin_page(612.0, 792.0);
        pages += 1;

        // Header at the top of every page
        if cursor.is_first_row() {
            doc.fit_row(&table, &header, &mut cursor).unwrap();
        }

        while let Some(row) = iter.peek() {
            match doc.fit_row(&table, row, &mut cursor).unwrap() {
                FitResult::Stop    => { iter.next(); }
                FitResult::BoxFull => break,
                FitResult::BoxEmpty => { iter.next(); break; }
            }
        }

        doc.end_page().unwrap();
        if iter.peek().is_some() {
            cursor.reset(&small_rect);
        }
    }

    let bytes = doc.end_document().unwrap();
    // "Name" appears once per page
    let count = bytes.windows(b"(Name) Tj".len())
        .filter(|w| *w == b"(Name) Tj")
        .count();
    assert!(pages >= 2);
    assert_eq!(count, pages, "header should appear on every page");
}

// -------------------------------------------------------
// Borders
// -------------------------------------------------------

#[test]
fn borders_enabled_by_default() {
    let table = two_col_table();
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    doc.fit_row(&table, &data_row("A", "B"), &mut cursor).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(contains(&bytes, b"re\nS\n"));
    assert!(contains(&bytes, b" RG\n"));
}

#[test]
fn borders_disabled_when_width_zero() {
    let mut table = two_col_table();
    table.border_width = 0.0;
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    doc.fit_row(&table, &data_row("A", "B"), &mut cursor).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(!contains(&bytes, b" RG\n"));
}

#[test]
fn custom_border_color_is_emitted() {
    let mut table = two_col_table();
    table.border_color = Color::rgb(1.0, 0.0, 0.0);
    table.border_width = 1.0;
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    doc.fit_row(&table, &data_row("A", "B"), &mut cursor).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(contains(&bytes, b"1 0 0 RG\n"));
}

// -------------------------------------------------------
// Background colors
// -------------------------------------------------------

#[test]
fn row_background_color_emits_fill() {
    let table = Table::new(vec![468.0]);
    let mut row = Row::new(vec![Cell::new("Hello")]);
    row.background_color = Some(Color::rgb(0.8, 0.9, 1.0));

    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    doc.fit_row(&table, &row, &mut cursor).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(contains(&bytes, b"0.8 0.9 1 rg\n"));
    assert!(contains(&bytes, b" re\nf\n"));
}

#[test]
fn cell_background_overrides_row_background() {
    let cell_style = CellStyle {
        background_color: Some(Color::rgb(1.0, 0.0, 0.0)),
        ..CellStyle::default()
    };
    let mut row = Row::new(vec![Cell::styled("Text", cell_style)]);
    row.background_color = Some(Color::gray(0.5));

    let table = Table::new(vec![468.0]);
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    doc.fit_row(&table, &row, &mut cursor).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(contains(&bytes, b"0.5 0.5 0.5 rg\n"));
    assert!(contains(&bytes, b"1 0 0 rg\n"));
}

// -------------------------------------------------------
// Text color
// -------------------------------------------------------

#[test]
fn cell_text_color_emits_rg_in_bt_block() {
    let style = CellStyle {
        text_color: Some(Color::rgb(0.0, 0.5, 1.0)),
        ..CellStyle::default()
    };
    let table = Table::new(vec![468.0]);
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    doc.fit_row(&table, &Row::new(vec![Cell::styled("Colored", style)]), &mut cursor).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(contains(&bytes, b"0 0.5 1 rg\n"));
    assert!(contains(&bytes, b"(Colored) Tj"));
}

#[test]
fn default_text_color_is_black_not_background_color() {
    // Regression: without an explicit rg, text uses the fill color left by
    // background drawing — which makes text invisible on colored backgrounds.
    let mut row = Row::new(vec![Cell::new("Visible")]);
    row.background_color = Some(Color::rgb(0.9, 0.9, 0.9));

    let table = Table::new(vec![468.0]);
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    doc.fit_row(&table, &row, &mut cursor).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    // Black text color must be set explicitly inside BT block
    assert!(contains(&bytes, b"0 0 0 rg\n"));
    assert!(contains(&bytes, b"(Visible) Tj"));
}

// -------------------------------------------------------
// Overflow modes
// -------------------------------------------------------

#[test]
fn wrap_mode_multi_line_content_fits() {
    let long_text = "word ".repeat(60);
    let table = Table::new(vec![234.0]);
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    let result = doc
        .fit_row(&table, &Row::new(vec![Cell::new(long_text.trim())]), &mut cursor)
        .unwrap();
    doc.end_page().unwrap();
    doc.end_document().unwrap();
    assert_eq!(result, FitResult::Stop);
}

#[test]
fn clip_mode_with_fixed_row_height() {
    let style = CellStyle { overflow: CellOverflow::Clip, ..CellStyle::default() };
    let long_text = "word ".repeat(40);
    let mut row = Row::new(vec![Cell::styled(long_text.trim(), style)]);
    row.height = Some(25.0);

    let table = Table::new(vec![234.0]);
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    let result = doc.fit_row(&table, &row, &mut cursor).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    assert!(contains(&bytes, b"re\nW\nn\n"));
}

#[test]
fn shrink_mode_with_fixed_row_height() {
    let style = CellStyle {
        overflow: CellOverflow::Shrink,
        font_size: 20.0,
        ..CellStyle::default()
    };
    let mut row = Row::new(vec![Cell::styled(
        "This is a longer sentence that needs to fit within one line height.",
        style,
    )]);
    row.height = Some(30.0);

    let table = Table::new(vec![234.0]);
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    let result = doc.fit_row(&table, &row, &mut cursor).unwrap();
    doc.end_page().unwrap();
    doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
}

#[test]
fn wrap_mode_row_height_accounts_for_wrapped_lines() {
    // Narrow column forces multi-line text; verify Td operators show multiple lines
    let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa";
    let table = Table::new(vec![80.0]);
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    let result = doc
        .fit_row(&table, &Row::new(vec![Cell::new(text)]), &mut cursor)
        .unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert_eq!(result, FitResult::Stop);
    assert!(contains(&bytes, b"0 -"), "Expected multi-line Td operators");
}

// -------------------------------------------------------
// Font selection
// -------------------------------------------------------

#[test]
fn cell_style_custom_font_is_used() {
    let style = CellStyle {
        font: FontRef::Builtin(BuiltinFont::HelveticaBold),
        font_size: 12.0,
        ..CellStyle::default()
    };
    let table = Table::new(vec![468.0]);
    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    doc.fit_row(&table, &Row::new(vec![Cell::styled("Bold", style)]), &mut cursor).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(contains(&bytes, b"/F2 12 Tf"));
}

// -------------------------------------------------------
// Header row pattern
// -------------------------------------------------------

#[test]
fn header_row_with_styled_cells() {
    let header_style = CellStyle {
        font: FontRef::Builtin(BuiltinFont::HelveticaBold),
        font_size: 10.0,
        background_color: Some(Color::gray(0.2)),
        text_color: Some(Color::rgb(1.0, 1.0, 1.0)),
        ..CellStyle::default()
    };

    let table = Table::new(vec![156.0, 156.0, 156.0]);
    let header = Row::new(vec![
        Cell::styled("Name", header_style.clone()),
        Cell::styled("Status", header_style.clone()),
        Cell::styled("Amount", header_style.clone()),
    ]);
    let data = Row::new(vec![
        Cell::new("Alice"),
        Cell::new("Active"),
        Cell::new("$1,000"),
    ]);

    let mut doc = make_doc();
    doc.begin_page(612.0, 792.0);
    let mut cursor = TableCursor::new(&full_rect());
    doc.fit_row(&table, &header, &mut cursor).unwrap();
    doc.fit_row(&table, &data, &mut cursor).unwrap();
    doc.end_page().unwrap();
    let bytes = doc.end_document().unwrap();

    assert!(contains(&bytes, b"(Name) Tj"));
    assert!(contains(&bytes, b"(Alice) Tj"));
    assert!(contains(&bytes, b"0.2 0.2 0.2 rg\n"));
    assert!(contains(&bytes, b"1 1 1 rg\n"));
}
