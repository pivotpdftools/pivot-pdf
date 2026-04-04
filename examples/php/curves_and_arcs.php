<?php
/**
 * Curves and arcs example.
 *
 * Mirrors: examples/rust/curves_and_arcs.rs
 *
 * Run with:
 *   php -d extension=target/release/libpdf_php.so examples/php/curves_and_arcs.php
 */

@mkdir(__DIR__ . '/../output', 0755, true);
$path = __DIR__ . '/../output/php-curves-and-arcs.pdf';

$doc = PdfDocument::create($path);
$doc->setCompression(true);
$doc->setInfo("Creator", "rust-pdf-php");
$doc->setInfo("Title", "Curves and Arcs Demo");
$doc->beginPage(612.0, 792.0);

$doc->placeText("Curves and Arcs Demo", 72.0, 750.0);

// --- Cubic Bezier curve (S-curve) ---
$doc->saveState();
$doc->setStrokeColor(new Color(0.0, 0.0, 0.8));
$doc->setLineWidth(2.0);
$doc->moveTo(80.0, 680.0);
$doc->curveTo(150.0, 730.0, 220.0, 630.0, 290.0, 680.0);
$doc->stroke();
$doc->restoreState();
$doc->placeText("Cubic Bezier (curveTo)", 300.0, 680.0);

// --- Quarter arc (pie slice) ---
$doc->saveState();
$doc->setStrokeColor(new Color(0.8, 0.0, 0.0));
$doc->setFillColor(new Color(1.0, 0.8, 0.8));
$doc->setLineWidth(2.0);
$cx = 130.0;
$cy = 580.0;
$r  = 60.0;
$doc->moveTo($cx, $cy);
$doc->lineTo($cx + $r, $cy);
$doc->arc($cx, $cy, $r, 0.0, 90.0);
$doc->closePath();
$doc->fillStroke();
$doc->restoreState();
$doc->placeText("Quarter arc (pie slice)", 220.0, 580.0);

// --- Half circle (dome) ---
$doc->saveState();
$doc->setFillColor(new Color(0.9, 0.9, 0.2));
$doc->setStrokeColor(new Color(0.5, 0.5, 0.0));
$doc->setLineWidth(1.5);
$doc->arc(130.0, 460.0, 60.0, 0.0, 180.0);
$doc->closePath();
$doc->fillStroke();
$doc->restoreState();
$doc->placeText("Half circle (arc 0°–180°)", 220.0, 480.0);

// --- Full circles ---
$doc->saveState();
$doc->setStrokeColor(new Color(0.0, 0.6, 0.0));
$doc->setLineWidth(2.0);
$doc->circle(130.0, 340.0, 50.0);
$doc->stroke();
$doc->restoreState();
$doc->placeText("Stroked circle", 200.0, 355.0);

$doc->saveState();
$doc->setFillColor(new Color(0.8, 0.4, 0.8));
$doc->circle(130.0, 220.0, 50.0);
$doc->fill();
$doc->restoreState();
$doc->placeText("Filled circle", 200.0, 235.0);

// --- Concentric rings ---
$doc->saveState();
$doc->setStrokeColor(new Color(0.0, 0.4, 0.8));
for ($i = 1; $i <= 4; $i++) {
    $doc->setLineWidth($i * 0.5);
    $doc->circle(480.0, 450.0, $i * 30.0);
    $doc->stroke();
}
$doc->restoreState();
$doc->placeText("Concentric rings", 430.0, 580.0);

// --- Arc used as a rounded cap ---
$doc->saveState();
$doc->setStrokeColor(new Color(0.5, 0.0, 0.5));
$doc->setLineWidth(1.5);
$doc->moveTo(400.0, 200.0);
$doc->lineTo(520.0, 200.0);
$doc->arc(520.0, 200.0, 20.0, -90.0, 90.0);
$doc->stroke();
$doc->restoreState();
$doc->placeText("Arc as rounded cap", 390.0, 235.0);

$doc->endPage();
$doc->endDocument();

echo "Generated: $path\n";
