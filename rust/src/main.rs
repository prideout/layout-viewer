use anyhow::{anyhow, Result};
use colored::*;
use layout_viewer::Layout;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct CellId(usize);

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

struct StringInterner {
    strings: Vec<String>,
    indices: HashMap<String, CellId>,
}

impl StringInterner {
    fn new() -> Self {
        Self {
            strings: Vec::new(),
            indices: HashMap::new(),
        }
    }

    fn intern(&mut self, s: String) -> CellId {
        if let Some(&idx) = self.indices.get(&s) {
            idx
        } else {
            let idx = CellId(self.strings.len());
            self.indices.insert(s.clone(), idx);
            self.strings.push(s);
            idx
        }
    }

    #[allow(dead_code)]
    fn get(&self, id: CellId) -> &str {
        &self.strings[id.0]
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
        StatsRow::new("Structs", stats.struct_count),
        StatsRow::new("Polygons", stats.polygon_count),
        StatsRow::new("Paths", stats.path_count),
        StatsRow::new("SRefs", stats.sref_count),
        StatsRow::new("ARefs", stats.aref_count), // not present in our test files
        StatsRow::new("Texts", stats.text_count), // present in caravel but let's ignore
        StatsRow::new("Nodes", stats.node_count), // not present in our test files
        StatsRow::new("Boxes", stats.box_count),  // not present in our test files
    ];

    for row in stats_rows {
        println!("{:<12} {}", row.name, row.value);
    }

    let mut interner = StringInterner::new();

    // Maps from a cell name to the cells that reference it
    let mut hierarchy: HashMap<CellId, Vec<CellId>> = HashMap::new();

    for cell in &layout.library().structs {
        let cell_idx = interner.intern(cell.name.clone());
        for elem in &cell.elems {
            if let gds21::GdsElement::GdsStructRef(sref) = elem {
                let ref_idx = interner.intern(sref.name.clone());
                hierarchy.entry(ref_idx).or_default().push(cell_idx);
            }
            if let gds21::GdsElement::GdsArrayRef(aref) = elem {
                let ref_idx = interner.intern(aref.name.clone());
                hierarchy.entry(ref_idx).or_default().push(cell_idx);
            }
        }
    }

    for cell in &layout.library().structs {
        let cell_idx = interner.intern(cell.name.clone());
        let is_root = !hierarchy.contains_key(&cell_idx);
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
            if is_root {
                cell.name.color(Color::BrightYellow)
            } else {
                cell.name.color(Color::White)
            },
            shape_count.to_string().color(Color::BrightWhite),
        );
        if children_count > 0 {
            output.push_str(&format!(", {} children", children_count));
        }
        if !is_root {
            output.push_str(&format!(", {} parents", hierarchy[&cell_idx].len()));
        }
        println!("{}", output);
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
