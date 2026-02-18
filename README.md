# video-ascii-cli

A Rust CLI tool that converts a video into black-and-white ASCII art video frames and writes a new output video.

## Features

- Reads an input video file
- Converts each frame into ASCII art
- Renders black-and-white ASCII frames into a new video
- Preserves source audio when available
- Includes unit and integration tests

## Requirements

- Rust toolchain (edition 2024)
- `ffmpeg` and `ffprobe` available on `PATH`

## Build

```bash
cargo build --release
```

## Usage

Basic usage:

```bash
cargo run -- input.mp4
```

Write to a specific output file:

```bash
cargo run -- input.mp4 --output output_ascii.mp4
```

Tune ASCII density and fps:

```bash
cargo run -- input.mp4 --columns 160 --fps 24
```

Use a custom character set from dark to light:

```bash
cargo run -- input.mp4 --charset "@#*:. "
```

## CLI Arguments

- `input` (positional): path to input video
- `-o, --output <PATH>`: output video path (default: `<input_stem>_ascii.mp4`)
- `--columns <N>`: number of ASCII columns per frame (default: `120`)
- `--fps <FPS>`: override output framerate
- `--charset <CHARS>`: ASCII characters ordered from dark to light

## Test

```bash
cargo test
```

Notes:

- Tests that require ffmpeg are auto-skipped when `ffmpeg`/`ffprobe` are not available.
