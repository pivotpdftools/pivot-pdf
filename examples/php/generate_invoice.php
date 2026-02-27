<?php
/**
 * Invoice example — realistic single-page invoice layout.
 *
 * Mirrors: examples/rust/generate_invoice.rs
 *
 * Demonstrates the primary library use case: professional documents
 * combining graphics (logo), styled text, and tables (line items).
 *
 * Run with:
 *   php -d extension=target/release/libpdf_php.so examples/php/generate_invoice.php
 */

@mkdir(__DIR__ . '/../output', 0755, true);
$path = __DIR__ . '/../output/php-invoice.pdf';

// ── helpers ───────────────────────────────────────────────────────────────────

/** Format a monetary value with thousands separator: 9600.00 → "$9,600.00" */
function fmtMoney(float $amount): string {
    return '$' . number_format($amount, 2, '.', ',');
}

// ── constants ────────────────────────────────────────────────────────────────
define('PAGE_W',  612.0);
define('PAGE_H',  792.0);
define('MARGIN',   72.0);
define('RIGHT',   540.0); // PAGE_W - MARGIN

// ── colors ───────────────────────────────────────────────────────────────────
$navy     = new Color(0.118, 0.227, 0.373);
$teal     = new Color(0.0,   0.706, 0.847);
$midGray  = new Color(0.5,   0.5,   0.5);
$stripeBg = new Color(0.95,  0.97,  1.0);
$white    = new Color(1.0,   1.0,   1.0);
$black    = new Color(0.1,   0.1,   0.1);
$ltGray   = new Color(0.75,  0.75,  0.75);

// ── invoice data ──────────────────────────────────────────────────────────────
$lineItems = [
    ['description' => 'Web Development Services',    'qty' => 40, 'unit_price' => 150.00],
    ['description' => 'UI/UX Design',                'qty' => 20, 'unit_price' => 125.00],
    ['description' => 'Server Setup & Configuration', 'qty' =>  1, 'unit_price' => 500.00],
    ['description' => 'Monthly Maintenance',          'qty' =>  3, 'unit_price' => 200.00],
    ['description' => 'Brand Identity & Style Guide', 'qty' =>  1, 'unit_price' => 2500.00],
    ['description' => 'SEO Optimization Package',     'qty' =>  1, 'unit_price' =>  800.00],
    ['description' => 'CMS Training Sessions',        'qty' =>  4, 'unit_price' =>  150.00],
    ['description' => 'Cloud Infrastructure Setup',   'qty' =>  1, 'unit_price' => 1200.00],
    ['description' => 'Security Audit',               'qty' =>  1, 'unit_price' => 1500.00],
    ['description' => 'Mobile App Development',       'qty' => 80, 'unit_price' =>  150.00],
    ['description' => 'Annual Support Contract',      'qty' =>  1, 'unit_price' => 3600.00],
];

// ── document setup ────────────────────────────────────────────────────────────
$doc = PdfDocument::create($path);
$doc->setCompression(true);
$doc->setInfo('Title', 'Invoice INV-2024-0042');
$doc->setInfo('Creator', 'NovaPeak Solutions — generate_invoice example');

$doc->beginPage(PAGE_W, PAGE_H);

// ── logo ──────────────────────────────────────────────────────────────────────
// Navy filled block with teal accent stripe at the bottom
$doc->saveState();
$doc->setFillColor($navy);
$doc->rect(MARGIN, 740.0, 46.0, 40.0);
$doc->fill();
$doc->setFillColor($teal);
$doc->rect(MARGIN, 740.0, 46.0, 6.0);
$doc->fill();
// White "NP" initials
$doc->setFillColor($white);
$doc->placeTextStyled('NP', MARGIN + 5.0, 751.0, new TextStyle('Helvetica-Bold', 18.0));
$doc->restoreState();

// Company name (black — restored by restoreState above)
$doc->placeTextStyled('NovaPeak Solutions', MARGIN + 54.0, 765.0, new TextStyle('Helvetica-Bold', 11.0));

