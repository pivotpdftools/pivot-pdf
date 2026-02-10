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
assert_true($style->font === "Helvetica", "Default font is Helvetica");
assert_true($style->font_size === 12.0, "Default font_size is 12.0");

$style2 = new TextStyle("Helvetica-Bold", 18.0);
assert_true(
    $style2->font === "Helvetica-Bold",
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
// Summary
// ----------------------------------------------------------
echo "\n";
echo "Results: $pass passed, $fail failed\n";

if ($fail > 0) {
    exit(1);
}
echo "All tests passed!\n";
