use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Copy)]
pub struct VideoMetadata {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
}

pub fn tools_available() -> bool {
    command_success("ffmpeg", &["-version"]) && command_success("ffprobe", &["-version"])
}

pub fn probe_video(input: &Path) -> Result<VideoMetadata> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,r_frame_rate",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(input)
        .output()
        .map_err(|source| AppError::CommandSpawn {
            program: "ffprobe".to_string(),
            source,
        })?;

    ensure_command_success("ffprobe", &output)?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();

    let width = lines
        .next()
        .ok_or_else(|| AppError::ProbeParse("missing width".to_string()))?
        .trim()
        .parse::<u32>()
        .map_err(|_| AppError::ProbeParse("invalid width".to_string()))?;

    let height = lines
        .next()
        .ok_or_else(|| AppError::ProbeParse("missing height".to_string()))?
        .trim()
        .parse::<u32>()
        .map_err(|_| AppError::ProbeParse("invalid height".to_string()))?;

    let frame_rate = lines
        .next()
        .ok_or_else(|| AppError::ProbeParse("missing frame rate".to_string()))?
        .trim();
    let fps = parse_rational(frame_rate)
        .ok_or_else(|| AppError::ProbeParse(format!("invalid frame rate: {frame_rate}")))?;

    Ok(VideoMetadata { width, height, fps })
}

pub fn extract_frames(input: &Path, output_dir: &Path) -> Result<Vec<PathBuf>> {
    fs::create_dir_all(output_dir)?;
    let frame_pattern = output_dir.join("frame_%08d.png");

    let output = Command::new("ffmpeg")
        .args(["-y", "-v", "error", "-i"])
        .arg(input)
        .args(["-vsync", "0"])
        .arg(&frame_pattern)
        .output()
        .map_err(|source| AppError::CommandSpawn {
            program: "ffmpeg".to_string(),
            source,
        })?;

    ensure_command_success("ffmpeg", &output)?;

    let mut files: Vec<PathBuf> = fs::read_dir(output_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension() == Some(OsStr::new("png")))
        .collect();
    files.sort();

    if files.is_empty() {
        return Err(AppError::NoFramesExtracted);
    }

    Ok(files)
}

pub fn encode_video(
    ascii_frames_dir: &Path,
    source_video: &Path,
    fps: f64,
    output: &Path,
    transparent: bool,
) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let frame_pattern = ascii_frames_dir.join("frame_%08d.png");
    let fps_string = format!("{fps:.6}");

    let output_cmd = if transparent {
        // WebP with transparency
        Command::new("ffmpeg")
            .args(["-y", "-v", "error", "-framerate"])
            .arg(&fps_string)
            .arg("-i")
            .arg(&frame_pattern)
            .args([
                "-c:v",
                "libwebp",
                "-pix_fmt",
                "yuva420p", // Include alpha channel
                "-quality",
                "95",
                "-loop",
                "0", // Loop infinitely
            ])
            .arg(output)
            .output()
            .map_err(|source| AppError::CommandSpawn {
                program: "ffmpeg".to_string(),
                source,
            })?
    } else {
        // MP4 with H.264 (original behavior)
        Command::new("ffmpeg")
            .args(["-y", "-v", "error", "-framerate"])
            .arg(&fps_string)
            .arg("-i")
            .arg(&frame_pattern)
            .arg("-i")
            .arg(source_video)
            .args([
                "-map",
                "0:v:0",
                "-map",
                "1:a?",
                "-c:v",
                "libx264",
                "-preset",
                "veryfast",
                "-crf",
                "18",
                "-pix_fmt",
                "yuv420p",
                "-tune",
                "stillimage",
                "-c:a",
                "copy",
                "-shortest",
            ])
            .arg(output)
            .output()
            .map_err(|source| AppError::CommandSpawn {
                program: "ffmpeg".to_string(),
                source,
            })?
    };

    ensure_command_success("ffmpeg", &output_cmd)
}

pub fn create_comparison_video(
    original: &Path,
    ascii_video: &Path,
) -> Result<()> {
    // Determine output path (original + ASCII, stacked)
    let output = original.with_file_name(
        original
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string() + "_compare." + ascii_video.extension().unwrap().to_string_lossy().as_ref()
    );

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    // Use ffmpeg's vstack filter to stack videos vertically
    let output_cmd = Command::new("ffmpeg")
        .args(["-y", "-v", "error"])
        .arg("-i")
        .arg(original)
        .arg("-i")
        .arg(ascii_video)
        .args([
            "-filter_complex",
            "[0:v][1:v]vstack",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "18",
            "-pix_fmt",
            "yuv420p",
            "-tune",
            "stillimage",
        ])
        .arg(&output)
        .output()
        .map_err(|source| AppError::CommandSpawn {
            program: "ffmpeg".to_string(),
            source,
        })?;

    ensure_command_success("ffmpeg", &output_cmd)?;

    // Replace the ASCII video with the comparison video
    fs::rename(&output, ascii_video)?;

    Ok(())
}

pub fn create_test_video(
    output: &Path,
    width: u32,
    height: u32,
    fps: u32,
    duration_seconds: f32,
) -> Result<()> {
    let size = format!("{width}x{height}");
    let rate = fps.to_string();
    let duration = format!("{duration_seconds}");

    let output_cmd = Command::new("ffmpeg")
        .args(["-y", "-v", "error", "-f", "lavfi", "-i"])
        .arg(format!(
            "testsrc=size={size}:rate={rate}:duration={duration}"
        ))
        .arg(output)
        .output()
        .map_err(|source| AppError::CommandSpawn {
            program: "ffmpeg".to_string(),
            source,
        })?;

    ensure_command_success("ffmpeg", &output_cmd)
}

fn parse_rational(value: &str) -> Option<f64> {
    if let Some((num, den)) = value.split_once('/') {
        let numerator = num.parse::<f64>().ok()?;
        let denominator = den.parse::<f64>().ok()?;
        if denominator == 0.0 {
            None
        } else {
            Some(numerator / denominator)
        }
    } else {
        value.parse::<f64>().ok()
    }
}

fn ensure_command_success(program: &str, output: &std::process::Output) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(AppError::CommandFailed {
        program: program.to_string(),
        code: output.status.code(),
        stderr,
    })
}

fn command_success(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rational_frame_rate() {
        assert_eq!(parse_rational("30000/1001").unwrap().round(), 30.0);
        assert_eq!(parse_rational("24").unwrap(), 24.0);
        assert!(parse_rational("1/0").is_none());
        assert!(parse_rational("abc").is_none());
    }
}
