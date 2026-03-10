<?php
/**
 * Demonstrates the top-left coordinate origin.
 *
 * In this mode, (x, y) = (0, 0) is the top-left corner of the page.
 * y increases downward, matching the mental model of screen/web layouts.
 *
 * Mirrors: examples/rust/top_left_origin.rs
 *
 * Run with:
 *   php -d extension=target/release/libpdf_php.so examples/php/top_left_origin.php
 */

@mkdir(__DIR__ . '/../output', 0755, true);
$path = __DIR__ . '/../output/php-top-left-origin.pdf';

$opts = new PdfDocumentOptions();
$opts->origin = 'top-left';

$doc = PdfDocument::create($path, $opts);
$doc->setInfo("Title", "Top-Left Origin Demo (PHP)");
$doc->setCompression(true);

const PAGE_W = 612.0;
const PAGE_H = 792.0;
const MARGIN = 72.0;

$doc->beginPage(PAGE_W, PAGE_H);

// Title — y=36 means 36 points from the top.
$titleStyle = new TextStyle('Helvetica-Bold', 16.0);
$doc->placeTextStyled("Top-Left Origin Demo", MARGIN, 36.0, $titleStyle);

$bodyStyle = new TextStyle('Helvetica', 11.0);
$doc->placeTextStyled(
    "(0, 0) is the top-left corner. y increases downward.",
    MARGIN,
    62.0,
    $bodyStyle
);

// Horizontal rule
$doc->setStrokeColor(Color::gray(0.5));
$doc->setLineWidth(0.5);
$doc->moveTo(MARGIN, 80.0);
$doc->lineTo(PAGE_W - MARGIN, 80.0);
$doc->stroke();

// Bounding box — (x, y) is the top-left corner of the box.
$doc->setStrokeColor(new Color(0.2, 0.4, 0.8));
$doc->setLineWidth(1.0);
$doc->rect(MARGIN, 100.0, PAGE_W - 2 * MARGIN, 60.0);
$doc->stroke();

$labelStyle = new TextStyle('Helvetica', 10.0);
$doc->placeTextStyled(
    "rect(72, 100, 468, 60)  →  top-left corner at (72, 100)",
    MARGIN + 8.0,
    120.0,
    $labelStyle
);

// TextFlow — rect->y is the top of the text area.
$textRect = new Rect(MARGIN, 180.0, PAGE_W - 2 * MARGIN, 120.0);
$flow = new TextFlow();
$flow->addText(
    "This paragraph uses fitTextflow with a rect whose y-coordinate " .
    "(180) is measured from the top of the page. The library transparently " .
    "converts all coordinates to PDF space so you can think entirely in " .
    "top-left terms.",
    $bodyStyle
);
$doc->fitTextflow($flow, $textRect);

// Two-column table — tableRect->y is the top of the table.
$headerStyle = new CellStyle();
$headerStyle->fontName = 'Helvetica-Bold';
$headerStyle->fontSize = 10.0;
$headerStyle->padding = 6.0;

$dataStyle = new CellStyle();
$dataStyle->fontSize = 10.0;
$dataStyle->padding = 6.0;

$table = new Table([220.0, PAGE_W - 2 * MARGIN - 220.0]);
$tableRect = new Rect(MARGIN, 320.0, PAGE_W - 2 * MARGIN, 400.0);

$rows = [
    ["API",                "TopLeft meaning of y"],
    ["placeText(x, y)",    "y = distance from top of page"],
    ["moveTo / lineTo",    "y = distance from top of page"],
    ["rect(x, y, w, h)",  "y = top edge of rectangle"],
    ["fitTextflow(rect)",  "rect->y = top of text area"],
    ["fitRow(cursor)",     "cursor rect->y = top of table area"],
    ["placeImage(rect)",   "rect->y = top of image area"],
];

$cursor = new TableCursor($tableRect);
foreach ($rows as $i => [$api, $meaning]) {
    $style = $i === 0 ? $headerStyle : $dataStyle;
    $row = new Row([
        Cell::styled($api, $style),
        Cell::styled($meaning, $style),
    ]);
    $doc->fitRow($table, $row, $cursor);
}

$doc->endPage();
$doc->endDocument();

echo "Generated: $path\n";
