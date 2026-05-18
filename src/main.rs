use clap::{Args, Parser, Subcommand};
use std::fs;
use std::path::Path;

use blueprinter::jitter::JitterConfig;
use blueprinter::render::{extract_mermaid_blocks, mermaid_to_svg, RenderError};
use blueprinter::svg::{
    export_to_png, export_to_webp, theme_style, transform_svg, Theme, TransformOptions,
    DEFAULT_SEED,
};

#[derive(Parser)]
#[command(name = "blueprinter")]
#[command(version)]
#[command(about = "Hand-drawn style diagram renderer CLI")]
#[command(
    long_about = "Render structured diagrams (Mermaid, draw.io-planned) as hand-drawn raster images. \
PNG/WebP are the primary outputs; SVG output remains available for debugging the pipeline."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Styling options shared by `render` and `transform`.
#[derive(Args)]
struct StyleArgs {
    /// Theme name (blueprint, sumi, watercolor, chalk, marker, manga, none)
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

    /// Extra font directory loaded into the rasterizer's fontdb (for raster
    /// output). Useful for cross-platform reproducibility — drop the desired
    /// TTF/OTF files in one folder and pass it here.
    #[arg(long)]
    font_dir: Option<String>,
}

/// Output options shared by `render` and `transform`.
#[derive(Args)]
struct OutputArgs {
    /// Output file path
    #[arg(short, long)]
    output: String,

    /// Output format (png, webp, svg). Inferred from output extension if
    /// omitted; defaults to png when the extension is unrecognised. svg is
    /// debug-only.
    #[arg(long)]
    format: Option<String>,

    /// Scale factor for raster output (default: 1.0)
    #[arg(long, default_value = "1.0")]
    scale: f32,

    /// Explicit output width (in pixels, for raster formats)
    #[arg(long)]
    width: Option<u32>,

