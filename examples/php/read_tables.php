<?php
/**
 * Read an existing PDF and report the number of pages.
 *
 * Reads the file produced by generate_tables.php and prints the page count.
 * Run generate_tables.php first to create the input file.
 *
 * Mirrors: examples/rust/read_tables.rs
 *
 * Run with:
 *   php -d extension=target/release/libpdf_php.so examples/php/read_tables.php
 */

$path = __DIR__ . '/../output/php-tables.pdf';

if (!file_exists($path)) {
    fwrite(STDERR, "Error: file not found: $path\n");
    fwrite(STDERR, "Hint: run generate_tables.php first.\n");
    exit(1);
}

$reader = PdfReader::open($path);

echo "File:    $path\n";
echo "Version: PDF " . $reader->pdfVersion() . "\n";
echo "Pages:   " . $reader->pageCount() . "\n";
