use anyhow::{anyhow, Result};
use colored::*;
use layout_viewer::Layout;
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
    let gds_data = fs::read(input_path)?;
    let layout = Layout::process_gds_file(&gds_data)?;

    let stats = layout.stats();
    let stats_rows = vec![
        StatsRow::new("Cells", stats.cell_count),
        StatsRow::new("Polygons", stats.total_polygons),
        StatsRow::new("Paths", stats.total_paths),
        StatsRow::new("SRefs", stats.total_srefs),
        StatsRow::new("ARefs", stats.total_arefs), // not present in our test files
        StatsRow::new("Texts", stats.total_texts), // present in caravel but let's ignore
        StatsRow::new("Nodes", stats.total_nodes), // not present in our test files
        StatsRow::new("Boxes", stats.total_boxes), // not present in our test files
    ];

    for row in stats_rows {
        println!("{:<12} {}", row.name, row.value);
    }

    // Generate and save SVG only if output path is provided
    if let Some(output_path) = output_path {
        let svg_content = layout.to_svg().map_err(|e| {
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
