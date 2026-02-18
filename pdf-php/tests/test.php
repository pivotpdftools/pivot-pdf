<?php
/**
 * Integration tests for pdf-php extension.
 *
 * Run with:
 *   php -d extension=target/release/libpdf_php.so pdf-php/tests/test.php
 */

$pass = 0;
$fail = 0;

function assert_true(bool $cond, string $msg): void {
    global $pass, $fail;
    if ($cond) {
        $pass++;
    } else {
        $fail++;
        echo "FAIL: $msg\n";
    }
}

// ----------------------------------------------------------
// Test 1: File-based PDF creation
// ----------------------------------------------------------
$outFile = tempnam(sys_get_temp_dir(), 'pdf_') . '.pdf';

$doc = PdfDocument::create($outFile);
$doc->setInfo("Creator", "rust-pdf-php-test");
$doc->setInfo("Title", "Test Document");
$doc->beginPage(612.0, 792.0);
$doc->placeText("Hello from PHP!", 72.0, 720.0);
$doc->endPage();
$result = $doc->endDocument();

assert_true($result === null, "File-based endDocument returns null");
assert_true(file_exists($outFile), "PDF file was created");

$bytes = file_get_contents($outFile);
assert_true(str_starts_with($bytes, '%PDF-'), "File starts with %PDF-");
assert_true(str_contains($bytes, 'Hello from PHP!'), "File contains text");
assert_true(
    str_contains($bytes, 'rust-pdf-php-test'),
    "File contains Creator info"
);

unlink($outFile);
echo "Test 1 (file-based): OK\n";

// ----------------------------------------------------------
// Test 2: In-memory PDF creation
// ----------------------------------------------------------
$doc = PdfDocument::createInMemory();
$doc->beginPage(612.0, 792.0);
$doc->placeText("In-memory test", 72.0, 720.0);
$doc->endPage();
$bytes = $doc->endDocument();

assert_true(is_string($bytes), "In-memory endDocument returns string");
assert_true(strlen($bytes) > 0, "In-memory PDF is non-empty");
assert_true(
    str_starts_with($bytes, '%PDF-'),
    "In-memory starts with %PDF-"
);
assert_true(
    str_contains($bytes, 'In-memory test'),
    "In-memory contains text"
);

echo "Test 2 (in-memory): OK\n";

// ----------------------------------------------------------
// Test 3: TextFlow across pages
// ----------------------------------------------------------
$outFile = __DIR__ . '/php-textflow_output.pdf';
$doc = PdfDocument::create($outFile);

$times = new TextStyle('Times-Roman', 11.0);
$courier = new TextStyle('Courier', 10.0);

$tf = new TextFlow();
$tf->addText("TextFlow Demo (PHP)\n\n", new TextStyle("Helvetica-Bold"));
$tf->addText("This document demonstrates the TextFlow feature of the "
         . "rust-pdf library. Text is automatically wrapped within a "
         . "bounding box and flows across multiple pages when the box "
         . "is full.\n\n", new TextStyle());

// Demonstrate Times-Roman
$tf->addText(
    "This paragraph is set in Times-Roman at 11pt. "
    . "The quick brown fox jumps over the lazy dog.\n\n",
    $times
);

// Demonstrate Courier
$tf->addText(
    "This line is in Courier at 10pt (monospaced).\n\n",
    $courier
);

