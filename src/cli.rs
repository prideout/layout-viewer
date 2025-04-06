use anyhow::anyhow;
use anyhow::Result;
use bevy_ecs::world::World;
use clap::Parser;
use colored::*;
use futures::StreamExt;
use layout_viewer::generate_svg;
use layout_viewer::load_gds_into_world;
use layout_viewer::Instancer;
use layout_viewer::Project;
use layout_viewer::RootFinder;
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

    //// BEGIN NEW ECS STUFF
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        let gds_data = file_content.clone();
        let progress_stream = load_gds_into_world(&gds_data, World::new()).await;
        let mut progress_stream = std::pin::pin!(progress_stream);
        let mut world = None;
        while let Some(mut progress) = progress_stream.next().await {
            log::info!("{}", progress.phase);
            world = progress.world.take();
        }
        log::info!("Done with loading.");

        let mut root_finder = RootFinder::new(world.as_mut().unwrap());
        let roots = root_finder.find_roots(world.as_ref().unwrap());

        log::info!("Found {} roots.", roots.len());

        let mut instancer = Instancer::new(world.as_mut().unwrap());
        instancer.select_root(world.as_mut().unwrap(), roots[0]);

        log::info!("Done with instantiation.");
    });
    //// END NEW ECS STUFF

    let mut project = Project::from_bytes(&file_content)?;
    project.update_world_transforms();
    project.update_layers();

    let bounds = project.bounds();
    println!(
        "{:<12} ({}, {}) to ({}, {})",
        "Bounds".color(Color::BrightYellow),
        pretty_print_float(bounds.min_x),
        pretty_print_float(bounds.min_y),
        pretty_print_float(bounds.max_x),
        pretty_print_float(bounds.max_y)
    );

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
