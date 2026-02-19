use image::{GrayImage, Luma};
use tempfile::TempDir;

use video_ascii_cli::ascii::{AsciiOptions, convert_frame_to_ascii};
use video_ascii_cli::pipeline::{PipelineConfig, run};
use video_ascii_cli::video;

fn skip_if_no_ffmpeg() -> bool {
    if !video::tools_available() {
        eprintln!("Skipping ffmpeg-dependent test: ffmpeg/ffprobe not available.");
        true
    } else {
        false
    }
}

#[test]
fn video_loading_reads_metadata() {
    if skip_if_no_ffmpeg() {
        return;
    }

    let temp = TempDir::new().expect("temp dir");
    let video_path = temp.path().join("input.mp4");

    video::create_test_video(&video_path, 64, 48, 5, 1.0).expect("create test video");
    let meta = video::probe_video(&video_path).expect("probe metadata");

    assert_eq!(meta.width, 64);
    assert_eq!(meta.height, 48);
    assert!((meta.fps - 5.0).abs() < 0.2);
}

#[test]
fn ascii_conversion_outputs_black_and_white_pixels() {
    let mut source = GrayImage::from_pixel(32, 24, Luma([255]));
    for y in 0..12 {
        for x in 0..16 {
            source.put_pixel(x, y, Luma([0]));
        }
    }

    let options = AsciiOptions::new(8, "@ ", 1);
    let converted = convert_frame_to_ascii(&source, &options);

    // Source 32x24 → 4 columns x 3 rows (32/8, 24/8)
    // Output: 4*8 x 3*8 = 32 x 24
    assert_eq!(converted.width(), 4 * 8);
    assert_eq!(converted.height(), 3 * 8);
    assert!(converted.pixels().all(|p| p[0] == 0 || p[0] == 255));
}

#[test]
fn output_generation_creates_ascii_video_file() {
    if skip_if_no_ffmpeg() {
        return;
    }

    let temp = TempDir::new().expect("temp dir");
    let input = temp.path().join("input.mp4");
    let output = temp.path().join("output_ascii.mp4");

    video::create_test_video(&input, 80, 60, 6, 1.0).expect("create test video");

    let config = PipelineConfig {
        input: input.clone(),
        output: output.clone(),
        columns: 20,
        fps: Some(6.0),
        charset: "@%#*+=-:. ".to_string(),
        shades: 1,
        transparent: false,
        bg_color: None,
        threshold: 0,
        compare: false,
    };

    let stats = run(&config).expect("run pipeline");

    assert!(output.exists());
    assert!(stats.frames_processed > 0);
    assert!((stats.output_fps - 6.0).abs() < 0.01);

    let output_meta = video::probe_video(&output).expect("probe output video");
    // Input 80x60 → 10 columns x 7 rows (80/8, 60/8 rounded down)
    // Output: 10*8 x 7*8 = 80 x 56
    assert_eq!(output_meta.width, 80);
    assert_eq!(output_meta.height, 56);
}