// Generate several paragraphs of text to fill multiple pages.
for($i=1; $i<=6; $i++) {
    $tf->addText("Section $i\n", new TextStyle("Helvetica-Bold"));

    $tf->addText("Lorem ipsum dolor sit amet, consectetur adipiscing "
        . "elit. Sed do eiusmod tempor incididunt ut labore et "
        . "dolore magna aliqua. Ut enim ad minim veniam, quis "
        . "nostrud exercitation ullamco laboris nisi ut aliquip "
        . "ex ea commodo consequat. Duis aute irure dolor in "
        . "reprehenderit in voluptate velit esse cillum dolore "
        . "eu fugiat nulla pariatur. Excepteur sint occaecat "
        . "cupidatat non proident, sunt in culpa qui officia "
        . "deserunt mollit anim id est laborum.\n\n", new TextStyle());
    $tf->addText(" this is bold ", new TextStyle("Helvetica-Bold"));
    $tf->addText(
        "Curabitur pretium tincidunt lacus. Nulla gravida orci "
            . "a odio. Nullam varius, turpis et commodo pharetra, "
            . "est eros bibendum elit, nec luctus magna felis "
            . "sollicitudin mauris. Integer in mauris eu nibh euismod "
            . "gravida. Duis ac tellus et risus vulputate vehicula. "
            . "Donec lobortis risus a elit. Etiam tempor. Ut "
            . "ullamcorper, ligula ut dictum pharetra, nisi nunc "
            . "fringilla magna, in commodo elit erat nec turpis. "
            . "Ut pharetra augue nec augue.\n\n",
        new TextStyle(),
    );
}

$tf->addText("End of document.", new TextStyle("Helvetica-Bold"));

$rect = new Rect(72.0, 720.0, 468.0, 648.0);
$pages = 0;

while (true) {
    $doc->beginPage(612.0, 792.0);
    $result = $doc->fitTextflow($tf, $rect);
    $doc->endPage();
    $pages++;

    if ($result === "stop") break;
    if ($result === "box_empty") break;
    // box_full => continue to next page
}

$doc->endDocument();

assert_true($pages > 1, "TextFlow spans multiple pages (got $pages)");
assert_true(
    $result === "stop",
    "TextFlow finished with 'stop' result"
);

$bytes = file_get_contents($outFile);
assert_true(
    str_contains($bytes, 'Lorem'),
    "TextFlow PDF contains text"
);
// Trailing space after " this is bold " must carry over so
// "Curabitur" has a leading space in the content stream.
assert_true(
    str_contains($bytes, '( Curabitur)'),
    "Space preserved between text flow spans"
);

// unlink($outFile);
echo "Test 3 (textflow) $outFile: OK\n";

// ----------------------------------------------------------
// Test 4: TextStyle defaults
// ----------------------------------------------------------
$style = new TextStyle();
assert_true($style->font_name === "Helvetica", "Default font is Helvetica");
assert_true($style->font_size === 12.0, "Default font_size is 12.0");

$style2 = new TextStyle("Helvetica-Bold", 18.0);
assert_true(
    $style2->font_name === "Helvetica-Bold",
    "Custom font is Helvetica-Bold"
);
assert_true($style2->font_size === 18.0, "Custom font_size is 18.0");

echo "Test 4 (TextStyle): OK\n";

// ----------------------------------------------------------
// Test 5: Rect properties
// ----------------------------------------------------------
$rect = new Rect(10.0, 20.0, 300.0, 400.0);
assert_true($rect->x === 10.0, "Rect x");
assert_true($rect->y === 20.0, "Rect y");
assert_true($rect->width === 300.0, "Rect width");
assert_true($rect->height === 400.0, "Rect height");

echo "Test 5 (Rect): OK\n";

// ----------------------------------------------------------
// Test 6: Error on double endDocument
// ----------------------------------------------------------
$doc = PdfDocument::createInMemory();
$doc->beginPage(612.0, 792.0);
$doc->endPage();
$doc->endDocument();

$threw = false;
try {
    $doc->endDocument();
} catch (Throwable $e) {
    $threw = true;
}
assert_true($threw, "Double endDocument throws");

echo "Test 6 (double end): OK\n";

// ----------------------------------------------------------
// Test 7: TextStyle with Times-Roman font
// ----------------------------------------------------------
$doc = PdfDocument::createInMemory();
$doc->beginPage(612.0, 792.0);
$tf = new TextFlow();
$tf->addText("Times text", new TextStyle("Times-Roman"));
$rect = new Rect(72.0, 720.0, 468.0, 648.0);
$result = $doc->fitTextflow($tf, $rect);
$doc->endPage();
$bytes = $doc->endDocument();

