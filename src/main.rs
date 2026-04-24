use clap::{Parser, Subcommand};
use std::fs;

use blueprinter::jitter::JitterConfig;
use blueprinter::svg::{transform_svg, TransformOptions};

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

        /// Override SVG text font-family while preserving layout
        #[arg(long)]
        font_family: Option<String>,

        /// Maximum coordinate offset applied to jittered geometry
        #[arg(long)]
        jitter_amplitude: Option<f64>,

        /// Segment density used to subdivide jittered strokes
        #[arg(long)]
        jitter_frequency: Option<f64>,

        /// Relative stroke-width variation applied per shape
        #[arg(long)]
        jitter_stroke_width_var: Option<f64>,
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
            font_family,
            jitter_amplitude,
            jitter_frequency,
            jitter_stroke_width_var,
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
            let config = jitter_config_from_flags(
                jitter_amplitude,
                jitter_frequency,
                jitter_stroke_width_var,
            );
            let options = TransformOptions {
                seed,
                font_family_override: font_family,
            };
            let transformed = match transform_svg(&svg, &config, &options) {
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

fn jitter_config_from_flags(
    amplitude: Option<f64>,
    frequency: Option<f64>,
    stroke_width_var: Option<f64>,
) -> JitterConfig {
    let mut config = JitterConfig::default();
    if let Some(value) = amplitude {
        config.amplitude = value;
    }
    if let Some(value) = frequency {
        config.frequency = value;
    }
    if let Some(value) = stroke_width_var {
        config.stroke_width_var = value;
    }
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_cli_defaults_match_jitter_defaults() {
        let cli =
            Cli::try_parse_from(["blueprinter", "transform", "-i", "in.svg", "-o", "out.svg"])
                .unwrap();

        let Commands::Transform {
            jitter_amplitude,
            jitter_frequency,
            jitter_stroke_width_var,
            font_family,
            ..
        } = cli.command
        else {
            panic!("expected transform command");
        };

        assert_eq!(font_family, None);

        assert_eq!(
            jitter_config_from_flags(jitter_amplitude, jitter_frequency, jitter_stroke_width_var),
            JitterConfig::default()
        );
    }

    #[test]
    fn transform_cli_accepts_explicit_jitter_flags() {
        let cli = Cli::try_parse_from([
            "blueprinter",
            "transform",
            "-i",
            "in.svg",
            "-o",
            "out.svg",
            "--jitter-amplitude",
            "3.5",
            "--jitter-frequency",
            "7",
            "--jitter-stroke-width-var",
            "0.4",
            "--font-family",
            "Virgil",
        ])
        .unwrap();

        let Commands::Transform {
            jitter_amplitude,
            jitter_frequency,
            jitter_stroke_width_var,
            font_family,
            ..
        } = cli.command
        else {
            panic!("expected transform command");
        };

        assert_eq!(font_family.as_deref(), Some("Virgil"));

        assert_eq!(
            jitter_config_from_flags(jitter_amplitude, jitter_frequency, jitter_stroke_width_var),
            JitterConfig {
                amplitude: 3.5,
                frequency: 7.0,
                stroke_width_var: 0.4,
            }
        );
    }
}
