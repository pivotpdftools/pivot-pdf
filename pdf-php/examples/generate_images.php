<?php
/**
 * Image support example — JPEG, PNG, and RGBA PNG with four fit modes.
 *
 * Mirrors: pdf-core/examples/generate_images.rs
 *
 * Run with:
 *   php -d extension=target/release/libpdf_php.so pdf-php/examples/generate_images.php
 */

@mkdir(__DIR__ . '/../../output', 0755, true);
$path        = __DIR__ . '/../../output/php-images.pdf';
$fixturesDir = __DIR__ . '/../../pdf-core/tests/fixtures';

$doc = PdfDocument::create($path);
$doc->setCompression(true);
$doc->setInfo("Creator", "rust-pdf-php");
$doc->setInfo("Title", "Image Support Demo");

// Load images
$jpeg     = $doc->loadImageFile("$fixturesDir/test.jpg");
$png      = $doc->loadImageFile("$fixturesDir/test.png");
$pngAlpha = $doc->loadImageFile("$fixturesDir/test_alpha.png");

// --- Page 1: all four fit modes ---
$doc->beginPage(612.0, 792.0);
$doc->placeText("Image Support Demo", 72.0, 750.0);

// Fit mode — scales to fit, preserves aspect ratio
$doc->placeText("Fit (JPEG)", 72.0, 700.0);
$doc->placeImage($jpeg, new Rect(72.0, 100.0, 200.0, 150.0), "fit");

// Stretch mode — fills rect exactly, may distort
$doc->placeText("Stretch (PNG)", 320.0, 700.0);
$doc->placeImage($png, new Rect(320.0, 100.0, 200.0, 150.0), "stretch");

// Fill mode — scales to cover, clips overflow
$doc->placeText("Fill (PNG Alpha)", 72.0, 480.0);
$doc->placeImage($pngAlpha, new Rect(72.0, 320.0, 200.0, 150.0), "fill");

// None mode — natural size (1px = 1pt)
$doc->placeText("None (PNG)", 320.0, 480.0);
$doc->placeImage($png, new Rect(320.0, 320.0, 200.0, 150.0), "none");

$doc->endPage();

// --- Page 2: same JPEG (demonstrates XObject reuse) ---
$doc->beginPage(612.0, 792.0);
$doc->placeText("Same JPEG on page 2 (XObject reused)", 72.0, 750.0);
$doc->placeImage($jpeg, new Rect(72.0, 100.0, 468.0, 600.0), "fit");
$doc->endPage();

$doc->endDocument();

echo "Generated: $path\n";
