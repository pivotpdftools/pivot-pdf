<?php
/**
 * TrueType font example — embedded font with multi-page TextFlow.
 *
 * Mirrors: examples/rust/generate_truetype.rs
 *
 * Uses DejaVu Sans from the test fixtures by default. Pass a font path
 * as the first argument to use a different font:
 *   php ... generate_truetype.php /path/to/font.ttf
 *
 * Run with:
 *   php -d extension=target/release/libpdf_php.so examples/php/generate_truetype.php
 */

@mkdir(__DIR__ . '/../output', 0755, true);
$path     = __DIR__ . '/../output/php-truetype.pdf';
$fontPath = $argv[1] ?? __DIR__ . '/../../pdf-core/tests/fixtures/DejaVuSans.ttf';

$doc = PdfDocument::create($path);
$doc->setCompression(true);
$doc->setInfo("Creator", "rust-pdf-php");
$doc->setInfo("Title", "TrueType Font Example");

$ttHandle = $doc->loadFontFile($fontPath);
$ttStyle  = TextStyle::truetype($ttHandle, 14.0);
$ttSmall  = TextStyle::truetype($ttHandle, 11.0);
$builtin  = new TextStyle();
$bold     = new TextStyle("Helvetica-Bold", 14.0);

// --- Page 1: direct placement via TextFlow ---
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
$doc->fitTextflow($tf1, $rect);
$doc->endPage();

// --- Pages 2+: TextFlow with mixed fonts ---
$tf2 = new TextFlow();
$tf2->addText("TextFlow with TrueType\n\n", TextStyle::truetype($ttHandle, 16.0));
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
$tf2->addText("Both can appear in the same TextFlow.\n\n", $builtin);

// Add enough text to demonstrate multi-page flow
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
    if ($result === "box_empty") {
        fwrite(STDERR, "Warning: bounding box too small\n");
        break;
    }
    // box_full => continue to next page
}

$doc->endDocument();

echo "Generated: $path ($pageCount pages)\n";