// Gray address / contact lines
$doc->saveState();
$doc->setFillColor($midGray);
$doc->placeTextStyled('456 Innovation Drive, Suite 200',       MARGIN + 54.0, 753.0, new TextStyle('Helvetica', 9.0));
$doc->placeTextStyled('San Francisco, CA 94102',               MARGIN + 54.0, 742.0, new TextStyle('Helvetica', 9.0));
$doc->placeTextStyled('info@novapeak.io  |  (415) 555-9200',   MARGIN + 54.0, 731.0, new TextStyle('Helvetica', 9.0));
$doc->restoreState();

// ── invoice title + metadata ──────────────────────────────────────────────────
$doc->placeTextStyled('INVOICE', 392.0, 766.0, new TextStyle('Helvetica-Bold', 22.0));

$metaRows = [
    ['Invoice #:', 'INV-2024-0042',     748.0],
    ['Date:',      'January 15, 2024',  736.0],
    ['Due Date:',  'February 15, 2024', 724.0],
];
foreach ($metaRows as [$label, $value, $y]) {
    $doc->saveState();
    $doc->setFillColor($midGray);
    $doc->placeTextStyled($label, 392.0, $y, new TextStyle('Helvetica-Bold', 9.0));
    $doc->restoreState();
    $doc->placeTextStyled($value, 453.0, $y, new TextStyle('Helvetica', 9.0));
}

// ── horizontal rule (teal) ────────────────────────────────────────────────────
function drawRule(PdfDocument $doc, float $y, Color $color): void {
    $doc->saveState();
    $doc->setStrokeColor($color);
    $doc->setLineWidth(0.75);
    $doc->moveTo(MARGIN, $y);
    $doc->lineTo(RIGHT, $y);
    $doc->stroke();
    $doc->restoreState();
}

drawRule($doc, 718.0, $teal);

// ── bill-to block ─────────────────────────────────────────────────────────────
$doc->saveState();
$doc->setFillColor($teal);
$doc->placeTextStyled('BILL TO', MARGIN, 706.0, new TextStyle('Helvetica-Bold', 8.0));
$doc->restoreState();

$doc->placeTextStyled('Acme Corporation', MARGIN, 694.0, new TextStyle('Helvetica-Bold', 11.0));
$doc->saveState();
$doc->setFillColor($midGray);
$doc->placeTextStyled('123 Business Ave',   MARGIN, 682.0, new TextStyle('Helvetica', 9.0));
$doc->placeTextStyled('New York, NY 10001', MARGIN, 671.0, new TextStyle('Helvetica', 9.0));
$doc->placeTextStyled('accounts@acme.com',  MARGIN, 660.0, new TextStyle('Helvetica', 9.0));
$doc->restoreState();

// ── line-items table ──────────────────────────────────────────────────────────
// Columns: Description | Qty | Unit Price | Total (sum = 468pt)
$table = new Table([250.0, 50.0, 90.0, 78.0]);
$table->setBorderColor($ltGray);
$table->setBorderWidth(0.5);

$tableRect = new Rect(MARGIN, 638.0, 468.0, 420.0);
$cursor    = new TableCursor($tableRect);

// Header row
$hs = new CellStyle();
$hs->font_name = 'Helvetica-Bold';
$hs->font_size = 9.0;
$hs->padding   = 5.0;
$hs->setBackgroundColor($navy);
$hs->setTextColor($white);

$doc->fitRow($table, new Row([
    Cell::styled('DESCRIPTION', $hs),
    Cell::styled('QTY',         $hs),
    Cell::styled('UNIT PRICE',  $hs),
    Cell::styled('TOTAL',       $hs),
]), $cursor);

