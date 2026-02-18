use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("input file does not exist: {0}")]
    InputNotFound(PathBuf),

    #[error("ffmpeg and ffprobe must be installed and available on PATH")]
    MissingFfmpeg,

    #[error("failed to run command `{program}`: {source}")]
    CommandSpawn {
        program: String,
        #[source]
        source: std::io::Error,
    },

    #[error("command `{program}` failed (exit code {code:?}): {stderr}")]
    CommandFailed {
        program: String,
        code: Option<i32>,
        stderr: String,
    },

    #[error("failed to parse ffprobe output: {0}")]
    ProbeParse(String),

    #[error("no frames were extracted from the input video")]
    NoFramesExtracted,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Image(#[from] image::ImageError),
}