    /// Explicit output height (in pixels, for raster formats)
    #[arg(long)]
    height: Option<u32>,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a Mermaid diagram (via external `mmdc`) into hand-drawn output
    Render {
        /// Input Mermaid file path (.mmd / .mermaid)
        #[arg(short, long)]
        input: String,

        #[command(flatten)]
        style: StyleArgs,

        #[command(flatten)]
        output_args: OutputArgs,
    },
    /// Transform an existing SVG's appearance without changing layout
    Transform {
        /// Input SVG file path
        #[arg(short, long)]
        input: String,

        #[command(flatten)]
        style: StyleArgs,

        #[command(flatten)]
        output_args: OutputArgs,
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
    /// Batch-render every ` ```mermaid ` block in a Markdown file
    Md {
        /// Input Markdown file path
        #[arg(short, long)]
        input: String,

        /// Output directory (created if it does not exist). Files are named
        /// `<md-stem>-<index>.<ext>` where index starts at 1.
        #[arg(short, long)]
        out_dir: String,

        #[command(flatten)]
        style: StyleArgs,

        /// Output format (png, webp, svg). Default: png. svg is debug-only.
        #[arg(long, default_value = "png")]
        format: String,

        /// Scale factor for raster output (default: 1.0)
        #[arg(long, default_value = "1.0")]
        scale: f32,

        /// Explicit output width (in pixels, for raster formats)
        #[arg(long)]
        width: Option<u32>,

        /// Explicit output height (in pixels, for raster formats)
        #[arg(long)]
        height: Option<u32>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Render {
            input,
            style,
            output_args,
        } => {
            let mermaid = read_input(&input);
            let svg = match mermaid_to_svg(&mermaid) {
                Ok(svg) => svg,
                Err(RenderError::MmdcNotFound) => {
                    eprintln!("Error: {}", RenderError::MmdcNotFound);
                    std::process::exit(127);
                }
                Err(err) => {
                    eprintln!("Error: {err}");
                    std::process::exit(1);
                }
            };
            run_pipeline(&svg, &input, &style, &output_args, "rendered");
        }
        Commands::Transform {
            input,
            style,
            output_args,
        } => {
            let svg = read_input(&input);
            run_pipeline(&svg, &input, &style, &output_args, "transformed");
        }
        Commands::Convert { input, output } => {
            eprintln!("Error: convert is not implemented yet.");
            let _ = (input, output);
            std::process::exit(1);
        }
        Commands::Md {
            input,
            out_dir,
            style,
            format,
            scale,
            width,
            height,
        } => {
            run_md_batch(&input, &out_dir, &style, &format, scale, width, height);
        }
    }
}

fn run_md_batch(
    input_path: &str,
    out_dir: &str,
    style: &StyleArgs,
    format: &str,
    scale: f32,
    width: Option<u32>,
    height: Option<u32>,
) {
    let md = read_input(input_path);
    let blocks = extract_mermaid_blocks(&md);
    if blocks.is_empty() {
        eprintln!("No `mermaid` code blocks found in {input_path}.");
        std::process::exit(0);
    }

    if let Err(err) = fs::create_dir_all(out_dir) {
        eprintln!("Error: failed to create output directory '{out_dir}': {err}");
        std::process::exit(1);
    }

    let stem = Path::new(input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("diagram");
    let ext = match format {
        "svg" | "png" | "webp" => format,
        _ => {
            eprintln!("Error: unknown format '{format}'. Supported: png, webp, svg.");
            std::process::exit(1);
        }
    };

    let mut failures = 0usize;
    for (index, mermaid) in blocks.iter().enumerate() {
        let n = index + 1;
        let out_path = Path::new(out_dir).join(format!("{stem}-{n}.{ext}"));
        let out_str = out_path.to_string_lossy().into_owned();

        let svg = match mermaid_to_svg(mermaid) {
            Ok(svg) => svg,
            Err(RenderError::MmdcNotFound) => {
                eprintln!("Error: {}", RenderError::MmdcNotFound);
                std::process::exit(127);
            }
            Err(err) => {
                eprintln!("[{n}/{total}] mmdc failed: {err}", total = blocks.len());
                failures += 1;
                continue;
            }
        };

        let output_args = OutputArgs {
            output: out_str.clone(),
            format: Some(ext.to_string()),
            scale,
            width,
            height,
        };
        let label = format!("{input_path}#{n}");
        run_pipeline(&svg, &label, style, &output_args, "rendered");
    }

    if failures > 0 {
        eprintln!(
            "{failures}/{total} blocks failed (other blocks were written successfully).",
            total = blocks.len(),
        );
        std::process::exit(1);
    }
}

fn read_input(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error: failed to read input '{path}': {err}");
            std::process::exit(1);
        }
    }
}

