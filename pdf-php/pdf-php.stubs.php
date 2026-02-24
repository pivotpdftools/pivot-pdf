<?php

/**
 * Stubs for the pdf-php extension.
 *
 * This file is not executed — it provides type hints and
 * autocompletion for IDEs (PhpStorm, Intelephense, etc.).
 */

class Color
{
    public float $r;
    public float $g;
    public float $b;

    /**
     * Create an RGB color.
     *
     * @param float $r Red component (0.0–1.0)
     * @param float $g Green component (0.0–1.0)
     * @param float $b Blue component (0.0–1.0)
     */
    public function __construct(float $r, float $g, float $b) {}

    /**
     * Create a grayscale color (r = g = b = level).
     *
     * @param float $level Gray level (0.0–1.0)
     */
    public static function gray(float $level): self {}
}

class TextStyle
{
    public string $font_name;
    public float $font_size;
    public int $font_handle;

    /**
     * Create a TextStyle with a builtin font name.
     *
     * @param string $font      Font name (default: "Helvetica").
     *                           Valid names: Helvetica, Helvetica-Bold,
     *                           Helvetica-Oblique, Helvetica-BoldOblique,
     *                           Times-Roman, Times-Bold, Times-Italic,
     *                           Times-BoldItalic, Courier, Courier-Bold,
     *                           Courier-Oblique, Courier-BoldOblique,
     *                           Symbol, ZapfDingbats
     * @param float  $font_size Font size in points (default: 12.0)
     */
    public function __construct(
        string $font = 'Helvetica',
        float $font_size = 12.0
    ) {}

    /**
     * Create a TextStyle for a TrueType font loaded via
     * PdfDocument::loadFontFile().
     *
     * @param int   $handle    Font handle returned by loadFontFile()
     * @param float $font_size Font size in points (default: 12.0)
     */
    public static function truetype(
        int $handle,
        float $font_size = 12.0
    ): self {}
}

class Rect
{
    public float $x;
    public float $y;
    public float $width;
    public float $height;

    /**
     * @param float $x      X coordinate of the upper-left corner
     * @param float $y      Y coordinate of the upper-left corner
     * @param float $width  Width of the rectangle
     * @param float $height Height of the rectangle
     */
    public function __construct(
        float $x,
        float $y,
        float $width,
        float $height
    ) {}
}

class TextFlow
{
    public function __construct() {}

    /**
     * Add styled text to the flow.
     *
     * @param string    $text  The text to add
     * @param TextStyle $style The style to apply
     */
    public function addText(string $text, TextStyle $style): void {}

    /**
     * Check whether all text has been consumed.
     */
    public function isFinished(): bool {}
}

class CellStyle
{
    public string $font_name;
    public int $font_handle;
    public float $font_size;
    public float $padding;
    /** Overflow mode: "wrap", "clip", or "shrink" */
    public string $overflow;

    /**
     * Create a CellStyle with default values.
     *
     * Defaults: font = "Helvetica", font_size = 10.0, padding = 4.0, overflow = "wrap".
     */
    public function __construct() {}

    /**
     * Set the background color. Pass null to clear.
     *
     * @param Color|null $color Background color, or null for transparent
     */
    public function setBackgroundColor(?Color $color): void {}

    /**
     * Set the text color. Pass null to use the PDF default (black).
     *
     * @param Color|null $color Text color, or null for default
     */
    public function setTextColor(?Color $color): void {}
}

class Cell
{
    /**
     * Create a cell with default style.
     *
     * @param string $text Cell content
     */
    public function __construct(string $text) {}

    /**
     * Create a cell with an explicit style.
     *
     * @param string    $text  Cell content
     * @param CellStyle $style Cell style
     * @throws \Exception if the style contains an invalid font name
     */
    public static function styled(string $text, CellStyle $style): self {}
}

class Row
{
    /** Optional fixed height in points. Required for "clip" and "shrink" overflow. */
    public ?float $height;

    /**
     * Create a row with the given cells.
     *
     * @param Cell[] $cells Array of Cell objects
     */
    public function __construct(array $cells) {}

    /**
     * Set the row background color. Per-cell background takes priority.
     *
     * @param Color|null $color Row background color, or null for transparent
     */
    public function setBackgroundColor(?Color $color): void {}
}

class Table
{
    /**
     * Create a table with the given column widths.
     *
     * Table is config-only. Pass rows to PdfDocument::fitRow().
     *
     * @param float[] $columns Column widths in points
     */
    public function __construct(array $columns) {}

    /**
     * Set the border stroke color.
     *
     * @param Color $color Border color
     */
    public function setBorderColor(Color $color): void {}

