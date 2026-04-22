use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "blueprinter")]
#[command(version)]
#[command(about = "Hand-drawn style diagram renderer CLI")]
#[command(long_about = "Turn Mermaid, draw.io, and any SVG into sketchy SVG/PNG/WebP.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a diagram into hand-drawn style output
    Render {
        /// Input file path
        #[arg(short, long)]
        input: String,

        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Theme name (blueprint, sumi, chalk, marker, watercolor, manga)
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

        /// Theme name
        #[arg(short, long, default_value = "blueprint")]
        theme: String,

        /// Seed for reproducible output
        #[arg(long)]
        seed: Option<u64>,
    },
    /// Convert input to another format (SVG -> PNG/WebP)
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
            println!("Rendering: {} -> {} (theme: {})", input, output, theme);
            if let Some(s) = seed {
                println!("Seed: {}", s);
            }
            // TODO: implement render logic
        }
        Commands::Transform {
            input,
            output,
            theme,
            seed,
        } => {
            println!("Transforming: {} -> {} (theme: {})", input, output, theme);
            if let Some(s) = seed {
                println!("Seed: {}", s);
            }
            // TODO: implement transform logic
        }
        Commands::Convert { input, output } => {
            println!("Converting: {} -> {}", input, output);
            // TODO: implement convert logic
        }
    }
}