assert_true($result === "stop", "Times-Roman textflow stops");
assert_true(
    str_contains($bytes, '/F5'),
    "Times-Roman uses F5 resource"
);

echo "Test 7 (Times-Roman): OK\n";

// ----------------------------------------------------------
// Test 8: TrueType font (mirrors generate_truetype.rs)
// ----------------------------------------------------------
$outFile = __DIR__ . '/php-truetype_output.pdf';
$fontPath = __DIR__ . '/../../pdf-core/tests/fixtures/DejaVuSans.ttf';
$doc = PdfDocument::create($outFile);
$doc->setCompression(true);
$doc->setInfo("Creator", "rust-pdf-php-test");
$doc->setInfo("Title", "TrueType Font Example");

$ttHandle = $doc->loadFontFile($fontPath);
assert_true(is_int($ttHandle), "loadFontFile returns int handle");
assert_true($ttHandle >= 0, "Font handle is non-negative");

$ttStyle = TextStyle::truetype($ttHandle, 14.0);
$ttSmall = TextStyle::truetype($ttHandle, 11.0);
$builtin = new TextStyle();
$bold = new TextStyle("Helvetica-Bold", 14.0);

// --- Page 1: Direct text placement via TextFlow ---
$tf1 = new TextFlow();
$tf1->addText("TrueType Font Demo\n", $bold);
$tf1->addText(
    "This line uses an embedded TrueType font (DejaVu Sans).\n",
    $ttStyle
);
$tf1->addText("This line uses builtin Helvetica.\n", $builtin);
$tf1->addText(
    "Mixed fonts on the same page work correctly.\n",
    $ttSmall
);

$rect = new Rect(72.0, 720.0, 468.0, 648.0);
$doc->beginPage(612.0, 792.0);
$result = $doc->fitTextflow($tf1, $rect);
$doc->endPage();
assert_true($result === "stop", "TT page 1 textflow stops");

// --- Pages 2+: TextFlow with mixed fonts ---
$tf2 = new TextFlow();
$tf2->addText(
    "TextFlow with TrueType\n\n",
    TextStyle::truetype($ttHandle, 16.0)
);
$tf2->addText(
    "This paragraph is set in DejaVu Sans via an "
    . "embedded TrueType font. The text flows naturally "
    . "within the bounding box and wraps at word "
    . "boundaries just like builtin fonts.\n\n",
    $ttStyle
);
$tf2->addText("Mixing fonts: ", $builtin);
$tf2->addText("this is Helvetica, ", $builtin);
$tf2->addText("and this is DejaVu Sans. ", $ttStyle);
$tf2->addText(
    "Both can appear in the same TextFlow.\n\n",
    $builtin
);

for ($i = 1; $i <= 4; $i++) {
    $tf2->addText("Section $i ", $bold);
    $tf2->addText(
        "Lorem ipsum dolor sit amet, consectetur "
        . "adipiscing elit. Sed do eiusmod tempor "
        . "incididunt ut labore et dolore magna aliqua. "
        . "Ut enim ad minim veniam, quis nostrud "
        . "exercitation ullamco laboris nisi ut aliquip "
        . "ex ea commodo consequat.\n\n",
        $ttSmall
    );
}

$tf2->addText("End of document.", $bold);

$pageCount = 1; // already wrote page 1
while (true) {
    $doc->beginPage(612.0, 792.0);
    $result = $doc->fitTextflow($tf2, $rect);
    $doc->endPage();
    $pageCount++;

    if ($result === "stop") break;
    if ($result === "box_empty") break;
}

$doc->endDocument();

assert_true(
    $pageCount > 1,
    "TT TextFlow spans multiple pages (got $pageCount)"
);
assert_true(
    $result === "stop",
    "TT TextFlow finished with 'stop'"
);

