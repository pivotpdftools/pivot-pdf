<?php
/**
 * Page numbering example using openPage().
 *
 * Mirrors: pdf-core/examples/generate_page_numbers.rs
 *
 * Demonstrates the "Page X of Y" pattern where the total page count is
 * unknown until all pages have been written:
 *
 * 1. Write all pages of content using the standard textflow loop.
 * 2. Call pageCount() to get the total.
 * 3. Loop back over pages using openPage(i) to add footer overlays.
 *
 * Run with:
 *   php -d extension=target/release/libpdf_php.so pdf-php/examples/generate_page_numbers.php
 */

@mkdir(__DIR__ . '/../../output', 0755, true);
$path = __DIR__ . '/../../output/php-page-numbers.pdf';

$doc = PdfDocument::create($path);
$doc->setCompression(true);
$doc->setInfo("Creator", "rust-pdf-php");
$doc->setInfo("Title", "Page Numbering Example");

$pageWidth     = 612.0;
$pageHeight    = 792.0;
$margin        = 72.0;
$contentWidth  = $pageWidth - 2.0 * $margin;
$contentHeight = $pageHeight - 2.0 * $margin - 40.0; // leave room for footer

$bodyStyle   = new TextStyle("Times-Roman", 12.0);
$boldStyle   = new TextStyle("Helvetica-Bold", 12.0);
$footerStyle = new TextStyle("Helvetica", 9.0);

// Build enough content to fill several pages
$lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. "
    . "Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. "
    . "Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris. "
    . "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum. "
    . "Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia. ";

$flow = new TextFlow();
for ($i = 1; $i <= 8; $i++) {
    $flow->addText("Section $i. ", $boldStyle);
    for ($j = 0; $j < 4; $j++) {
        $flow->addText($lorem, $bodyStyle);
    }
    $flow->addText("\n\n", $bodyStyle);
}

$contentRect = new Rect($margin, $pageHeight - $margin, $contentWidth, $contentHeight);

// --- Pass 1: write all content pages ---
while (true) {
    $doc->beginPage($pageWidth, $pageHeight);
    $result = $doc->fitTextflow($flow, $contentRect);
    $doc->endPage();

    if ($result === "stop") break;
    if ($result === "box_empty") break;
    // box_full => continue to next page
}

// --- Pass 2: add "Page X of Y" footer to every page ---
$total = $doc->pageCount();
echo "Total pages: $total\n";

for ($i = 1; $i <= $total; $i++) {
    $doc->openPage($i);
    $doc->placeTextStyled("Page $i of $total", $margin, 28.0, $footerStyle);
    $doc->endPage();
}

$doc->endDocument();

echo "Generated: $path\n";
