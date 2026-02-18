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
}

impl Cli {
    pub fn output_path(&self) -> PathBuf {
        match &self.output {
            Some(path) => path.clone(),
            None => default_output_path(&self.input),
        }
    }
}

fn default_output_path(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let mut output = input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    output.push(format!("{stem}_ascii.mp4"));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn derives_default_output_path() {
        let cli = Cli::parse_from(["video-ascii-cli", "input.mp4"]);
        assert_eq!(cli.output_path(), PathBuf::from("./input_ascii.mp4"));
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
        ]);

        assert_eq!(cli.input, PathBuf::from("in.mp4"));
        assert_eq!(cli.output, Some(PathBuf::from("out.mp4")));
        assert_eq!(cli.columns, 80);
        assert_eq!(cli.fps, Some(12.0));
        assert_eq!(cli.charset, "# ");
    }
}
