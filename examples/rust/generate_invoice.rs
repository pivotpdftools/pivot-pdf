/// Invoice example — realistic single-page invoice layout.
///
/// Demonstrates the primary library use case: professional documents
/// combining graphics (logo), styled text, and tables (line items).
///
/// Run with:
///   cargo run --example generate_invoice -p pdf-examples
///
/// Opens output at: examples/output/rust-invoice.pdf
use pdf_core::{
    BuiltinFont, Cell, CellStyle, Color, FitResult, FontRef, PdfDocument, Rect, Row, Table,
    TableCursor, TextAlign, TextStyle,
};

const PAGE_W: f64 = 612.0;
const PAGE_H: f64 = 792.0;
const MARGIN: f64 = 72.0;
const RIGHT: f64 = PAGE_W - MARGIN; // 540.0

// ── font helpers ──────────────────────────────────────────────────────────────

fn bold(sz: f64) -> TextStyle {
    TextStyle { font: FontRef::Builtin(BuiltinFont::HelveticaBold), font_size: sz }
}

fn regular(sz: f64) -> TextStyle {
    TextStyle { font: FontRef::Builtin(BuiltinFont::Helvetica), font_size: sz }
}

fn oblique(sz: f64) -> TextStyle {
    TextStyle { font: FontRef::Builtin(BuiltinFont::HelveticaOblique), font_size: sz }
}

// ── color helpers ─────────────────────────────────────────────────────────────

fn navy() -> Color { Color::rgb(0.118, 0.227, 0.373) }
fn teal() -> Color { Color::rgb(0.0, 0.706, 0.847) }
fn mid_gray() -> Color { Color::rgb(0.5, 0.5, 0.5) }
fn stripe_bg() -> Color { Color::rgb(0.95, 0.97, 1.0) }

// ── currency formatting ───────────────────────────────────────────────────────

/// Format a monetary value with thousands separator: 9600.00 → "$9,600.00"
fn fmt_money(amount: f64) -> String {
    let cents = (amount * 100.0).round() as u64;
    let dollars = cents / 100;
    let cents_part = cents % 100;

    let s = dollars.to_string();
    let with_commas = s
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect::<Vec<_>>()
        .join(",");

    format!("${}.{:02}", with_commas, cents_part)
}

// ── invoice data ──────────────────────────────────────────────────────────────

struct LineItem {
    description: &'static str,
    qty: u32,
    unit_price: f64,
}

impl LineItem {
    fn total(&self) -> f64 {
        self.qty as f64 * self.unit_price
    }
}

const ITEMS: &[LineItem] = &[
    LineItem { description: "Web Development Services",     qty: 40, unit_price: 150.00 },
    LineItem { description: "UI/UX Design",                 qty: 20, unit_price: 125.00 },
    LineItem { description: "Server Setup & Configuration",  qty:  1, unit_price: 500.00 },
    LineItem { description: "Monthly Maintenance",           qty:  3, unit_price: 200.00 },
    LineItem { description: "Brand Identity & Style Guide",  qty:  1, unit_price: 2_500.00 },
    LineItem { description: "SEO Optimization Package",      qty:  1, unit_price: 800.00 },
    LineItem { description: "CMS Training Sessions",         qty:  4, unit_price: 150.00 },
    LineItem { description: "Cloud Infrastructure Setup",    qty:  1, unit_price: 1_200.00 },
    LineItem { description: "Security Audit",                qty:  1, unit_price: 1_500.00 },
    LineItem { description: "Mobile App Development",        qty: 80, unit_price: 150.00 },
    LineItem { description: "Annual Support Contract",       qty:  1, unit_price: 3_600.00 },
];

// ── logo ──────────────────────────────────────────────────────────────────────

/// Draws a graphic logo: navy block with teal accent stripe and white initials,
/// followed by company name and address.
fn draw_logo<W: std::io::Write>(doc: &mut PdfDocument<W>) {
    // Navy filled block with teal accent stripe at the bottom
    doc.save_state();
    doc.set_fill_color(navy());
    doc.rect(MARGIN, 740.0, 46.0, 40.0);
    doc.fill();
    doc.set_fill_color(teal());
    doc.rect(MARGIN, 740.0, 46.0, 6.0);
    doc.fill();
    // White "NP" initials centered in the block
    doc.set_fill_color(Color::rgb(1.0, 1.0, 1.0));
    doc.place_text_styled("NP", MARGIN + 5.0, 751.0, &bold(18.0));
    doc.restore_state();

    // Company name (black — restored by restore_state above)
    doc.place_text_styled("NovaPeak Solutions", MARGIN + 54.0, 765.0, &bold(11.0));

    // Gray address / contact lines
    doc.save_state();
    doc.set_fill_color(mid_gray());
    doc.place_text_styled(
        "456 Innovation Drive, Suite 200",
        MARGIN + 54.0, 753.0, &regular(9.0),
    );
    doc.place_text_styled("San Francisco, CA 94102",        MARGIN + 54.0, 742.0, &regular(9.0));
    doc.place_text_styled("info@novapeak.io  |  (415) 555-9200", MARGIN + 54.0, 731.0, &regular(9.0));
    doc.restore_state();
}

