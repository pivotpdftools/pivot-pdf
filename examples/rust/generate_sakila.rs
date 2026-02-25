/// Example: Large PDF report from the Sakila SQLite database.
///
/// Queries rental history (3475 rows) and renders it as a multi-page landscape
/// table. The table header repeats on each page. A "Page X of Y" footer appears
/// in the lower-left corner of every page.
///
/// Run with:
///   cargo run --example generate_sakila -p pdf-examples -- /path/to/sakila.db
///
/// Output: examples/output/rust-sakila.pdf
use pdf_core::{
    BuiltinFont, Cell, CellOverflow, CellStyle, Color, FitResult, FontRef, PdfDocument, Rect, Row,
    Table, TableCursor, TextStyle,
};
use rusqlite::{Connection, params};

const PAGE_WIDTH: f64 = 792.0; // landscape
const PAGE_HEIGHT: f64 = 612.0;
const MARGIN: f64 = 36.0;
const FOOTER_HEIGHT: f64 = 20.0;

const TABLE_X: f64 = MARGIN;
const TABLE_TOP: f64 = PAGE_HEIGHT - MARGIN;
const TABLE_BOTTOM: f64 = MARGIN + FOOTER_HEIGHT;
const TABLE_WIDTH: f64 = PAGE_WIDTH - 2.0 * MARGIN;
const TABLE_HEIGHT: f64 = TABLE_TOP - TABLE_BOTTOM;

// Column widths sum to TABLE_WIDTH (720.0):
// ID | Date | Film Title | Year | Rating | Category | Length |
// First Name | Last Name | Email | Address | City | Postal Code
const COL_WIDTHS: [f64; 13] = [
    30.0, 68.0, 85.0, 32.0, 35.0, 60.0, 38.0, 52.0, 52.0, 100.0, 75.0, 55.0, 38.0,
];

const HEADERS: [&str; 13] = [
    "ID", "Date", "Film Title", "Year", "Rating", "Category", "Length", "First Name", "Last Name",
    "Email", "Address", "City", "Postal",
];

const SQL: &str = "
    SELECT
        r.rental_id,
        r.rental_date,
        f.title,
        f.release_year,
        f.rating,
        cat.name AS category,
        f.length AS film_length,
        c.first_name,
        c.last_name,
        c.email,
        a.address,
        cty.city,
        a.postal_code
    FROM rental r
    JOIN customer c ON r.customer_id = c.customer_id
    JOIN address a ON c.address_id = a.address_id
    JOIN city cty ON cty.city_id = a.city_id
    JOIN film f ON r.inventory_id = f.film_id
    JOIN film_category fc ON f.film_id = fc.film_id
    JOIN category cat ON fc.category_id = cat.category_id
";

fn header_style() -> CellStyle {
    CellStyle {
        font: FontRef::Builtin(BuiltinFont::HelveticaBold),
        font_size: 7.0,
        background_color: Some(Color::rgb(0.2, 0.3, 0.5)),
        text_color: Some(Color::rgb(1.0, 1.0, 1.0)),
        padding: 3.0,
        ..CellStyle::default()
    }
}

fn header_row() -> Row {
    let hs = header_style();
    Row::new(HEADERS.iter().map(|h| Cell::styled(*h, hs.clone())).collect())
}

const LAST_NAME_COL: usize = 8;
const EMAIL_COL: usize = 9;

fn data_row(values: &[String], row_index: usize) -> Row {
    let bg = if row_index % 2 == 0 {
        Color::rgb(0.95, 0.97, 1.0)
    } else {
        Color::rgb(1.0, 1.0, 1.0)
    };
    let cell_style = CellStyle {
        font: FontRef::Builtin(BuiltinFont::Helvetica),
        font_size: 7.0,
        padding: 3.0,
        ..CellStyle::default()
    };
    // Last names are single words that can't wrap; shrink the font to fit.
    let last_name_style = CellStyle { overflow: CellOverflow::Shrink, ..cell_style.clone() };
    // Email addresses have no word-break characters so they can't wrap.
    // Clip prevents them from visually overflowing into adjacent columns.
    let email_style = CellStyle { overflow: CellOverflow::Clip, ..cell_style.clone() };

    let cells = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let style = match i {
                LAST_NAME_COL => last_name_style.clone(),
                EMAIL_COL => email_style.clone(),
                _ => cell_style.clone(),
            };
            Cell::styled(v.as_str(), style)
        })
        .collect();

    let mut row = Row::new(cells);
    row.background_color = Some(bg);
    row
}

