use std::path::PathBuf;

use tempfile::TempDir;

use crate::ascii::{AsciiOptions, convert_frame_to_ascii, detect_background_color, convert_to_transparent};
use crate::error::{AppError, Result};
use crate::video;

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub input: PathBuf,
    pub output: PathBuf,
    pub columns: u32,
    pub fps: Option<f64>,
    pub charset: String,
    pub shades: u32,
    pub transparent: bool,
    pub bg_color: Option<u8>,
    /// Tolerance for background matching (0 = exact, 255 = everything).
    /// Pixels within ±threshold of the detected/specified bg_color are made transparent.
    pub threshold: u8,
    /// Create a comparison video with original and ASCII versions stacked vertically
    pub compare: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct PipelineStats {
    pub frames_processed: usize,
    pub output_fps: f64,
}

pub fn run(config: &PipelineConfig) -> Result<PipelineStats> {
    if !config.input.exists() {
        return Err(AppError::InputNotFound(config.input.clone()));
    }

    if !video::tools_available() {
        return Err(AppError::MissingFfmpeg);
    }

    let metadata = video::probe_video(&config.input)?;
    let fps = config.fps.unwrap_or(metadata.fps);

    let temp_dir = TempDir::new()?;
    let extracted_dir = temp_dir.path().join("extracted");
    let ascii_dir = temp_dir.path().join("ascii");

    let frames = video::extract_frames(&config.input, &extracted_dir)?;
    std::fs::create_dir_all(&ascii_dir)?;

    let options = AsciiOptions::new(config.columns, &config.charset, config.shades);

    // Detect background color from first frame if not specified
    let bg_color = if config.transparent {
        match config.bg_color {
            Some(color) => color,
            None => {
                let first_frame = image::open(&frames[0])?.to_luma8();
                detect_background_color(&first_frame)
            }
        }
    } else {
        255 // Not used in non-transparent mode
    };

    for (index, frame_path) in frames.iter().enumerate() {
        let image = image::open(frame_path)?.to_luma8();
        let ascii = convert_frame_to_ascii(&image, &options);

        let output_frame = ascii_dir.join(format!("frame_{:08}.png", index));

        if config.transparent {
            // Convert to transparent RGBA
            let rgba = convert_to_transparent(&ascii, bg_color, config.threshold);
            rgba.save(output_frame)?;
        } else {
            ascii.save(output_frame)?;
        }
    }

    video::encode_video(&ascii_dir, &config.input, fps, &config.output, config.transparent)?;

    // Create comparison video if requested
    if config.compare {
        video::create_comparison_video(&config.input, &config.output)?;
    }

    Ok(PipelineStats {
        frames_processed: frames.len(),
        output_fps: fps,
    })
}