// ── invoice title + metadata ──────────────────────────────────────────────────

fn draw_invoice_header<W: std::io::Write>(doc: &mut PdfDocument<W>) {
    doc.place_text_styled("INVOICE", 392.0, 766.0, &bold(22.0));

    let rows: &[(&str, &str, f64)] = &[
        ("Invoice #:", "INV-2024-0042",     748.0),
        ("Date:",      "January 15, 2024",  736.0),
        ("Due Date:",  "February 15, 2024", 724.0),
    ];
    for &(label, value, y) in rows {
        doc.save_state();
        doc.set_fill_color(mid_gray());
        doc.place_text_styled(label, 392.0, y, &bold(9.0));
        doc.restore_state();
        doc.place_text_styled(value, 453.0, y, &regular(9.0));
    }
}

// ── horizontal rule ───────────────────────────────────────────────────────────

fn draw_rule<W: std::io::Write>(doc: &mut PdfDocument<W>, y: f64) {
    doc.save_state();
    doc.set_stroke_color(teal());
    doc.set_line_width(0.75);
    doc.move_to(MARGIN, y);
    doc.line_to(RIGHT, y);
    doc.stroke();
    doc.restore_state();
}

// ── bill-to block ─────────────────────────────────────────────────────────────

fn draw_bill_to<W: std::io::Write>(doc: &mut PdfDocument<W>) {
    doc.save_state();
    doc.set_fill_color(teal());
    doc.place_text_styled("BILL TO", MARGIN, 706.0, &bold(8.0));
    doc.restore_state();

    doc.place_text_styled("Acme Corporation", MARGIN, 694.0, &bold(11.0));

    doc.save_state();
    doc.set_fill_color(mid_gray());
    doc.place_text_styled("123 Business Ave",   MARGIN, 682.0, &regular(9.0));
    doc.place_text_styled("New York, NY 10001", MARGIN, 671.0, &regular(9.0));
    doc.place_text_styled("accounts@acme.com",  MARGIN, 660.0, &regular(9.0));
    doc.restore_state();
}

// ── line-items table ──────────────────────────────────────────────────────────

/// Returns the cursor's Y position after all rows are placed.
/// The caller uses this to position the totals section immediately below
/// the table rather than guessing a hardcoded coordinate.
fn draw_line_items<W: std::io::Write>(doc: &mut PdfDocument<W>) -> f64 {
    // Columns: Description | Qty | Unit Price | Total (sum = 468pt)
    let mut table = Table::new(vec![250.0, 50.0, 90.0, 78.0]);
    table.border_color = Color::rgb(0.75, 0.75, 0.75);

    let rect = Rect { x: MARGIN, y: 638.0, width: 468.0, height: 420.0 };
    let mut cursor = TableCursor::new(&rect);

    // Header row — Qty, Unit Price, Total are right-aligned to match data columns.
    let hs = CellStyle {
        background_color: Some(navy()),
        text_color: Some(Color::rgb(1.0, 1.0, 1.0)),
        font: FontRef::Builtin(BuiltinFont::HelveticaBold),
        font_size: 9.0,
        padding: 5.0,
        ..CellStyle::default()
    };
    let hs_right = CellStyle { text_align: TextAlign::Right, ..hs.clone() };
    let header = Row::new(vec![
        Cell::styled("DESCRIPTION", hs),
        Cell::styled("QTY",         hs_right.clone()),
        Cell::styled("UNIT PRICE",  hs_right.clone()),
        Cell::styled("TOTAL",       hs_right),
    ]);
    doc.fit_row(&table, &header, &mut cursor).expect("fit_row header");

    // Data rows — description is left-aligned; numeric columns are right-aligned.
    for (i, item) in ITEMS.iter().enumerate() {
        let ds = CellStyle {
            background_color: if i % 2 == 0 { Some(stripe_bg()) } else { None },
            font: FontRef::Builtin(BuiltinFont::Helvetica),
            font_size: 9.0,
            padding: 5.0,
            ..CellStyle::default()
        };
        let ds_right = CellStyle { text_align: TextAlign::Right, ..ds.clone() };
        let row = Row::new(vec![
            Cell::styled(item.description,            ds),
            Cell::styled(&item.qty.to_string(),       ds_right.clone()),
            Cell::styled(&fmt_money(item.unit_price), ds_right.clone()),
            Cell::styled(&fmt_money(item.total()),    ds_right),
        ]);
        match doc.fit_row(&table, &row, &mut cursor).expect("fit_row item") {
            FitResult::BoxFull | FitResult::BoxEmpty => {
                eprintln!("Warning: table unexpectedly full at row {}", i + 1);
                break;
            }
            FitResult::Stop => {}
        }
    }

    cursor.current_y()
}

