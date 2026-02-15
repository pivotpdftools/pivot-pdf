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
