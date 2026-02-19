use font8x8::UnicodeFonts;
use image::{GrayImage, Luma, RgbaImage, Rgba};

#[derive(Debug, Clone)]
pub struct AsciiOptions {
    pub columns: u32,
    pub charset: Vec<char>,
    pub shades: u32,  // Number of grayscale shades (1 = B/W, 2-256 = grayscale)
}

impl AsciiOptions {
    pub fn new(columns: u32, charset: &str, shades: u32) -> Self {
        let mut chars: Vec<char> = charset.chars().collect();
        if chars.is_empty() {
            chars = "@#*+=-:. ".chars().collect();
        }

        Self {
            columns: columns.max(1),
            charset: chars,
            shades: shades.clamp(1, 256),
        }
    }
}

pub fn convert_frame_to_ascii(source: &GrayImage, options: &AsciiOptions) -> GrayImage {
    // Calculate grid size based on character size (8x8 pixels per char)
    let char_width = 8u32;
    let char_height = 8u32;
    
    // Calculate how many characters fit in the original dimensions
    let columns = source.width() / char_width;
    let rows = source.height() / char_height;
    
    // Output will be SAME size as input (each char = 8x8 block)
    let out_width = columns * char_width;
    let out_height = rows * char_height;

    let mut output = GrayImage::from_pixel(out_width, out_height, Luma([255]));

    for row in 0..rows {
        let y0 = row * char_height;
        let y1 = y0 + char_height;

        for col in 0..columns {
            let x0 = col * char_width;
            let x1 = x0 + char_width;

            let luma = average_luma(source, x0, x1, y0, y1);
            // Enhance contrast: stretch 0-255 to have more separation
            let enhanced = enhance_contrast(luma);
            let ch = map_luma_to_char(enhanced, &options.charset);
            
            // Draw with grayscale if shades > 1, otherwise pure B/W
            if options.shades > 1 {
                draw_glyph_gray(&mut output, x0, y0, ch, enhanced, options.shades);
            } else {
                draw_glyph_bw(&mut output, x0, y0, ch);
            }
        }
    }

    output
}

/// Detect the most common background color in the image
pub fn detect_background_color(image: &GrayImage) -> u8 {
    let mut histogram = [0usize; 256];

    for pixel in image.pixels() {
        histogram[pixel[0] as usize] += 1;
    }

    // Find the most frequent color (likely background)
    let mut max_count = 0;
    let mut bg_color = 255u8;

    for (color, &count) in histogram.iter().enumerate() {
        if count > max_count {
            max_count = count;
            bg_color = color as u8;
        }
    }

    bg_color
}

/// Convert grayscale ASCII to RGBA with transparency.
/// Pixels whose grayscale value is within `threshold` of `bg_color`
/// (i.e. `|pixel - bg_color| <= threshold`) become fully transparent.
/// Pass `threshold = 0` for exact-match behaviour.
pub fn convert_to_transparent(source: &GrayImage, bg_color: u8, threshold: u8) -> RgbaImage {
    let mut rgba = RgbaImage::new(source.width(), source.height());

    for (x, y, pixel) in source.enumerate_pixels() {
        let luma = pixel[0];
        let is_background = (luma as i16 - bg_color as i16).unsigned_abs() as u8 <= threshold;

        // If background, make transparent; otherwise, keep grayscale
        let rgba_pixel = if is_background {
            Rgba([255, 255, 255, 0]) // Fully transparent
        } else {
            Rgba([luma, luma, luma, 255]) // Opaque grayscale
        };

        rgba.put_pixel(x, y, rgba_pixel);
    }

    rgba
}

fn enhance_contrast(luma: u8) -> u8 {
    // Apply mild contrast stretch to make edges more visible
    let f = luma as f32 / 255.0;
    let enhanced = ((f - 0.5) * 1.5 + 0.5).clamp(0.0, 1.0);
    (enhanced * 255.0) as u8
}

fn average_luma(image: &GrayImage, x0: u32, x1: u32, y0: u32, y1: u32) -> u8 {
    let mut sum: u64 = 0;
    let mut count: u64 = 0;

    for y in y0..y1.min(image.height()) {
        for x in x0..x1.min(image.width()) {
            sum += image.get_pixel(x, y)[0] as u64;
            count += 1;
        }
    }

    if count == 0 { 0 } else { (sum / count) as u8 }
}

fn map_luma_to_char(luma: u8, charset: &[char]) -> char {
    let last = charset.len().saturating_sub(1);
    let idx = (luma as usize * last) / 255;
    charset[idx]
}