$bytes = file_get_contents($outFile);
assert_true(
    str_starts_with($bytes, '%PDF-'),
    "TT PDF starts with %PDF-"
);
// Type0 composite font structure
assert_true(
    str_contains($bytes, '/Subtype /Type0'),
    "TT PDF contains Type0 font"
);
assert_true(
    str_contains($bytes, '/Subtype /CIDFontType2'),
    "TT PDF contains CIDFontType2"
);
assert_true(
    str_contains($bytes, '/Type /FontDescriptor'),
    "TT PDF contains FontDescriptor"
);
assert_true(
    str_contains($bytes, '/Encoding /Identity-H'),
    "TT PDF has Identity-H encoding"
);
// ToUnicode CMap for copy/paste support
assert_true(
    str_contains($bytes, 'beginbfchar'),
    "TT PDF has ToUnicode CMap"
);
// Hex-encoded glyph IDs (TrueType text)
assert_true(
    str_contains($bytes, '> Tj'),
    "TT PDF has hex-encoded text"
);
// Builtin font also present (mixed page)
assert_true(
    str_contains($bytes, '/Subtype /Type1'),
    "TT PDF also contains builtin Type1 font"
);

echo "Test 8 (TrueType) $outFile: OK\n";

// ----------------------------------------------------------
// Test 9: Image support (mirrors generate_images.rs)
// ----------------------------------------------------------
$outFile = __DIR__ . '/php-images_output.pdf';
$fixturesDir = __DIR__ . '/../../pdf-core/tests/fixtures';
$doc = PdfDocument::create($outFile);
$doc->setCompression(true);
$doc->setInfo("Creator", "rust-pdf-php-test");
$doc->setInfo("Title", "Image Support Demo");

// Load images
$jpeg = $doc->loadImageFile("$fixturesDir/test.jpg");
assert_true(is_int($jpeg), "loadImageFile JPEG returns int handle");
assert_true($jpeg >= 0, "JPEG handle is non-negative");

$png = $doc->loadImageFile("$fixturesDir/test.png");
assert_true(is_int($png), "loadImageFile PNG returns int handle");

$pngAlpha = $doc->loadImageFile("$fixturesDir/test_alpha.png");
assert_true(is_int($pngAlpha), "loadImageFile RGBA PNG returns int handle");

// Load from bytes
$jpegBytes = file_get_contents("$fixturesDir/test.jpg");
$jpegFromBytes = $doc->loadImageBytes($jpegBytes);
assert_true(is_int($jpegFromBytes), "loadImageBytes returns int handle");

// Page 1: All four fit modes with text
$doc->beginPage(612.0, 792.0);
$doc->placeText("Image Support Demo (PHP)", 72.0, 750.0);

// Fit mode (JPEG)
$doc->placeText("Fit (JPEG)", 72.0, 700.0);
$doc->placeImage($jpeg, new Rect(72.0, 100.0, 200.0, 150.0), "fit");

// Stretch mode (PNG)
$doc->placeText("Stretch (PNG)", 320.0, 700.0);
$doc->placeImage($png, new Rect(320.0, 100.0, 200.0, 150.0), "stretch");

// Fill mode (RGBA PNG)
$doc->placeText("Fill (PNG Alpha)", 72.0, 480.0);
$doc->placeImage($pngAlpha, new Rect(72.0, 320.0, 200.0, 150.0), "fill");

// None mode (PNG) — uses default fit param
$doc->placeText("None (PNG)", 320.0, 480.0);
$doc->placeImage($png, new Rect(320.0, 320.0, 200.0, 150.0), "none");

$doc->endPage();

// Page 2: Same JPEG on second page (XObject reused)
$doc->beginPage(612.0, 792.0);
$doc->placeText("Same JPEG on page 2 (XObject reused)", 72.0, 750.0);
$doc->placeImage($jpeg, new Rect(72.0, 100.0, 468.0, 600.0), "fit");
$doc->endPage();

