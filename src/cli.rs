use std::path::{Path, PathBuf};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Convert video frames into black-and-white ASCII art"
)]
pub struct Cli {
    /// Input video path
    pub input: PathBuf,

    /// Output video path (defaults to <input-stem>_ascii.mp4)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Number of ASCII columns per frame
    #[arg(long, default_value_t = 120)]
    pub columns: u32,

    /// Override output framerate
    #[arg(long)]
    pub fps: Option<f64>,

    /// Characters from dark to light
    #[arg(long, default_value = "@%#*+=-:. ")]
    pub charset: String,

    /// Number of grayscale shades (1 = pure B/W, 2-256 = grayscale depth)
    #[arg(long, default_value_t = 1)]
    pub shades: u32,

    /// Make background transparent (outputs WebP instead of MP4)
    #[arg(long)]
    pub transparent: bool,

    /// Background color to remove (0-255, default: auto-detect)
    #[arg(long)]
    pub bg_color: Option<u8>,

    /// Tolerance for background detection (0-255, default: 0 = exact match).
    /// Pixels whose grayscale value is within ±threshold of the background color
    /// are treated as background and made transparent.
    /// Example: --transparent --threshold 10  (removes colors within ±10 of bg)
    #[arg(long, default_value_t = 0)]
    pub threshold: u8,

    /// Create a comparison video with original and ASCII versions stacked vertically
    #[arg(long)]
    pub compare: bool,
}

impl Cli {
    pub fn output_path(&self) -> PathBuf {
        match &self.output {
            Some(path) => path.clone(),
            None => default_output_path(&self.input, self.transparent, self.compare),
        }
    }
}

fn default_output_path(input: &Path, transparent: bool, compare: bool) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let mut output = input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    let ext = if transparent { "webp" } else { "mp4" };

    if compare {
        output.push(format!("{stem}_compare.{ext}"));
    } else {
        output.push(format!("{stem}_ascii.{ext}"));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn derives_default_output_path() {
        let cli = Cli::parse_from(["video-ascii-cli", "input.mp4"]);
        assert_eq!(cli.output_path(), PathBuf::from("input_ascii.mp4"));

        let cli_transparent = Cli::parse_from(["video-ascii-cli", "input.mp4", "--transparent"]);
        assert_eq!(cli_transparent.output_path(), PathBuf::from("input_ascii.webp"));
    }

    #[test]
    fn parses_custom_args() {
        let cli = Cli::parse_from([
            "video-ascii-cli",
            "in.mp4",
            "--output",
            "out.mp4",
            "--columns",
            "80",
            "--fps",
            "12",
            "--charset",
            "# ",
            "--shades",
            "4",
        ]);

        assert_eq!(cli.input, PathBuf::from("in.mp4"));
        assert_eq!(cli.output, Some(PathBuf::from("out.mp4")));
        assert_eq!(cli.columns, 80);
        assert_eq!(cli.fps, Some(12.0));
        assert_eq!(cli.charset, "# ");
        assert_eq!(cli.shades, 4);
    }

    #[test]
    fn threshold_defaults_to_zero() {
        let cli = Cli::parse_from(["video-ascii-cli", "input.mp4"]);
        assert_eq!(cli.threshold, 0);
    }

    #[test]
    fn parses_threshold_flag() {
        let cli = Cli::parse_from([
            "video-ascii-cli",
            "input.mp4",
            "--transparent",
            "--threshold",
            "15",
        ]);
        assert!(cli.transparent);
        assert_eq!(cli.threshold, 15);
    }
}
