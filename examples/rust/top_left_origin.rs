/// Demonstrates the top-left coordinate origin.
///
/// In this mode, (x, y) = (0, 0) is the top-left corner of the page.
/// y increases downward, matching the mental model of screen/web layouts.
use pivot_pdf::{
    BuiltinFont, Cell, CellStyle, Color, DocumentOptions, FontRef, Origin, PdfDocument, Rect, Row,
    Table, TableCursor, TextAlign, TextFlow, TextStyle,
};

const PAGE_W: f64 = 612.0;
const PAGE_H: f64 = 792.0;
const MARGIN: f64 = 72.0;

fn main() {
    std::fs::create_dir_all("examples/output").unwrap();
    let path = "examples/output/rust-top-left-origin.pdf";

    let opts = DocumentOptions { origin: Origin::TopLeft };
    let mut doc = PdfDocument::create(path, opts).unwrap();
    doc.set_info("Title", "Top-Left Origin Demo");
    doc.set_compression(true);

    doc.begin_page(PAGE_W, PAGE_H);

    // Title at the top of the page — y=36 means 36 points from the top.
    let title_style = TextStyle {
        font: FontRef::Builtin(BuiltinFont::HelveticaBold),
        font_size: 16.0,
    };
    doc.place_text_styled("Top-Left Origin Demo", MARGIN, 36.0, &title_style);

    // Subtitle
    let body_style = TextStyle {
        font: FontRef::Builtin(BuiltinFont::Helvetica),
        font_size: 11.0,
    };
    doc.place_text_styled(
        "(0, 0) is the top-left corner. y increases downward.",
        MARGIN,
        62.0,
        &body_style,
    );

    // Horizontal rule below title area
    doc.set_stroke_color(Color::rgb(0.5, 0.5, 0.5));
    doc.set_line_width(0.5);
    doc.move_to(MARGIN, 80.0);
    doc.line_to(PAGE_W - MARGIN, 80.0);
    doc.stroke();

    // Bounding box drawn with rect() — (x, y) is the top-left corner.
    doc.set_stroke_color(Color::rgb(0.2, 0.4, 0.8));
    doc.set_line_width(1.0);
    doc.rect(MARGIN, 100.0, PAGE_W - 2.0 * MARGIN, 60.0);
    doc.stroke();

    // Label inside the box
    let label_style = TextStyle {
        font: FontRef::Builtin(BuiltinFont::Helvetica),
        font_size: 10.0,
    };
    doc.place_text_styled(
        "rect(72, 100, 468, 60)  →  top-left corner at (72, 100)",
        MARGIN + 8.0,
        120.0,
        &label_style,
    );

    // TextFlow below the box — rect.y is the top of the text area.
    let text_rect = Rect {
        x: MARGIN,
        y: 180.0, // 180 pts from the top
        width: PAGE_W - 2.0 * MARGIN,
        height: 120.0,
    };

    let mut flow = TextFlow::new();
    flow.add_text(
        "This paragraph uses fit_textflow with a rect whose y-coordinate \
         (180) is measured from the top of the page. The library transparently \
         converts all coordinates to PDF space so you can think entirely in \
         top-left terms.",
        &body_style,
    );
    doc.fit_textflow(&mut flow, &text_rect).unwrap();

    // Simple two-column table — table_rect.y is the top of the table.
    let header_style = CellStyle {
        font: FontRef::Builtin(BuiltinFont::HelveticaBold),
        font_size: 10.0,
        padding: 6.0,
        text_align: TextAlign::Left,
        ..CellStyle::default()
    };
    let data_style = CellStyle {
        font_size: 10.0,
        padding: 6.0,
        ..CellStyle::default()
    };

    let table = Table::new(vec![220.0, PAGE_W - 2.0 * MARGIN - 220.0]);
    let table_rect = Rect {
        x: MARGIN,
        y: 320.0, // 320 pts from the top
        width: PAGE_W - 2.0 * MARGIN,
        height: 400.0,
    };

    let rows: &[(&str, &str)] = &[
        ("API", "TopLeft meaning of y"),
        ("place_text(x, y)", "y = distance from top of page"),
        ("move_to / line_to", "y = distance from top of page"),
        ("rect(x, y, w, h)", "y = top edge of rectangle"),
        ("fit_textflow(rect)", "rect.y = top of text area"),
        ("fit_row(cursor)", "cursor rect.y = top of table area"),
        ("place_image(rect)", "rect.y = top of image area"),
    ];

    let mut cursor = TableCursor::new(&table_rect);
    for (i, (api, meaning)) in rows.iter().enumerate() {
        let row = if i == 0 {
            Row::new(vec![
                Cell::styled(*api, header_style.clone()),
                Cell::styled(*meaning, header_style.clone()),
            ])
        } else {
            Row::new(vec![
                Cell::styled(*api, data_style.clone()),
                Cell::styled(*meaning, data_style.clone()),
            ])
        };
        doc.fit_row(&table, &row, &mut cursor).unwrap();
    }

    doc.end_page().unwrap();
    doc.end_document().unwrap();
    println!("Generated: {}", path);
}
