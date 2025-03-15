use anyhow::{anyhow, Result};
use colored::*;
use layout_viewer::Project;
use std::fs;
use std::path::Path;

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

fn print_usage(program_name: &str) {
    println!("Usage: {} <input.gds> [output.svg]", program_name);
    println!("\nArguments:");
    println!("  <input.gds>   Input GDSII file to process");
    println!("  [output.svg]  Optional output SVG file to generate");
    println!("\nOptions:");
    println!("  --help        Display this help message");
    println!("\nIf output.svg is omitted, only statistics will be displayed.");
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
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 2 && args[1] == "--help" {
        print_usage(&args[0]);
        return Ok(());
    }

    if args.len() < 2 || args.len() > 3 {
        return Err(anyhow!("Usage: {} <input.gds> [output.svg]", args[0]));
    }

    let input_path = Path::new(&args[1]);
    let output_path = if args.len() == 3 {
        Some(Path::new(&args[2]))
    } else {
        None
    };

    // Verify file extensions
    verify_file_extension(input_path, "gds")?;
    if let Some(output_path) = output_path {
        verify_file_extension(output_path, "svg")?;
    }

    // Read and process the GDSII file
    let file_content = fs::read(input_path)?;
    let projects = Project::from_bytes(&file_content)?;

    let stats = projects.stats();
    let stats_rows = vec![
        StatsRow::new("Structs", stats.struct_count),
        StatsRow::new("Boundaries", stats.polygon_count),
        StatsRow::new("Paths", stats.path_count),
        StatsRow::new("SRefs", stats.sref_count),
        StatsRow::new("ARefs", stats.aref_count),
        StatsRow::new("Texts", stats.text_count),
        StatsRow::new("Nodes", stats.node_count),
        StatsRow::new("Boxes", stats.box_count),
        StatsRow::new("Layers", (projects.highest_layer() + 1) as usize),
    ];

    for row in stats_rows {
        println!("{:<12} {}", row.name, row.value);
    }

    let mut has_root_cell = false;

    for cell in &projects.library().structs {
        if !projects.is_root_cell(&cell.name) {
            continue;
        }
        has_root_cell = true;
        let poly_count = cell
            .elems
            .iter()
            .filter(|e| matches!(e, gds21::GdsElement::GdsBoundary(_)))
            .count();
        let path_count = cell
            .elems
            .iter()
            .filter(|e| matches!(e, gds21::GdsElement::GdsPath(_)))
            .count();
        let shape_count = poly_count + path_count;
        let sref_count = cell
            .elems
            .iter()
            .filter(|e| matches!(e, gds21::GdsElement::GdsStructRef(_)))
            .count();
        let aref_count = cell
            .elems
            .iter()
            .filter(|e| matches!(e, gds21::GdsElement::GdsArrayRef(_)))
            .count();
        let children_count = sref_count + aref_count;
        let mut output = format!(
            "{} :: {} shapes",
            cell.name.color(Color::BrightYellow),
            shape_count.to_string().color(Color::BrightWhite),
        );
        match children_count {
            0 => (),
            1 => output.push_str(", contains 1 child"),
            n => output.push_str(&format!(", contains {} children", n)),
        }
        println!("{}", output);
    }

    if !has_root_cell {
        println!("{}", "No root cell found".color(Color::Red));
    }

    // Generate and save SVG only if output path is provided
    if let Some(output_path) = output_path {
        let svg_content = projects.to_svg().map_err(|e| {
            anyhow!(
                "Failed to generate SVG: {}",
                e.as_string().unwrap_or_default()
            )
        })?;

        fs::write(output_path, svg_content)?;
        println!("\nSVG file written to: {}", output_path.display());
    }

    Ok(())
}