    /**
     * Set the border line width. Set to 0.0 to disable borders.
     *
     * @param float $width Border width in points
     */
    public function setBorderWidth(float $width): void {}

    /**
     * Set the default style used as a fallback for cells without explicit styles.
     *
     * @param CellStyle $style Default cell style
     * @throws \Exception if the style contains an invalid font name
     */
    public function setDefaultStyle(CellStyle $style): void {}
}

class TableCursor
{
    /**
     * Create a cursor for placing rows within a bounding rectangle.
     *
     * Call reset() when starting a new page. Use isFirstRow() to detect
     * the top of a page and insert a repeated header row.
     *
     * @param Rect $rect The bounding rectangle for table rows on this page
     */
    public function __construct(Rect $rect) {}

    /**
     * Reset the cursor to the top of a new bounding rectangle.
     *
     * Call this after starting a new page.
     *
     * @param Rect $rect The bounding rectangle on the new page
     */
    public function reset(Rect $rect): void {}

    /**
     * Returns true if no rows have been placed yet on the current page.
     *
     * Use this to insert a repeated header row at the top of each page.
     */
    public function isFirstRow(): bool {}
}

class PdfDocument
{
    /**
     * Create a new PDF document that writes to a file.
     *
     * @param string $path File path to write the PDF to
     * @throws \Exception on I/O error
     */
    public static function create(string $path): self {}

    /**
     * Create a new PDF document in memory.
     *
     * @throws \Exception on error
     */
    public static function createInMemory(): self {}

    /**
     * Load a TrueType font file (.ttf).
     *
     * Returns an integer font handle for use with
     * TextStyle::truetype().
     *
     * @param string $path Path to the .ttf font file
     * @return int Font handle
     * @throws \Exception if the file cannot be read or parsed
     */
    public function loadFontFile(string $path): int {}

    /**
     * Set a document info entry (e.g. "Creator", "Title").
     *
     * @param string $key   Info key
     * @param string $value Info value
     * @throws \Exception if the document has already ended
     */
    public function setInfo(string $key, string $value): void {}

    /**
     * Enable or disable FlateDecode compression for stream objects.
     *
     * When enabled, page content, embedded fonts, and ToUnicode CMaps
     * are compressed, typically reducing file size by 50-80%.
     * Disabled by default.
     *
     * @param bool $enabled Whether to enable compression
     * @throws \Exception if the document has already ended
     */
    public function setCompression(bool $enabled): void {}

    /**
     * Begin a new page with the given dimensions in points.
     *
     * @param float $width  Page width in points
     * @param float $height Page height in points
     * @throws \Exception if the document has already ended
     */
    public function beginPage(float $width, float $height): void {}

    /**
     * Place text at (x, y) using default 12pt Helvetica.
     *
     * @param string $text Text to place
     * @param float  $x   X coordinate (bottom-left origin)
     * @param float  $y   Y coordinate (bottom-left origin)
     * @throws \Exception if the document has already ended
     */
    public function placeText(
        string $text,
        float $x,
        float $y
    ): void {}

    /**
     * Place text at (x, y) using an explicit TextStyle.
     *
     * @param string    $text  Text to place
     * @param float     $x     X coordinate (bottom-left origin)
     * @param float     $y     Y coordinate (bottom-left origin)
     * @param TextStyle $style Font and size to use
     * @throws \Exception if the document has already ended or style is invalid
     */
    public function placeTextStyled(
        string $text,
        float $x,
        float $y,
        TextStyle $style
    ): void {}

    /**
     * Fit a TextFlow into a bounding rectangle on the current page.
     *
     * @param TextFlow $flow The text flow to fit
     * @param Rect     $rect The bounding rectangle
     * @return string "stop", "box_full", or "box_empty"
     * @throws \Exception on error or if the document has already ended
     */
    public function fitTextflow(
        TextFlow $flow,
        Rect $rect
    ): string {}

    /**
     * Place a single row on the current page using the streaming fit-row pattern.
     *
     * Returns "stop" when the row was placed successfully (advance to next row).
     * Returns "box_full" when the page is full (end page, begin new page, reset
     * cursor, then retry the same row). Returns "box_empty" when the rect is
     * intrinsically too small for the row.
     *
     * @param Table       $table  Table config (column widths, border, default style)
     * @param Row         $row    The row to place
     * @param TableCursor $cursor Page-level cursor tracking current Y position
     * @return string "stop", "box_full", or "box_empty"
     * @throws \Exception on error or if the document has already ended
     */
    public function fitRow(Table $table, Row $row, TableCursor $cursor): string {}

