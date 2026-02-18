use std::path::PathBuf;

use tempfile::TempDir;

use crate::ascii::{AsciiOptions, convert_frame_to_ascii};
use crate::error::{AppError, Result};
use crate::video;

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub input: PathBuf,
    pub output: PathBuf,
    pub columns: u32,
    pub fps: Option<f64>,
    pub charset: String,
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

    let options = AsciiOptions::new(config.columns, &config.charset);

    for (index, frame_path) in frames.iter().enumerate() {
        let image = image::open(frame_path)?.to_luma8();
        let ascii = convert_frame_to_ascii(&image, &options);
        let output_frame = ascii_dir.join(format!("frame_{index:08}.png"));
        ascii.save(output_frame)?;
    }

    video::encode_video(&ascii_dir, &config.input, fps, &config.output)?;

    Ok(PipelineStats {
        frames_processed: frames.len(),
        output_fps: fps,
    })
}
