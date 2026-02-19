use clap::Parser;
use video_ascii_cli::cli::Cli;
use video_ascii_cli::pipeline::{PipelineConfig, run};

fn main() {
    let cli = Cli::parse();
    let config = PipelineConfig {
        input: cli.input.clone(),
        output: cli.output_path(),
        columns: cli.columns,
        fps: cli.fps,
        charset: cli.charset.clone(),
        shades: cli.shades,
        transparent: cli.transparent,
        bg_color: cli.bg_color,
        threshold: cli.threshold,
        compare: cli.compare,
    };

    if let Err(err) = run(&config) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
