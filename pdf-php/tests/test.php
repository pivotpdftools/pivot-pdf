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
$outFile = __DIR__ . '/php-pdf.pdf';
$doc = PdfDocument::create($outFile);

$tf = new TextFlow();
$tf->addText("Normal text. ", new TextStyle());
$tf->addText("Bold text. ", new TextStyle(true, 14.0));

// Add enough text to fill multiple pages
$longText = str_repeat(
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ",
    80
);
$tf->addText($longText, new TextStyle());

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

// unlink($outFile);
echo "Test 3 (textflow) $outFile: OK\n";

// ----------------------------------------------------------
// Test 4: TextStyle defaults
// ----------------------------------------------------------
$style = new TextStyle();
assert_true($style->bold === false, "Default bold is false");
assert_true($style->font_size === 12.0, "Default font_size is 12.0");

$style2 = new TextStyle(true, 18.0);
assert_true($style2->bold === true, "Custom bold is true");
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
} catch (\Throwable $e) {
    $threw = true;
}
assert_true($threw, "Double endDocument throws");

echo "Test 6 (double end): OK\n";

// ----------------------------------------------------------
// Summary
// ----------------------------------------------------------
echo "\n";
echo "Results: $pass passed, $fail failed\n";

if ($fail > 0) {
    exit(1);
}
echo "All tests passed!\n";