    // -------------------------------------------------------
    // Graphics operations
    // -------------------------------------------------------

    /**
     * Set the stroke color.
     *
     * @param Color $color The stroke color
     * @throws \Exception if the document has already ended
     */
    public function setStrokeColor(Color $color): void {}

    /**
     * Set the fill color.
     *
     * @param Color $color The fill color
     * @throws \Exception if the document has already ended
     */
    public function setFillColor(Color $color): void {}

    /**
     * Set the line width.
     *
     * @param float $width Line width in points
     * @throws \Exception if the document has already ended
     */
    public function setLineWidth(float $width): void {}

    /**
     * Move to a point without drawing.
     *
     * @param float $x X coordinate
     * @param float $y Y coordinate
     * @throws \Exception if the document has already ended
     */
    public function moveTo(float $x, float $y): void {}

    /**
     * Draw a line from the current point.
     *
     * @param float $x X coordinate of the end point
     * @param float $y Y coordinate of the end point
     * @throws \Exception if the document has already ended
     */
    public function lineTo(float $x, float $y): void {}

    /**
     * Append a rectangle to the path.
     *
     * @param float $x      X coordinate of the lower-left corner
     * @param float $y      Y coordinate of the lower-left corner
     * @param float $width  Width of the rectangle
     * @param float $height Height of the rectangle
     * @throws \Exception if the document has already ended
     */
    public function rect(
        float $x,
        float $y,
        float $width,
        float $height
    ): void {}

    /**
     * Close the current subpath.
     *
     * @throws \Exception if the document has already ended
     */
    public function closePath(): void {}

    /**
     * Stroke the current path.
     *
     * @throws \Exception if the document has already ended
     */
    public function stroke(): void {}

    /**
     * Fill the current path.
     *
     * @throws \Exception if the document has already ended
     */
    public function fill(): void {}

    /**
     * Fill and stroke the current path.
     *
     * @throws \Exception if the document has already ended
     */
    public function fillStroke(): void {}

    /**
     * Save the graphics state.
     *
     * @throws \Exception if the document has already ended
     */
    public function saveState(): void {}

    /**
     * Restore the graphics state.
     *
     * @throws \Exception if the document has already ended
     */
    public function restoreState(): void {}

    // -------------------------------------------------------
    // Image operations
    // -------------------------------------------------------

    /**
     * Load an image from a file path (JPEG or PNG).
     *
     * Returns an integer image handle for use with placeImage().
     *
     * @param string $path Path to the image file
     * @return int Image handle
     * @throws \Exception if the file cannot be read or parsed
     */
    public function loadImageFile(string $path): int {}

    /**
     * Load an image from raw bytes (JPEG or PNG).
     *
     * Returns an integer image handle for use with placeImage().
     *
     * @param string $data Raw image bytes
     * @return int Image handle
     * @throws \Exception if the data cannot be parsed
     */
    public function loadImageBytes(string $data): int {}

    /**
     * Place an image on the current page within a bounding rectangle.
     *
     * @param int    $handle Image handle from loadImageFile/loadImageBytes
     * @param Rect   $rect   Bounding rectangle for the image
     * @param string $fit    Fit mode: "fit" (default), "fill", "stretch", "none"
     * @throws \Exception if the document has already ended
     */
    public function placeImage(
        int $handle,
        Rect $rect,
        string $fit = 'fit'
    ): void {}

    /**
     * Returns the number of completed pages.
     *
     * A page is "completed" once `endPage()` has been called for it.
     * An open page (after `beginPage()` but before `endPage()`) is not
     * yet counted.
     *
     * @return int Number of completed pages
     * @throws \Exception if the document has already ended
     */
    public function pageCount(): int {}

    /**
     * Open a completed page for editing (1-indexed).
     *
     * Used for adding overlay content such as page numbers ("Page X of Y")
     * after all pages have been written. Call `endPage()` when done.
     *
     * If a page is currently open, it is automatically closed first.
     *
     * @param int $pageNum 1-indexed page number to open for editing
     * @throws \Exception if pageNum is out of range or document already ended
     */
    public function openPage(int $pageNum): void {}

    /**
     * End the current page.
     *
     * @throws \Exception if the document has already ended
     */
    public function endPage(): void {}

    /**
     * End the document.
     *
     * For file-based documents, returns null.
     * For in-memory documents, returns the PDF as a binary string.
     *
     * @return string|null Binary PDF data (in-memory) or null (file)
     * @throws \Exception if the document has already ended
     */
    public function endDocument(): ?string {}
}
