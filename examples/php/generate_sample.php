<?php
/**
 * Basic PDF creation example.
 *
 * Mirrors: examples/rust/generate_sample.rs
 *
 * Run with:
 *   php -d extension=target/release/libpdf_php.so examples/php/generate_sample.php
 */

@mkdir(__DIR__ . '/../output', 0755, true);
$path = __DIR__ . '/../output/php-sample.pdf';

$doc = PdfDocument::create($path);
$doc->setCompression(true);
$doc->setInfo("Creator", "rust-pdf-php");
$doc->setInfo("Title", "A Test Document");
$doc->beginPage(612.0, 792.0);
$doc->placeText("Hello, PDF!", 72.0, 720.0);
$doc->placeText("Created by rust-pdf-php library.", 72.0, 700.0);
$doc->endPage();
$doc->endDocument();

echo "Generated: $path\n";