fn run_pipeline(svg: &str, input_label: &str, style: &StyleArgs, out: &OutputArgs, verb: &str) {
    let theme_enum = match parse_theme(&style.theme) {
        Some(t) => t,
        None => {
            eprintln!(
                "Error: theme `{}` is not supported. Valid: blueprint, sumi, watercolor, chalk, marker, manga, none.",
                style.theme
            );
            std::process::exit(1);
        }
    };
    let config = jitter_config_from_flags(
        style.jitter_amplitude,
        style.jitter_frequency,
        style.jitter_stroke_width_var,
    );
    let options = TransformOptions {
        seed: style.seed,
        font_family_override: style.font_family.clone(),
        theme: theme_enum,
    };
    let transformed = match transform_svg(svg, &config, &options) {
        Ok(svg) => svg,
        Err(err) => {
            eprintln!("Error: failed to transform SVG: {err}");
            std::process::exit(1);
        }
    };

    let output_format = out
        .format
        .as_deref()
        .unwrap_or_else(|| infer_format_from_path(&out.output));

    let font_dir = style.font_dir.as_deref().map(Path::new);
    let bleed_params = theme_style(theme_enum).bleed_pass_params();
    // Seed forwarded to the aquarelle raster bleed pass; falls back to a
    // fixed value so omitting --seed still produces a deterministic bleed.
    let bleed_seed = style.seed.unwrap_or(DEFAULT_SEED);
    let result = match output_format {
        "svg" => fs::write(&out.output, &transformed).map_err(|e| e.to_string()),
        "png" => export_to_png(
            &transformed,
            build_dimensions(out.width, out.height),
            out.scale,
            font_dir,
            bleed_params,
            bleed_seed,
        )
        .and_then(|bytes| fs::write(&out.output, bytes).map_err(|e| e.to_string())),
        "webp" => export_to_webp(
            &transformed,
            build_dimensions(out.width, out.height),
            out.scale,
            font_dir,
            bleed_params,
            bleed_seed,
        )
        .and_then(|bytes| fs::write(&out.output, bytes).map_err(|e| e.to_string())),
        _ => {
            eprintln!("Error: unknown format '{output_format}'. Supported: png, webp, svg.");
            std::process::exit(1);
        }
    };

    if let Err(err) = result {
        eprintln!("Error: failed to write output {output_format}: {err}");
        std::process::exit(1);
    }

    println!(
        "{verb}: {input_label} -> {output} (theme: {theme}, format: {output_format})",
        output = out.output,
        theme = style.theme,
    );
}

fn parse_theme(name: &str) -> Option<Theme> {
    match name {
        "blueprint" => Some(Theme::Blueprint),
        "sumi" => Some(Theme::Sumi),
        "watercolor" => Some(Theme::Watercolor),
        "chalk" => Some(Theme::Chalk),
        "marker" => Some(Theme::Marker),
        "manga" => Some(Theme::Manga),
        "none" => Some(Theme::None),
        _ => None,
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

fn infer_format_from_path(path: &str) -> &'static str {
    let path = Path::new(path);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("png") => "png",
        Some("webp") => "webp",
        Some("svg") => "svg",
        _ => "png",
    }
}