fn draw_glyph_gray(canvas: &mut GrayImage, x: u32, y: u32, ch: char, brightness: u8, num_shades: u32) {
    let fallback = font8x8::BASIC_FONTS.get('?').unwrap_or([0; 8]);
    let glyph = font8x8::BASIC_FONTS.get(ch).unwrap_or(fallback);
    
    // Map brightness (0-255) to grayscale value based on num_shades
    // More shades = smoother gradients, fewer shades = more contrasty
    let shade_step = 255.0 / (num_shades as f32 - 1.0);
    let shade_index = (brightness as f32 / shade_step).round().clamp(0.0, num_shades as f32 - 1.0) as u8;
    let pixel_value = if num_shades == 2 {
        // For 2 shades, use pure B/W for maximum contrast
        if brightness < 128 { 0 } else { 255 }
    } else {
        // For 3+ shades, use actual grayscale
        (shade_index as f32 * 255.0 / (num_shades as f32 - 1.0)).round() as u8
    };

    for (gy, row_bits) in glyph.iter().enumerate() {
        for gx in 0..8_u32 {
            let bit_on = (row_bits >> gx) & 1 == 1;
            // If bit is on, use the brightness value; if off, use white (255)
            let value = if bit_on { pixel_value } else { 255 };
            canvas.put_pixel(x + gx, y + gy as u32, Luma([value]));
        }
    }
}

fn draw_glyph_bw(canvas: &mut GrayImage, x: u32, y: u32, ch: char) {
    let fallback = font8x8::BASIC_FONTS.get('?').unwrap_or([0; 8]);
    let glyph = font8x8::BASIC_FONTS.get(ch).unwrap_or(fallback);

    for (gy, row_bits) in glyph.iter().enumerate() {
        for gx in 0..8_u32 {
            let bit_on = (row_bits >> gx) & 1 == 1;
            let value = if bit_on { 0 } else { 255 };
            canvas.put_pixel(x + gx, y + gy as u32, Luma([value]));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_luma_extremes_to_expected_charset_bounds() {
        let charset: Vec<char> = "# ".chars().collect();
        assert_eq!(map_luma_to_char(0, &charset), '#');
        assert_eq!(map_luma_to_char(255, &charset), ' ');
    }

    #[test]
    fn conversion_creates_expected_dimensions() {
        let source = GrayImage::from_pixel(64, 32, Luma([120]));
        let options = AsciiOptions::new(16, "# ", 1);
        let output = convert_frame_to_ascii(&source, &options);

        // Source 64x32 → 8 columns x 4 rows (64/8, 32/8)
        // Output: 8*8 x 4*8 = 64 x 32
        assert_eq!(output.width(), 8 * 8);
        assert_eq!(output.height(), 4 * 8);
    }

    #[test]
    fn conversion_is_strictly_black_and_white() {
        let mut source = GrayImage::from_pixel(16, 16, Luma([0]));
        for y in 8..16 {
            for x in 8..16 {
                source.put_pixel(x, y, Luma([255]));
            }
        }

        let options = AsciiOptions::new(4, "@ ", 1);
        let output = convert_frame_to_ascii(&source, &options);

        for pixel in output.pixels() {
            assert!(pixel[0] == 0 || pixel[0] == 255);
        }
    }

    #[test]
    fn transparent_exact_match_makes_bg_transparent() {
        // 4x1 image: pixels 0, 100, 200, 255
        let mut img = GrayImage::new(4, 1);
        img.put_pixel(0, 0, Luma([0]));
        img.put_pixel(1, 0, Luma([100]));
        img.put_pixel(2, 0, Luma([200]));
        img.put_pixel(3, 0, Luma([255]));

        let rgba = convert_to_transparent(&img, 255, 0);

        // Only pixel at value 255 should be transparent
        assert_eq!(rgba.get_pixel(0, 0)[3], 255, "pixel 0 should be opaque");
        assert_eq!(rgba.get_pixel(1, 0)[3], 255, "pixel 100 should be opaque");
        assert_eq!(rgba.get_pixel(2, 0)[3], 255, "pixel 200 should be opaque");
        assert_eq!(rgba.get_pixel(3, 0)[3], 0, "pixel 255 (bg) should be transparent");
    }

    #[test]
    fn transparent_threshold_removes_nearby_colors() {
        // bg_color = 240, threshold = 20 → values 220..=255 become transparent
        let mut img = GrayImage::new(4, 1);
        img.put_pixel(0, 0, Luma([219])); // outside threshold (219 < 220)
        img.put_pixel(1, 0, Luma([220])); // exactly at boundary (240 - 20 = 220)
        img.put_pixel(2, 0, Luma([255])); // within threshold (255 - 240 = 15 <= 20)
        img.put_pixel(3, 0, Luma([100])); // well outside threshold

        let rgba = convert_to_transparent(&img, 240, 20);

        assert_eq!(rgba.get_pixel(0, 0)[3], 255, "219 should be opaque");
        assert_eq!(rgba.get_pixel(1, 0)[3], 0, "220 should be transparent (|220-240|=20)");
        assert_eq!(rgba.get_pixel(2, 0)[3], 0, "255 should be transparent (|255-240|=15)");
        assert_eq!(rgba.get_pixel(3, 0)[3], 255, "100 should be opaque");
    }
}
