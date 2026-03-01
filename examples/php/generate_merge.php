<?php
/**
 * Merge two previously generated PDFs and report the page count.
 *
 * Merges the files produced by generate_tables.php and generate_invoice.php
 * into a single PDF, then reads it back and prints the total page count.
 *
 * Run generate_tables.php and generate_invoice.php first, then:
 *   php -d extension=target/release/libpdf_php.so examples/php/generate_merge.php
 *
 * Mirrors: examples/rust/generate_merge.rs
 *
 * Output: examples/output/php-merged.pdf
 */

$inputs = [
    __DIR__ . '/../output/php-tables.pdf',
    __DIR__ . '/../output/php-invoice.pdf',
];
$output = __DIR__ . '/../output/php-merged.pdf';

foreach ($inputs as $path) {
    if (!file_exists($path)) {
        fwrite(STDERR, "Missing input: $path\n");
        fwrite(STDERR, "Hint: run generate_tables.php and generate_invoice.php first.\n");
        exit(1);
    }
}

echo "Merging:\n";
foreach ($inputs as $path) {
    $reader = PdfReader::open($path);
    echo "  $path  (" . $reader->pageCount() . " pages)\n";
}

$opts = new MergeOptions();
merge_pdfs($inputs, $output, $opts);

$reader = PdfReader::open($output);
echo "\nOutput: $output\n";
echo "Pages:  " . $reader->pageCount() . "\n";
