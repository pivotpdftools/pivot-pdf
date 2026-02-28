/// Example: Simulated database report rendered as a multi-page table.
///
/// Demonstrates streaming row-by-row placement using `fit_row` + `TableCursor`.
/// A header row is automatically repeated at the top of each new page by
/// checking `cursor.is_first_row()` before placing each data row.
///
/// Run with:
///   cargo run --example generate_tables -p pdf-examples
///
/// Opens output at: examples/output/rust-tables.pdf
use pdf_core::{
    BuiltinFont, Cell, CellStyle, Color, FitResult, FontRef, PdfDocument, Rect, Row, Table,
    TableCursor,
};

const PAGE_WIDTH: f64 = 612.0;
const PAGE_HEIGHT: f64 = 792.0;
const MARGIN: f64 = 72.0;

const TABLE_WIDTH: f64 = PAGE_WIDTH - 2.0 * MARGIN;
const TABLE_TOP: f64 = PAGE_HEIGHT - MARGIN;
const TABLE_BOTTOM: f64 = MARGIN;
const TABLE_HEIGHT: f64 = TABLE_TOP - TABLE_BOTTOM;

fn header_style() -> CellStyle {
    CellStyle {
        font: FontRef::Builtin(BuiltinFont::HelveticaBold),
        font_size: 9.0,
        background_color: Some(Color::rgb(0.2, 0.3, 0.5)),
        text_color: Some(Color::rgb(1.0, 1.0, 1.0)),
        padding: 5.0,
        ..CellStyle::default()
    }
}

fn header_row() -> Row {
    let hs = header_style();
    Row::new(vec![
        Cell::styled("ID", hs.clone()),
        Cell::styled("Name", hs.clone()),
        Cell::styled("Department", hs.clone()),
        Cell::styled("Status", hs.clone()),
        Cell::styled("Amount ($)", hs),
    ])
}

/// Build 60 simulated database rows.
fn db_rows() -> Vec<Row> {
    let row_bg_even = Color::rgb(0.95, 0.97, 1.0);
    let row_bg_odd = Color::rgb(1.0, 1.0, 1.0);

    let departments = [
        "Engineering",
        "Marketing",
        "Sales",
        "HR",
        "Finance",
        "Operations",
    ];
    let statuses = ["Active", "Inactive", "Pending", "Suspended", "Active"];
    let names = [
        "Alice Johnson",
        "Bob Smith",
        "Carol White",
        "David Brown",
        "Emma Davis",
        "Frank Miller",
        "Grace Wilson",
        "Henry Moore",
        "Iris Taylor",
        "Jack Anderson",
    ];

    (0..160_usize)
        .map(|i| {
            let mut row = Row::new(vec![
                Cell::new(format!("{}", i + 1)),
                Cell::new(names[i % names.len()]),
                Cell::new(departments[i % departments.len()]),
                Cell::new(statuses[i % statuses.len()]),
                Cell::new(format!("{:.2}", 1000.0 + (i as f64 * 137.5) % 9000.0)),
            ]);
            row.background_color = Some(if i % 2 == 0 { row_bg_even } else { row_bg_odd });
            row
        })
        .collect()
}

fn new_page_rect() -> Rect {
    Rect {
        x: MARGIN,
        y: TABLE_TOP,
        width: TABLE_WIDTH,
        height: TABLE_HEIGHT,
    }
}

fn main() {
    std::fs::create_dir_all("examples/output").unwrap();
    let path = "examples/output/rust-tables.pdf";
    let mut doc = PdfDocument::create(path).expect("create PDF");
    doc.set_compression(true);
    doc.set_info("Title", "Database Report Example");
    doc.set_info("Creator", "rust-pdf generate_tables example");

    // Table config: column widths — ID | Name | Department | Status | Amount
    let table = Table::new(vec![40.0, 120.0, 130.0, 90.0, 88.0]);

    let rows = db_rows();
    let mut rows_iter = rows.iter().peekable();

    doc.begin_page(PAGE_WIDTH, PAGE_HEIGHT);
    let mut cursor = TableCursor::new(&new_page_rect());

    while rows_iter.peek().is_some() {
        // Repeat header at the top of every page.
        if cursor.is_first_row() {
            match doc
                .fit_row(&table, &header_row(), &mut cursor)
                .expect("fit_row header")
            {
                FitResult::BoxEmpty => {
                    eprintln!("Warning: bounding box too small to fit header row");
                    break;
                }
                _ => {}
            }
        }

        let row = rows_iter.peek().unwrap();
        match doc.fit_row(&table, row, &mut cursor).expect("fit_row") {
            FitResult::Stop => {
                rows_iter.next();
            }
            FitResult::BoxFull => {
                // Page is full — start a new page and retry the same row.
                doc.end_page().expect("end_page");
                doc.begin_page(PAGE_WIDTH, PAGE_HEIGHT);
                cursor.reset(&new_page_rect());
            }
            FitResult::BoxEmpty => {
                eprintln!("Warning: bounding box too small to fit any rows");
                break;
            }
        }
    }

    doc.end_page().expect("end_page");
    doc.end_document().expect("end_document");

    println!("Written to {}", path);
}
