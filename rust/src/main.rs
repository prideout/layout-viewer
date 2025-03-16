use anyhow::{anyhow, Result};
use clap::Parser;
use colored::*;
use layout_viewer::{Project, run_gl_window};
use std::fs;
use std::path::{Path, PathBuf};

fn pretty_print_float(value: f64) -> String {
    let value = format!("{:.4}", value);
    value.trim_end_matches('0').trim_end_matches('.').to_string()
}

struct StatsRow {
    name: ColoredString,
    value: usize,
}

impl StatsRow {
    fn new(name: &str, value: usize) -> Self {
        Self {
            name: name.to_string().color(Color::Green),
            value,
        }
    }
}

/// A GDSII layout viewer with SVG export capability
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input GDSII file to process
    #[arg(required = true)]
    input: PathBuf,

    /// Optional output SVG file to generate
    #[arg(value_name = "OUTPUT.svg")]
    output: Option<PathBuf>,

    /// Open OpenGL window with interactive visualization
    #[arg(long)]
    gl: bool,
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

fn main() -> Result<()> {
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
    let stats_rows = vec![
        StatsRow::new("Structs", stats.struct_count),
        StatsRow::new("Boundaries", stats.polygon_count),
        StatsRow::new("Paths", stats.path_count),
        StatsRow::new("SRefs", stats.sref_count),
        StatsRow::new("ARefs", stats.aref_count),
        StatsRow::new("Layers", (project.highest_layer() + 1) as usize),
    ];

    for row in stats_rows {
        println!("{:<12} {}", row.name, row.value);
    }

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
        let svg_content = project.to_svg().map_err(|e| {
            anyhow!(
                "Failed to generate SVG: {}",
                e.as_string().unwrap_or_default()
            )
        })?;

        fs::write(output_path, svg_content)?;
        println!("SVG file written to: {}", output_path.display());
    }

    println!();

    // Show GL window if requested
    if args.gl {
        run_gl_window()?;
    }

    Ok(())
}
