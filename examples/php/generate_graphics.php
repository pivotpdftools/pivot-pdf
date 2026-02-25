<?php
/**
 * Line graphics example.
 *
 * Mirrors: examples/rust/generate_graphics.rs
 *
 * Run with:
 *   php -d extension=target/release/libpdf_php.so examples/php/generate_graphics.php
 */

@mkdir(__DIR__ . '/../output', 0755, true);
$path = __DIR__ . '/../output/php-graphics.pdf';

$doc = PdfDocument::create($path);
$doc->setCompression(true);
$doc->setInfo("Creator", "rust-pdf-php");
$doc->setInfo("Title", "Line Graphics Demo");
$doc->beginPage(612.0, 792.0);

// Stroked rectangle (page border)
$doc->setStrokeColor(new Color(0.0, 0.0, 0.0));
$doc->setLineWidth(1.0);
$doc->rect(72.0, 72.0, 468.0, 648.0);
$doc->stroke();

// Filled rectangle (light gray background box)
$doc->setFillColor(Color::gray(0.9));
$doc->rect(100.0, 600.0, 200.0, 50.0);
$doc->fill();

// Diagonal line
$doc->setStrokeColor(new Color(0.0, 0.0, 1.0));
$doc->setLineWidth(2.0);
$doc->moveTo(100.0, 500.0);
$doc->lineTo(300.0, 550.0);
$doc->stroke();

// Triangle with fill and stroke
$doc->saveState();
$doc->setFillColor(new Color(1.0, 0.0, 0.0));
$doc->setStrokeColor(new Color(0.0, 0.0, 0.0));
$doc->setLineWidth(1.5);
$doc->moveTo(350.0, 400.0);
$doc->lineTo(450.0, 400.0);
$doc->lineTo(400.0, 480.0);
$doc->closePath();
$doc->fillStroke();
$doc->restoreState();

// Nested rectangles using save/restore to isolate state
$doc->saveState();
$doc->setStrokeColor(new Color(0.0, 0.5, 0.0));
$doc->setLineWidth(3.0);
$doc->rect(150.0, 200.0, 300.0, 150.0);
$doc->stroke();

$doc->setFillColor(new Color(0.8, 0.9, 0.8));
$doc->rect(180.0, 230.0, 240.0, 90.0);
$doc->fill();
$doc->restoreState();

// Add a label
$doc->placeText("Line Graphics Demo", 72.0, 740.0);

$doc->endPage();
$doc->endDocument();

echo "Generated: $path\n";