fn table_rect() -> Rect {
    Rect { x: TABLE_X, y: TABLE_TOP, width: TABLE_WIDTH, height: TABLE_HEIGHT }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: generate_sakila <path/to/sakila.db>");
        std::process::exit(1);
    }
    let db_path = &args[1];

    std::fs::create_dir_all("examples/output").unwrap();
    let out_path = "examples/output/rust-sakila.pdf";

    let conn = Connection::open(db_path).expect("open database");

    let mut doc = PdfDocument::create(out_path).expect("create PDF");
    doc.set_compression(true);
    doc.set_info("Title", "Sakila Rental Report");
    doc.set_info("Creator", "rust-pdf generate_sakila example");

    let table = Table::new(COL_WIDTHS.to_vec());

    let footer_style = TextStyle { font: FontRef::Builtin(BuiltinFont::Helvetica), font_size: 8.0 };

    let mut stmt = conn.prepare(SQL).expect("prepare SQL");
    let mut rows_iter = stmt
        .query_map(params![], |row| {
            Ok((0..13)
                .map(|i| {
                    let val: rusqlite::types::Value = row.get(i).unwrap();
                    match val {
                        rusqlite::types::Value::Null => String::new(),
                        rusqlite::types::Value::Integer(n) => n.to_string(),
                        rusqlite::types::Value::Real(f) => f.to_string(),
                        rusqlite::types::Value::Text(s) => s,
                        rusqlite::types::Value::Blob(_) => String::from("[blob]"),
                    }
                })
                .collect::<Vec<String>>())
        })
        .expect("query");

    // --- Pass 1: write all table pages ---
    // Stream one row at a time. `current` holds the row being placed; on
    // BoxFull the same row is put back into `current` so it is retried on the
    // next page without ever buffering the full result set.
    doc.begin_page(PAGE_WIDTH, PAGE_HEIGHT);
    let mut cursor = TableCursor::new(&table_rect());
    let mut total_rows: usize = 0;
    let mut row_index: usize = 0;
    let mut current: Option<Vec<String>> =
        rows_iter.next().map(|r| r.expect("row"));

    loop {
        let values = match current.take() {
            None => break,
            Some(v) => v,
        };

        if cursor.is_first_row() {
            match doc.fit_row(&table, &header_row(), &mut cursor).expect("fit_row header") {
                FitResult::BoxEmpty => {
                    eprintln!("Warning: bounding box too small to fit header");
                    break;
                }
                _ => {}
            }
        }

        let row = data_row(&values, row_index);
        match doc.fit_row(&table, &row, &mut cursor).expect("fit_row") {
            FitResult::Stop => {
                current = rows_iter.next().map(|r| r.expect("row"));
                row_index += 1;
                total_rows += 1;
            }
            FitResult::BoxFull => {
                // Put the row back and retry it on the next page.
                current = Some(values);
                doc.end_page().expect("end_page");
                doc.begin_page(PAGE_WIDTH, PAGE_HEIGHT);
                cursor.reset(&table_rect());
            }
            FitResult::BoxEmpty => {
                eprintln!("Warning: bounding box too small to fit any rows");
                break;
            }
        }
    }
    doc.end_page().expect("end_page");

    // --- Pass 2: add "Page X of Y" footer to every page ---
    // Table row backgrounds leave a non-black rg fill color in the graphics
    // state (set outside any q/Q block). PDF concatenates all content streams
    // on a page, so the overlay inherits that color. Reset to black first so
    // the footer text is visible.
    let black = Color::rgb(0.0, 0.0, 0.0);
    let total = doc.page_count();
    for i in 1..=total {
        doc.open_page(i).expect("open_page");
        doc.set_fill_color(black);
        doc.place_text_styled(
            &format!("Page {} of {}", i, total),
            MARGIN,
            16.0,
            &footer_style,
        );
        doc.end_page().expect("end_page after footer overlay");
    }

    doc.end_document().expect("end_document");

    println!("Written to {} ({} pages, {} rows)", out_path, total, total_rows);
}