$doc->endDocument();

$bytes = file_get_contents($outFile);
assert_true(
    str_starts_with($bytes, '%PDF-'),
    "Images PDF starts with %PDF-"
);
assert_true(
    str_contains($bytes, '/Subtype /Image'),
    "Images PDF contains Image XObject"
);
assert_true(
    str_contains($bytes, '/Filter /DCTDecode'),
    "Images PDF has DCTDecode for JPEG"
);
assert_true(
    str_contains($bytes, '/ColorSpace /DeviceRGB'),
    "Images PDF has DeviceRGB color space"
);
assert_true(
    str_contains($bytes, '/SMask'),
    "Images PDF has SMask for RGBA PNG"
);
assert_true(
    str_contains($bytes, '/XObject'),
    "Images PDF has XObject in Resources"
);
assert_true(
    str_contains($bytes, '/Count 2'),
    "Images PDF has 2 pages"
);

// Invalid data returns error
$threw = false;
try {
    $doc2 = PdfDocument::createInMemory();
    $doc2->loadImageBytes("not-an-image");
} catch (Throwable $e) {
    $threw = true;
}
assert_true($threw, "Invalid image data throws");

echo "Test 9 (Images) $outFile: OK\n";

// ----------------------------------------------------------
// Test 10: Tables (mirrors generate_tables.rs)
// ----------------------------------------------------------
$outFile = __DIR__ . '/php-tables_output.pdf';

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
$headerBg   = new Color(0.2, 0.3, 0.5);
$headerText = new Color(1.0, 1.0, 1.0);
$headerStyle->setBackgroundColor($headerBg);
$headerStyle->setTextColor($headerText);

$headerRow = new Row([
    Cell::styled("ID",          $headerStyle),
    Cell::styled("Name",        $headerStyle),
    Cell::styled("Department",  $headerStyle),
    Cell::styled("Status",      $headerStyle),
    Cell::styled("Amount ($)",  $headerStyle),
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

$doc = PdfDocument::create($outFile);
$doc->setCompression(true);
$doc->setInfo("Title", "Database Report Example");
$doc->setInfo("Creator", "rust-pdf-php-test");

$pageWidth  = 612.0;
$pageHeight = 792.0;
$margin     = 72.0;
$tableRect  = new Rect($margin, $pageHeight - $margin, $pageWidth - 2 * $margin, $pageHeight - 2 * $margin);

$doc->beginPage($pageWidth, $pageHeight);
$cursor = new TableCursor($tableRect);
$pageCount = 1;

$idx = 0;
$rowCount = count($dbRows);
while ($idx < $rowCount) {
    // Repeat header at the top of every page.
    if ($cursor->isFirstRow()) {
        $r = $doc->fitRow($table, $headerRow, $cursor);
        if ($r === 'box_empty') {
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
        $pageCount++;
    } else {
        // box_empty — rect too small
        break;
    }
}

$doc->endPage();
$doc->endDocument();

assert_true($pageCount > 1, "Tables PDF spans multiple pages (got $pageCount)");

$bytes = file_get_contents($outFile);
assert_true(str_starts_with($bytes, '%PDF-'), "Tables PDF starts with %PDF-");
// PDF structure: page tree, font resources
assert_true(str_contains($bytes, '/Type /Pages'), "Tables PDF has page tree");
assert_true(str_contains($bytes, '/Count ' . $pageCount), "Tables PDF has correct page count");
assert_true(str_contains($bytes, '/Font'), "Tables PDF has font resources");
// Compression filter is present (setCompression was enabled)
assert_true(str_contains($bytes, '/Filter /FlateDecode'), "Tables PDF uses compression");

echo "Test 10 (Tables) $outFile: OK\n";

// ----------------------------------------------------------
// Summary
// ----------------------------------------------------------
echo "\n";
echo "Results: $pass passed, $fail failed\n";

if ($fail > 0) {
    exit(1);
}
echo "All tests passed!\n";
