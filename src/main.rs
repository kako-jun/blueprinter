use clap::{Parser, Subcommand};
use std::fs;

use blueprinter::jitter::JitterConfig;
use blueprinter::svg::transform_svg;

#[derive(Parser)]
#[command(name = "blueprinter")]
#[command(version)]
#[command(about = "Hand-drawn style diagram renderer CLI")]
#[command(
    long_about = "Turn SVG into sketchy SVG. Mermaid, draw.io direct input, and raster export are planned."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a diagram into hand-drawn style output (planned; not implemented yet)
    Render {
        /// Input file path
        #[arg(short, long)]
        input: String,

        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Theme name (currently only blueprint is accepted)
        #[arg(short, long, default_value = "blueprint")]
        theme: String,

        /// Seed for reproducible output
        #[arg(long)]
        seed: Option<u64>,
    },
    /// Transform an existing SVG's appearance without changing layout
    Transform {
        /// Input SVG file path
        #[arg(short, long)]
        input: String,

        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Theme name (currently only blueprint is accepted)
        #[arg(short, long, default_value = "blueprint")]
        theme: String,

        /// Seed for reproducible output
        #[arg(long)]
        seed: Option<u64>,
    },
    /// Convert input to another format (planned; not implemented yet)
    Convert {
        /// Input file path
        #[arg(short, long)]
        input: String,

        /// Output file path
        #[arg(short, long)]
        output: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Render {
            input,
            output,
            theme,
            seed,
        } => {
            eprintln!(
                "Error: render is not implemented yet. Convert Mermaid/draw.io to SVG first, then use `transform`."
            );
            let _ = (input, output, theme, seed);
            std::process::exit(1);
        }
        Commands::Transform {
            input,
            output,
            theme,
            seed,
        } => {
            let svg = match fs::read_to_string(&input) {
                Ok(svg) => svg,
                Err(err) => {
                    eprintln!("Error: failed to read input SVG: {err}");
                    std::process::exit(1);
                }
            };
            if theme != "blueprint" {
                eprintln!("Error: theme `{theme}` is not implemented yet. Currently only `blueprint` works.");
                std::process::exit(1);
            }
            let config = JitterConfig::default();
            let transformed = match transform_svg(&svg, &config, seed) {
                Ok(svg) => svg,
                Err(err) => {
                    eprintln!("Error: failed to transform SVG: {err}");
                    std::process::exit(1);
                }
            };
            if let Err(err) = fs::write(&output, transformed) {
                eprintln!("Error: failed to write output SVG: {err}");
                std::process::exit(1);
            }
            println!("Transformed: {input} -> {output} (theme: {theme})");
        }
        Commands::Convert { input, output } => {
            eprintln!("Error: convert is not implemented yet.");
            let _ = (input, output);
            std::process::exit(1);
        }
    }
}