fn build_dimensions(width: Option<u32>, height: Option<u32>) -> Option<(Option<u32>, Option<u32>)> {
    match (width, height) {
        (None, None) => None,
        (Some(w), Some(h)) => Some((Some(w), Some(h))),
        (Some(w), None) => Some((Some(w), None)),
        (None, Some(h)) => Some((None, Some(h))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_transform_command(cli: Cli) -> (StyleArgs, OutputArgs) {
        match cli.command {
            Commands::Transform {
                style, output_args, ..
            } => (style, output_args),
            _ => panic!("expected transform command"),
        }
    }

    #[test]
    fn transform_cli_defaults_match_jitter_defaults() {
        let cli =
            Cli::try_parse_from(["blueprinter", "transform", "-i", "in.svg", "-o", "out.svg"])
                .unwrap();
        let (style, out) = assert_transform_command(cli);

        assert_eq!(style.font_family, None);
        assert_eq!(out.scale, 1.0);
        assert_eq!(out.width, None);
        assert_eq!(out.height, None);
        assert_eq!(out.format, None);

        assert_eq!(
            jitter_config_from_flags(
                style.jitter_amplitude,
                style.jitter_frequency,
                style.jitter_stroke_width_var
            ),
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
        let (style, _) = assert_transform_command(cli);

        assert_eq!(style.font_family.as_deref(), Some("Virgil"));
        assert_eq!(
            jitter_config_from_flags(
                style.jitter_amplitude,
                style.jitter_frequency,
                style.jitter_stroke_width_var
            ),
            JitterConfig {
                amplitude: 3.5,
                frequency: 7.0,
                stroke_width_var: 0.4,
            }
        );
    }

    #[test]
    fn render_cli_accepts_same_style_flags_as_transform() {
        let cli = Cli::try_parse_from([
            "blueprinter",
            "render",
            "-i",
            "diagram.mmd",
            "-o",
            "out.png",
            "--theme",
            "manga",
            "--seed",
            "7",
            "--width",
            "800",
        ])
        .unwrap();

        let Commands::Render {
            input,
            style,
            output_args,
        } = cli.command
        else {
            panic!("expected render command");
        };

        assert_eq!(input, "diagram.mmd");
        assert_eq!(style.theme, "manga");
        assert_eq!(style.seed, Some(7));
        assert_eq!(output_args.width, Some(800));
    }

    #[test]
    fn parse_theme_known_values() {
        assert_eq!(parse_theme("manga"), Some(Theme::Manga));
        assert_eq!(parse_theme("chalk"), Some(Theme::Chalk));
        assert_eq!(parse_theme("none"), Some(Theme::None));
        assert_eq!(parse_theme("nonsense"), None);
    }

    #[test]
    fn infer_format_from_path_svg() {
        assert_eq!(infer_format_from_path("output.svg"), "svg");
    }

    #[test]
    fn infer_format_from_path_png() {
        assert_eq!(infer_format_from_path("output.png"), "png");
    }

    #[test]
    fn infer_format_from_path_webp() {
        assert_eq!(infer_format_from_path("output.webp"), "webp");
    }

    #[test]
    fn infer_format_from_path_unknown_extension_falls_back_to_png() {
        assert_eq!(infer_format_from_path("output.txt"), "png");
    }

    #[test]
    fn build_dimensions_both() {
        assert_eq!(
            build_dimensions(Some(100), Some(200)),
            Some((Some(100), Some(200)))
        );
    }

    #[test]
    fn build_dimensions_width_only() {
        assert_eq!(build_dimensions(Some(100), None), Some((Some(100), None)));
    }

    #[test]
    fn build_dimensions_height_only() {
        assert_eq!(build_dimensions(None, Some(200)), Some((None, Some(200))));
    }

    #[test]
    fn build_dimensions_none() {
        assert_eq!(build_dimensions(None, None), None);
    }

    #[test]
    fn infer_format_from_path_no_extension_falls_back_to_png() {
        assert_eq!(infer_format_from_path("output"), "png");
    }

    #[test]
    fn infer_format_from_path_trailing_dot_falls_back_to_png() {
        assert_eq!(infer_format_from_path("output."), "png");
    }

    #[test]
    fn infer_format_from_path_uppercase_png_falls_back_to_png() {
        assert_eq!(infer_format_from_path("OUTPUT.PNG"), "png");
    }

    #[test]
    fn infer_format_from_path_uppercase_svg_falls_back_to_png_not_svg() {
        assert_eq!(infer_format_from_path("OUTPUT.SVG"), "png");
    }

    #[test]
    fn md_cli_format_defaults_to_png() {
        let cli =
            Cli::try_parse_from(["blueprinter", "md", "-i", "in.md", "-o", "out_dir"]).unwrap();
        let Commands::Md { format, .. } = cli.command else {
            panic!("expected md command");
        };
        assert_eq!(format, "png");
    }

    #[test]
    fn md_cli_format_explicit_svg_preserved() {
        let cli = Cli::try_parse_from([
            "blueprinter",
            "md",
            "-i",
            "in.md",
            "-o",
            "out_dir",
            "--format",
            "svg",
        ])
        .unwrap();
        let Commands::Md { format, .. } = cli.command else {
            panic!("expected md command");
        };
        assert_eq!(format, "svg");
    }

    #[test]
    fn transform_cli_format_explicit_svg_preserved() {
        let cli = Cli::try_parse_from([
            "blueprinter",
            "transform",
            "-i",
            "in.svg",
            "-o",
            "out.png",
            "--format",
            "svg",
        ])
        .unwrap();
        let (_, output_args) = assert_transform_command(cli);
        assert_eq!(output_args.format.as_deref(), Some("svg"));
    }
}
