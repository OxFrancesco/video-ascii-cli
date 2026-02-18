use font8x8::UnicodeFonts;
use image::{GrayImage, Luma};

#[derive(Debug, Clone)]
pub struct AsciiOptions {
    pub columns: u32,
    pub charset: Vec<char>,
}

impl AsciiOptions {
    pub fn new(columns: u32, charset: &str) -> Self {
        let mut chars: Vec<char> = charset.chars().collect();
        if chars.is_empty() {
            chars = "@#*+=-:. ".chars().collect();
        }

        Self {
            columns: columns.max(1),
            charset: chars,
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
            draw_glyph_bw(&mut output, x0, y0, ch);
        }
    }

    output
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
        let options = AsciiOptions::new(16, "# ");
        let output = convert_frame_to_ascii(&source, &options);

        assert_eq!(output.width(), 16 * 8);
        assert_eq!(output.height(), 8 * 8);
    }

    #[test]
    fn conversion_is_strictly_black_and_white() {
        let mut source = GrayImage::from_pixel(16, 16, Luma([0]));
        for y in 8..16 {
            for x in 8..16 {
                source.put_pixel(x, y, Luma([255]));
            }
        }

        let options = AsciiOptions::new(4, "@ ");
        let output = convert_frame_to_ascii(&source, &options);

        for pixel in output.pixels() {
            assert!(pixel[0] == 0 || pixel[0] == 255);
        }
    }
}