// ── totals section ────────────────────────────────────────────────────────────

/// Renders subtotal, tax, and total using a borderless table so amounts are
/// properly right-aligned flush with the items table's TOTAL column.
///
/// `table_bottom` is `cursor.current_y()` from `draw_line_items`.
fn draw_totals<W: std::io::Write>(doc: &mut PdfDocument<W>, table_bottom: f64) {
    let subtotal: f64 = ITEMS.iter().map(|i| i.total()).sum();
    let tax_rate = 0.08_f64;
    let tax = subtotal * tax_rate;
    let total = subtotal + tax;

    // Borderless 2-column table: label (100pt) + amount (78pt) = 178pt.
    // x=362 to x=540 — amount column aligns exactly with the TOTAL column above.
    let mut totals_table = Table::new(vec![100.0, 78.0]);
    totals_table.border_width = 0.0;

    // Light separator 10pt below the items table.
    let sep_y = table_bottom - 10.0;
    doc.save_state();
    doc.set_stroke_color(Color::rgb(0.75, 0.75, 0.75));
    doc.set_line_width(0.5);
    doc.move_to(362.0, sep_y);
    doc.line_to(RIGHT, sep_y);
    doc.stroke();
    doc.restore_state();

    let rect = Rect { x: 362.0, y: sep_y, width: 178.0, height: 200.0 };
    let mut cursor = TableCursor::new(&rect);

    // Base style: 9pt Helvetica, right-aligned, 5pt padding.
    // Variants adjust font weight and color.
    let base = CellStyle {
        font: FontRef::Builtin(BuiltinFont::Helvetica),
        font_size: 9.0,
        padding: 5.0,
        text_align: TextAlign::Right,
        ..CellStyle::default()
    };
    let gray_label  = CellStyle { text_color: Some(mid_gray()), ..base.clone() };
    let total_label = CellStyle { font: FontRef::Builtin(BuiltinFont::HelveticaBold), ..base.clone() };
    let total_amt   = CellStyle {
        font: FontRef::Builtin(BuiltinFont::HelveticaBold),
        text_color: Some(navy()),
        ..base.clone()
    };

    doc.fit_row(&totals_table, &Row::new(vec![
        Cell::styled("Subtotal:", gray_label.clone()),
        Cell::styled(&fmt_money(subtotal), base.clone()),
    ]), &mut cursor).expect("fit_row subtotal");

    doc.fit_row(&totals_table, &Row::new(vec![
        Cell::styled(&format!("Tax ({:.0}%):", tax_rate * 100.0), gray_label),
        Cell::styled(&fmt_money(tax), base),
    ]), &mut cursor).expect("fit_row tax");

    // Bold navy rule between tax and total.
    let rule_y = cursor.current_y();
    doc.save_state();
    doc.set_stroke_color(navy());
    doc.set_line_width(1.0);
    doc.move_to(362.0, rule_y);
    doc.line_to(RIGHT, rule_y);
    doc.stroke();
    doc.restore_state();

    doc.fit_row(&totals_table, &Row::new(vec![
        Cell::styled("TOTAL:", total_label),
        Cell::styled(&fmt_money(total), total_amt),
    ]), &mut cursor).expect("fit_row total");
}

// ── footer ────────────────────────────────────────────────────────────────────

fn draw_footer<W: std::io::Write>(doc: &mut PdfDocument<W>) {
    draw_rule(doc, 108.0);

    doc.save_state();
    doc.set_fill_color(mid_gray());
    doc.place_text_styled(
        "Payment Terms: Net 30  |  Please make checks payable to NovaPeak Solutions",
        MARGIN, 94.0, &regular(8.0),
    );
    doc.restore_state();

    doc.save_state();
    doc.set_fill_color(teal());
    doc.place_text_styled("Thank you for your business!", MARGIN, 80.0, &oblique(9.0));
    doc.restore_state();
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    std::fs::create_dir_all("examples/output").unwrap();
    let path = "examples/output/rust-invoice.pdf";
    let mut doc = PdfDocument::create(path).expect("create PDF");
    doc.set_compression(true);
    doc.set_info("Title", "Invoice INV-2024-0042");
    doc.set_info("Creator", "NovaPeak Solutions — generate_invoice example");

    doc.begin_page(PAGE_W, PAGE_H);
    draw_logo(&mut doc);
    draw_invoice_header(&mut doc);
    draw_rule(&mut doc, 718.0);
    draw_bill_to(&mut doc);
    let table_bottom = draw_line_items(&mut doc);
    draw_totals(&mut doc, table_bottom);
    draw_footer(&mut doc);
    doc.end_page().expect("end_page");

    doc.end_document().expect("end_document");
    println!("Written to {}", path);
}
