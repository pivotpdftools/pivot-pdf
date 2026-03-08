<?php
/**
 * AcroForm text fields example.
 *
 * Mirrors: examples/rust/generate_form_fields.rs
 *
 * Run with:
 *   php -d extension=target/release/libpdf_php.so examples/php/generate_form_fields.php
 */

@mkdir(__DIR__ . '/../output', 0755, true);
$path = __DIR__ . '/../output/php-form-fields.pdf';

$doc = PdfDocument::create($path);
$doc->setCompression(true);
$doc->setInfo("Creator", "pivot-pdf-php");
$doc->setInfo("Title", "Form Fields Example");

$doc->beginPage(612.0, 792.0);

// Labels
$doc->placeText("Full Name:", 72.0, 718.0);
$doc->placeText("Email:",     72.0, 688.0);
$doc->placeText("Phone:",     72.0, 658.0);
$doc->placeText("Comments:",  72.0, 608.0);

// Fillable text fields (invisible — viewer renders its own chrome)
// A line is drawn under each field at the bottom edge of its rect.
$doc->setLineWidth(0.5);

$fields = [
    ["full_name", 180.0, 706.0, 300.0, 18.0],
    ["email",     180.0, 676.0, 300.0, 18.0],
    ["phone",     180.0, 646.0, 200.0, 18.0],
    ["comments",  180.0, 576.0, 300.0, 48.0],
];

foreach ($fields as [$name, $x, $y, $width, $height]) {
    $doc->addTextField($name, new Rect($x, $y, $width, $height));
    $doc->moveTo($x, $y);
    $doc->lineTo($x + $width, $y);
    $doc->stroke();
}

$doc->endPage();
$doc->endDocument();

echo "Generated: $path\n";
