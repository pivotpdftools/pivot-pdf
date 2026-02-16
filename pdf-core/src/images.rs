use crate::textflow::Rect;

/// Opaque handle to a loaded image within a PdfDocument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageId(pub usize);

/// Supported image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
}

/// How an image should be scaled to fit a bounding rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFit {
    /// Scale to fit within the rect, preserving aspect ratio.
    Fit,
    /// Scale to cover the rect, clipping overflow.
    Fill,
    /// Stretch to fill the rect exactly (may distort).
    Stretch,
    /// Natural size: 1 pixel = 1 point, no scaling.
    None,
}

/// PDF color space for image data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    DeviceRGB,
    DeviceGray,
}

impl ColorSpace {
    pub fn pdf_name(&self) -> &'static str {
        match self {
            ColorSpace::DeviceRGB => "DeviceRGB",
            ColorSpace::DeviceGray => "DeviceGray",
        }
    }
}

/// Parsed image data ready for embedding into a PDF.
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub color_space: ColorSpace,
    pub bits_per_component: u8,
    /// Raw pixel data (RGB/Gray) or raw JPEG bytes.
    pub data: Vec<u8>,
    /// Separate alpha channel (grayscale), if present.
    pub smask_data: Option<Vec<u8>>,
}

/// Computed placement of an image on a PDF page.
#[derive(Debug)]
pub struct ImagePlacement {
    /// X position in PDF coordinates (bottom-left origin).
    pub x: f64,
    /// Y position in PDF coordinates (bottom-left origin).
    pub y: f64,
    /// Display width in points.
    pub width: f64,
    /// Display height in points.
    pub height: f64,
    /// Optional clip rectangle (for Fill mode) in PDF coordinates.
    pub clip: Option<ClipRect>,
}

/// A clip rectangle in PDF coordinates (bottom-left origin).
#[derive(Debug)]
pub struct ClipRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Detect image format from magic bytes.
pub fn detect_format(data: &[u8]) -> Result<ImageFormat, String> {
    if data.len() < 4 {
        return Err("Image data too short to detect format".to_string());
    }
    if data[0] == 0xFF && data[1] == 0xD8 {
        Ok(ImageFormat::Jpeg)
    } else if data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 {
        Ok(ImageFormat::Png)
    } else {
        Err("Unsupported image format (expected JPEG or PNG)".to_string())
    }
}

/// Load and parse image data from raw bytes.
pub fn load_image(data: Vec<u8>) -> Result<ImageData, String> {
    let format = detect_format(&data)?;
    match format {
        ImageFormat::Jpeg => parse_jpeg(data),
        ImageFormat::Png => parse_png(data),
    }
}

/// Parse JPEG SOF marker to extract dimensions and color space.
/// JPEG data is embedded as-is (DCTDecode); no pixel decoding needed.
fn parse_jpeg(data: Vec<u8>) -> Result<ImageData, String> {
    let (width, height, components) = jpeg_dimensions(&data)?;
    let color_space = match components {
        1 => ColorSpace::DeviceGray,
        3 => ColorSpace::DeviceRGB,
        _ => {
            return Err(format!(
                "Unsupported JPEG component count: {} (expected 1 or 3)",
                components
            ))
        }
    };

    Ok(ImageData {
        width,
        height,
        format: ImageFormat::Jpeg,
        color_space,
        bits_per_component: 8,
        data,
        smask_data: None,
    })
}

/// Scan JPEG data for SOF0-SOF3 markers and extract width/height/components.
fn jpeg_dimensions(data: &[u8]) -> Result<(u32, u32, u8), String> {
    let len = data.len();
    let mut i = 0;
    while i + 1 < len {
        if data[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = data[i + 1];
        // SOF0 (0xC0) through SOF3 (0xC3) — baseline, extended, progressive, lossless
        if (0xC0..=0xC3).contains(&marker) {
            if i + 9 >= len {
                return Err("JPEG SOF marker truncated".to_string());
            }
            let height = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
            let width = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
            let components = data[i + 9];
            return Ok((width, height, components));
        }
        // Skip non-SOF markers
        if marker == 0xFF || marker == 0x00 {
            i += 1;
            continue;
        }
        // Standalone markers (no length)
        if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) {
            i += 2;
            continue;
        }
        // Markers with length
        if i + 3 >= len {
            break;
        }
        let seg_len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
        i += 2 + seg_len;
    }
    Err("No SOF marker found in JPEG data".to_string())
}

