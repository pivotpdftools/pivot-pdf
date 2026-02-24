<?php
/**
 * TextFlow example — multi-page text with multiple fonts.
 *
 * Mirrors: pdf-core/examples/generate_textflow.rs
 *
 * Run with:
 *   php -d extension=target/release/libpdf_php.so pdf-php/examples/generate_textflow.php
 */

@mkdir(__DIR__ . '/../../output', 0755, true);
$path = __DIR__ . '/../../output/php-textflow.pdf';

$doc = PdfDocument::create($path);
$doc->setCompression(true);
$doc->setInfo("Creator", "rust-pdf-php");
$doc->setInfo("Title", "TextFlow Example");

$bold    = new TextStyle("Helvetica-Bold", 12.0);
$normal  = new TextStyle();
$times   = new TextStyle("Times-Roman", 11.0);
$courier = new TextStyle("Courier", 10.0);

$tf = new TextFlow();
$tf->addText("TextFlow Demo\n\n", $bold);
$tf->addText(
    "This document demonstrates the TextFlow feature of the "
    . "rust-pdf library. Text is automatically wrapped within a "
    . "bounding box and flows across multiple pages when the box "
    . "is full.\n\n",
    $normal
);

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
for ($i = 1; $i <= 6; $i++) {
    $tf->addText("Section $i\n", $bold);
    $tf->addText(
        "Lorem ipsum dolor sit amet, consectetur adipiscing "
        . "elit. Sed do eiusmod tempor incididunt ut labore et "
        . "dolore magna aliqua. Ut enim ad minim veniam, quis "
        . "nostrud exercitation ullamco laboris nisi ut aliquip "
        . "ex ea commodo consequat. Duis aute irure dolor in "
        . "reprehenderit in voluptate velit esse cillum dolore "
        . "eu fugiat nulla pariatur. Excepteur sint occaecat "
        . "cupidatat non proident, sunt in culpa qui officia "
        . "deserunt mollit anim id est laborum.\n\n",
        $normal
    );
    $tf->addText(" this is bold ", $bold);
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
        $normal
    );
}

$tf->addText("End of document.", $bold);

// 1-inch margins on US Letter (612x792pt).
$rect = new Rect(72.0, 720.0, 468.0, 648.0);

$pageCount = 0;
while (true) {
    $doc->beginPage(612.0, 792.0);
    $result = $doc->fitTextflow($tf, $rect);
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