// Data rows with alternating background
foreach ($lineItems as $i => $item) {
    $ds = new CellStyle();
    $ds->font_name = 'Helvetica';
    $ds->font_size = 9.0;
    $ds->padding   = 5.0;
    if ($i % 2 === 0) {
        $ds->setBackgroundColor($stripeBg);
    }

    $row = new Row([
        Cell::styled($item['description'],                         $ds),
        Cell::styled((string)$item['qty'],                         $ds),
        Cell::styled(fmtMoney($item['unit_price']),                $ds),
        Cell::styled(fmtMoney($item['qty'] * $item['unit_price']), $ds),
    ]);

    $result = $doc->fitRow($table, $row, $cursor);
    if ($result !== 'stop') {
        fwrite(STDERR, "Warning: table unexpectedly full at row " . ($i + 1) . "\n");
        break;
    }
}

// ── totals section ────────────────────────────────────────────────────────────
// Use the cursor's actual Y to position totals — no guessing.
$tableBottom = $cursor->currentY();

$subtotal = array_sum(array_map(fn($i) => $i['qty'] * $i['unit_price'], $lineItems));
$taxRate  = 0.08;
$tax      = $subtotal * $taxRate;
$total    = $subtotal + $tax;

// Reset fill color to black — the table may have left a background color
// as the active fill, which would render plain text invisible.
$doc->setFillColor($black);

$lx = 362.0; // label column x
$vx = 462.0; // value column x

// Layout positions derived from the actual table bottom
$sepY      = $tableBottom - 10.0; // light separator, 10pt gap below table
$subtotalY = $sepY        - 14.0; // subtotal row baseline
$taxY      = $subtotalY   - 14.0; // tax row baseline
$ruleY     = $taxY        - 10.0; // bold rule above total
$totalY    = $ruleY       - 14.0; // total row baseline

// Light separator
$doc->saveState();
$doc->setStrokeColor($ltGray);
$doc->setLineWidth(0.5);
$doc->moveTo($lx, $sepY);
$doc->lineTo(RIGHT, $sepY);
$doc->stroke();
$doc->restoreState();

// Subtotal
$doc->saveState();
$doc->setFillColor($midGray);
$doc->placeTextStyled('Subtotal:', $lx, $subtotalY, new TextStyle('Helvetica', 9.0));
$doc->restoreState();
$doc->placeTextStyled(fmtMoney($subtotal), $vx, $subtotalY, new TextStyle('Helvetica', 9.0));

// Tax
$doc->saveState();
$doc->setFillColor($midGray);
$doc->placeTextStyled(sprintf('Tax (%d%%):', (int)($taxRate * 100)), $lx, $taxY, new TextStyle('Helvetica', 9.0));
$doc->restoreState();
$doc->placeTextStyled(fmtMoney($tax), $vx, $taxY, new TextStyle('Helvetica', 9.0));

// Bold rule above the total line
$doc->saveState();
$doc->setStrokeColor($navy);
$doc->setLineWidth(1.0);
$doc->moveTo($lx, $ruleY);
$doc->lineTo(RIGHT, $ruleY);
$doc->stroke();
$doc->restoreState();

// Total (navy bold)
$doc->placeTextStyled('TOTAL:', $lx, $totalY, new TextStyle('Helvetica-Bold', 10.0));
$doc->saveState();
$doc->setFillColor($navy);
$doc->placeTextStyled(fmtMoney($total), $vx, $totalY, new TextStyle('Helvetica-Bold', 10.0));
$doc->restoreState();

// ── footer ────────────────────────────────────────────────────────────────────
drawRule($doc, 108.0, $teal);

$doc->saveState();
$doc->setFillColor($midGray);
$doc->placeTextStyled(
    'Payment Terms: Net 30  |  Please make checks payable to NovaPeak Solutions',
    MARGIN, 94.0, new TextStyle('Helvetica', 8.0)
);
$doc->restoreState();

$doc->saveState();
$doc->setFillColor($teal);
$doc->placeTextStyled('Thank you for your business!', MARGIN, 80.0, new TextStyle('Helvetica-Oblique', 9.0));
$doc->restoreState();

// ── finalise ──────────────────────────────────────────────────────────────────
$doc->endPage();
$doc->endDocument();

echo "Written to $path\n";