/// Decode PNG using the `png` crate and produce raw pixel data.
fn parse_png(data: Vec<u8>) -> Result<ImageData, String> {
    let decoder = png::Decoder::new(data.as_slice());
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("PNG decode error: {}", e))?;

    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| format!("PNG frame error: {}", e))?;
    buf.truncate(info.buffer_size());

    let width = info.width;
    let height = info.height;

    match info.color_type {
        png::ColorType::Rgb => Ok(ImageData {
            width,
            height,
            format: ImageFormat::Png,
            color_space: ColorSpace::DeviceRGB,
            bits_per_component: 8,
            data: buf,
            smask_data: None,
        }),
        png::ColorType::Rgba => {
            let pixel_count = (width * height) as usize;
            let mut rgb = Vec::with_capacity(pixel_count * 3);
            let mut alpha = Vec::with_capacity(pixel_count);
            for chunk in buf.chunks_exact(4) {
                rgb.push(chunk[0]);
                rgb.push(chunk[1]);
                rgb.push(chunk[2]);
                alpha.push(chunk[3]);
            }
            Ok(ImageData {
                width,
                height,
                format: ImageFormat::Png,
                color_space: ColorSpace::DeviceRGB,
                bits_per_component: 8,
                data: rgb,
                smask_data: Some(alpha),
            })
        }
        png::ColorType::Grayscale => Ok(ImageData {
            width,
            height,
            format: ImageFormat::Png,
            color_space: ColorSpace::DeviceGray,
            bits_per_component: 8,
            data: buf,
            smask_data: None,
        }),
        png::ColorType::GrayscaleAlpha => {
            let pixel_count = (width * height) as usize;
            let mut gray = Vec::with_capacity(pixel_count);
            let mut alpha = Vec::with_capacity(pixel_count);
            for chunk in buf.chunks_exact(2) {
                gray.push(chunk[0]);
                alpha.push(chunk[1]);
            }
            Ok(ImageData {
                width,
                height,
                format: ImageFormat::Png,
                color_space: ColorSpace::DeviceGray,
                bits_per_component: 8,
                data: gray,
                smask_data: Some(alpha),
            })
        }
        other => Err(format!("Unsupported PNG color type: {:?}", other)),
    }
}

/// Calculate image placement given a bounding rectangle and fit mode.
///
/// The `Rect` uses upper-left origin (y grows downward for layout),
/// but PDF uses bottom-left origin. The `page_height` parameter is
/// needed for the coordinate conversion.
pub fn calculate_placement(
    img_w: u32,
    img_h: u32,
    rect: &Rect,
    fit: ImageFit,
    page_height: f64,
) -> ImagePlacement {
    let iw = img_w as f64;
    let ih = img_h as f64;

    // Convert upper-left rect origin to PDF bottom-left origin.
    // rect.y is the top edge in upper-left coords.
    // In PDF coords, the bottom edge is: page_height - (rect.y + rect.height)
    let pdf_bottom = page_height - (rect.y + rect.height);

    match fit {
        ImageFit::Fit => {
            let scale_x = rect.width / iw;
            let scale_y = rect.height / ih;
            let scale = scale_x.min(scale_y);
            let w = iw * scale;
            let h = ih * scale;
            // Center within the rect
            let x = rect.x + (rect.width - w) / 2.0;
            let y = pdf_bottom + (rect.height - h) / 2.0;
            ImagePlacement {
                x,
                y,
                width: w,
                height: h,
                clip: None,
            }
        }
        ImageFit::Fill => {
            let scale_x = rect.width / iw;
            let scale_y = rect.height / ih;
            let scale = scale_x.max(scale_y);
            let w = iw * scale;
            let h = ih * scale;
            // Center the image (some parts will be clipped)
            let x = rect.x + (rect.width - w) / 2.0;
            let y = pdf_bottom + (rect.height - h) / 2.0;
            ImagePlacement {
                x,
                y,
                width: w,
                height: h,
                clip: Some(ClipRect {
                    x: rect.x,
                    y: pdf_bottom,
                    width: rect.width,
                    height: rect.height,
                }),
            }
        }
        ImageFit::Stretch => ImagePlacement {
            x: rect.x,
            y: pdf_bottom,
            width: rect.width,
            height: rect.height,
            clip: None,
        },
        ImageFit::None => {
            // 1 pixel = 1 point, positioned at top-left of rect
            let y = pdf_bottom + (rect.height - ih);
            ImagePlacement {
                x: rect.x,
                y,
                width: iw,
                height: ih,
                clip: None,
            }
        }
    }
}
