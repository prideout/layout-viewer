use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use colored::*;
use layout_viewer::generate_svg;
use layout_viewer::Project;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input GDSII file to process
    #[arg(required = true)]
    pub input: PathBuf,

    /// Optional output SVG file to generate
    #[arg(value_name = "OUTPUT.svg")]
    pub output: Option<PathBuf>,

    /// Request OpenGL window with interactive visualization
    #[arg(long)]
    pub gl: bool,
}

fn verify_file_extension(path: &Path, expected: &str) -> Result<()> {
    match path.extension() {
        Some(ext) if ext.to_string_lossy() == expected => Ok(()),
        _ => Err(anyhow!(
            "File '{}' must have .{} extension",
            path.display(),
            expected
        )),
    }
}

fn pretty_print_float(value: f64) -> String {
    let value = format!("{:.4}", value);
    value
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

pub fn run_cli() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    // Verify file extensions
    verify_file_extension(&args.input, "gds")?;
    if let Some(ref output_path) = args.output {
        verify_file_extension(output_path, "svg")?;
    }

    println!(
        "Reading {}...",
        args.input.file_name().unwrap().to_string_lossy()
    );

    // Read and process the GDSII file
    let file_content = fs::read(&args.input)?;
    let project = Project::from_bytes(&file_content)?;

    let stats = project.stats();
    println!(
        "{:<12} {}",
        "Structs".color(Color::Green),
        stats.struct_count
    );
    println!(
        "{:<12} {}",
        "Boundaries".color(Color::Green),
        stats.polygon_count
    );
    println!("{:<12} {}", "Paths".color(Color::Green), stats.path_count);
    println!("{:<12} {}", "SRefs".color(Color::Green), stats.sref_count);
    println!("{:<12} {}", "ARefs".color(Color::Green), stats.aref_count);
    println!(
        "{:<12} {}",
        "Layers".color(Color::Green),
        (project.highest_layer() + 1) as usize
    );

    let bounds = project.bounds();
    println!(
        "{:<12} ({}, {}) to ({}, {})",
        "Bounds".color(Color::BrightYellow),
        pretty_print_float(bounds.min_x),
        pretty_print_float(bounds.min_y),
        pretty_print_float(bounds.max_x),
        pretty_print_float(bounds.max_y)
    );

    let mut has_root_cell = false;
    for root_id in project.find_roots() {
        has_root_cell = true;
        println!(
            "{:<12} {}",
            "Root".color(Color::BrightYellow),
            project.struct_name(root_id)
        );
    }

    if !has_root_cell {
        println!("{}", "No root cell found".color(Color::Red));
    }

    // Generate and save SVG if output path is provided
    if let Some(ref output_path) = args.output {
        let svg_content = generate_svg(project.layers());

        fs::write(output_path, svg_content)?;
        println!("SVG file written to: {}", output_path.display());
    }

    println!();

    if args.gl {
        layout_viewer::spawn_window(project)?;
    }

    Ok(())
}
