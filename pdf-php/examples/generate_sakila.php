<?php
/**
 * Large PDF report from the Sakila SQLite database.
 *
 * Mirrors: pdf-core/examples/generate_sakila.rs
 *
 * Queries rental history (3475 rows) and renders it as a multi-page landscape
 * table. The table header repeats on each page. A "Page X of Y" footer appears
 * in the lower-left corner of every page.
 *
 * Run with:
 *   php -d extension=target/release/libpdf_php.so \
 *       pdf-php/examples/generate_sakila.php /path/to/sakila.db
 */

if ($argc < 2) {
    fwrite(STDERR, "Usage: generate_sakila.php <path/to/sakila.db>\n");
    exit(1);
}
$dbPath = $argv[1];

@mkdir(__DIR__ . '/../../output', 0755, true);
$outPath = __DIR__ . '/../../output/php-sakila.pdf';

const PAGE_WIDTH  = 792.0; // landscape
const PAGE_HEIGHT = 612.0;
const MARGIN      = 36.0;
const FOOTER_H    = 20.0;

$tableX      = MARGIN;
$tableTop    = PAGE_HEIGHT - MARGIN;
$tableBottom = MARGIN + FOOTER_H;
$tableWidth  = PAGE_WIDTH - 2 * MARGIN;
$tableHeight = $tableTop - $tableBottom;

// Column widths sum to 720.0 (TABLE_WIDTH):
// ID | Date | Film Title | Year | Rating | Category | Length |
// First Name | Last Name | Email | Address | City | Postal
$colWidths = [30.0, 68.0, 85.0, 32.0, 35.0, 60.0, 38.0, 52.0, 52.0, 100.0, 75.0, 55.0, 38.0];
$headers   = [
    'ID', 'Date', 'Film Title', 'Year', 'Rating', 'Category', 'Length',
    'First Name', 'Last Name', 'Email', 'Address', 'City', 'Postal',
];

$sql = "
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

// --- Build header row ---
$headerStyle = new CellStyle();
$headerStyle->font_name = 'Helvetica-Bold';
$headerStyle->font_size = 7.0;
$headerStyle->padding   = 3.0;
$headerBg   = new Color(0.2, 0.3, 0.5);
$headerFg   = new Color(1.0, 1.0, 1.0);
$headerStyle->setBackgroundColor($headerBg);
$headerStyle->setTextColor($headerFg);

$headerCells = array_map(
    fn($h) => Cell::styled($h, $headerStyle),
    $headers
);
$headerRow = new Row($headerCells);

// --- Table config ---
$table = new Table($colWidths);

// --- Open database ---
$pdo = new PDO('sqlite:' . $dbPath);
$pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

// Use a cursor-style query to avoid loading all rows into memory at once.
$stmt = $pdo->query($sql, PDO::FETCH_NUM);

// --- PDF document ---
$doc = PdfDocument::create($outPath);
$doc->setCompression(true);
$doc->setInfo('Title', 'Sakila Rental Report');
$doc->setInfo('Creator', 'rust-pdf-php generate_sakila example');

$footerStyle = new TextStyle('Helvetica', 8.0);

$tableRect = new Rect($tableX, $tableTop, $tableWidth, $tableHeight);

$cellStyle = new CellStyle();
$cellStyle->font_name = 'Helvetica';
$cellStyle->font_size = 7.0;
$cellStyle->padding   = 3.0;

// Last names are single words that can't wrap; shrink the font to fit.
$lastNameStyle = new CellStyle();
$lastNameStyle->font_name = 'Helvetica';
$lastNameStyle->font_size = 7.0;
$lastNameStyle->padding   = 3.0;
$lastNameStyle->overflow  = 'shrink';

// Email addresses have no word-break characters so they can't wrap.
// Clip prevents them from visually overflowing into adjacent columns.
$emailStyle = new CellStyle();
$emailStyle->font_name = 'Helvetica';
$emailStyle->font_size = 7.0;
$emailStyle->padding   = 3.0;
$emailStyle->overflow  = 'clip';

define('LAST_NAME_COL', 8);
define('EMAIL_COL', 9);

$evenBg = new Color(0.95, 0.97, 1.0);
$oddBg  = new Color(1.0, 1.0, 1.0);

// --- Pass 1: stream rows into table pages ---
$doc->beginPage(PAGE_WIDTH, PAGE_HEIGHT);
$cursor   = new TableCursor($tableRect);
$rowIndex = 0;
$totalRows = 0;

// Prefetch first row so we can use a peek-style loop.
$nextDbRow = $stmt->fetch();

while ($nextDbRow !== false) {
    if ($cursor->isFirstRow()) {
        $r = $doc->fitRow($table, $headerRow, $cursor);
        if ($r === 'box_empty') {
            fwrite(STDERR, "Warning: bounding box too small to fit header\n");
            break;
        }
    }

    $values = array_map('strval', $nextDbRow);
    $cells  = [];
    foreach ($values as $i => $v) {
        if ($i === LAST_NAME_COL) {
            $style = $lastNameStyle;
        } elseif ($i === EMAIL_COL) {
            $style = $emailStyle;
        } else {
            $style = $cellStyle;
        }
        $cells[] = Cell::styled($v, $style);
    }
    $dataRow = new Row($cells);
    $bg = $rowIndex % 2 === 0 ? $evenBg : $oddBg;
    $dataRow->setBackgroundColor($bg);

    $result = $doc->fitRow($table, $dataRow, $cursor);
    if ($result === 'stop') {
        $nextDbRow = $stmt->fetch();
        $rowIndex++;
        $totalRows++;
    } elseif ($result === 'box_full') {
        $doc->endPage();
        $doc->beginPage(PAGE_WIDTH, PAGE_HEIGHT);
        $cursor->reset($tableRect);
    } else {
        fwrite(STDERR, "Warning: bounding box too small to fit any rows\n");
        break;
    }
}

$doc->endPage();

// --- Pass 2: add "Page X of Y" footer to every page ---
// Table row backgrounds leave a non-black rg fill color in the graphics state
// (set outside any q/Q block). PDF concatenates all content streams on a page,
// so the overlay inherits that color. Reset to black first so the footer text
// is visible.
$total = $doc->pageCount();
$black = new Color(0.0, 0.0, 0.0);
for ($i = 1; $i <= $total; $i++) {
    $doc->openPage($i);
    $doc->setFillColor($black);
    $doc->placeTextStyled("Page $i of $total", MARGIN, 16.0, $footerStyle);
    $doc->endPage();
}

$doc->endDocument();

echo "Written to $outPath ($total pages, $totalRows rows)\n";
