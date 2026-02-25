<?php
/**
 * Database report example — multi-page table with streaming row placement.
 *
 * Mirrors: examples/rust/generate_tables.rs
 *
 * Demonstrates:
 * - Streaming row-by-row placement using fitRow() + TableCursor
 * - Header row repeated at the top of each new page via isFirstRow()
 * - Alternating row background colors
 *
 * Run with:
 *   php -d extension=target/release/libpdf_php.so examples/php/generate_tables.php
 */

@mkdir(__DIR__ . '/../output', 0755, true);
$path = __DIR__ . '/../output/php-tables.pdf';

$departments = ["Engineering", "Marketing", "Sales", "HR", "Finance", "Operations"];
$statuses    = ["Active", "Inactive", "Pending", "Suspended", "Active"];
$names       = [
    "Alice Johnson", "Bob Smith", "Carol White", "David Brown",
    "Emma Davis", "Frank Miller", "Grace Wilson", "Henry Moore",
    "Iris Taylor", "Jack Anderson",
];

// Header style: bold white text on dark background
$headerStyle = new CellStyle();
$headerStyle->font_name = "Helvetica-Bold";
$headerStyle->font_size = 9.0;
$headerStyle->padding   = 5.0;

$headerBgColor = new Color(0.2, 0.3, 0.5);
$headerStyle->setBackgroundColor($headerBgColor);

$headerFgColor = new Color(1.0, 1.0, 1.0);
$headerStyle->setTextColor($headerFgColor);

$headerRow = new Row([
    Cell::styled("ID",         $headerStyle),
    Cell::styled("Name",       $headerStyle),
    Cell::styled("Department", $headerStyle),
    Cell::styled("Status",     $headerStyle),
    Cell::styled("Amount ($)", $headerStyle),
]);

// Table config: column widths — ID | Name | Department | Status | Amount
$table = new Table([40.0, 120.0, 130.0, 90.0, 88.0]);
$table->setBorderWidth(0.5);

// Build 160 simulated database rows
$dbRows = [];
for ($i = 0; $i < 160; $i++) {
    $amount = 1000.0 + fmod($i * 137.5, 9000.0);
    $row = new Row([
        new Cell((string)($i + 1)),
        new Cell($names[$i % count($names)]),
        new Cell($departments[$i % count($departments)]),
        new Cell($statuses[$i % count($statuses)]),
        new Cell(number_format($amount, 2)),
    ]);
    $bg = $i % 2 === 0 ? new Color(0.95, 0.97, 1.0) : new Color(1.0, 1.0, 1.0);
    $row->setBackgroundColor($bg);
    $dbRows[] = $row;
}

$doc = PdfDocument::create($path);
$doc->setCompression(true);
$doc->setInfo("Title", "Database Report Example");
$doc->setInfo("Creator", "rust-pdf-php");

$pageWidth  = 612.0;
$pageHeight = 792.0;
$margin     = 72.0;
$tableRect  = new Rect($margin, $pageHeight - $margin, $pageWidth - 2 * $margin, $pageHeight - 2 * $margin);

$doc->beginPage($pageWidth, $pageHeight);
$cursor = new TableCursor($tableRect);

$idx      = 0;
$rowCount = count($dbRows);
while ($idx < $rowCount) {
    // Repeat header at the top of every page.
    if ($cursor->isFirstRow()) {
        $r = $doc->fitRow($table, $headerRow, $cursor);
        if ($r === 'box_empty') {
            fwrite(STDERR, "Warning: bounding box too small to fit header row\n");
            break;
        }
    }

    $result = $doc->fitRow($table, $dbRows[$idx], $cursor);
    if ($result === 'stop') {
        $idx++;
    } elseif ($result === 'box_full') {
        $doc->endPage();
        $doc->beginPage($pageWidth, $pageHeight);
        $cursor->reset($tableRect);
    } else {
        // box_empty — rect too small
        fwrite(STDERR, "Warning: bounding box too small to fit any rows\n");
        break;
    }
}

$doc->endPage();
$doc->endDocument();

echo "Written to $path\n";
