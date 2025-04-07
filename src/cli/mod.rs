pub mod app_window;
pub mod generate_svg;

use crate::cli::app_window::spawn_window;
use crate::cli::generate_svg::generate_svg;
use crate::core::instancer::Instancer;
use crate::core::loader::load_gds_into_world;
use crate::core::root_finder::RootFinder;

use anyhow::anyhow;
use anyhow::Result;
use bevy_ecs::world::World;
use clap::Parser;
use futures::StreamExt;
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

    let mut world = rt.block_on(async {
        let gds_data = file_content.clone();
        let progress_stream = load_gds_into_world(&gds_data, World::new()).await;
        let mut progress_stream = std::pin::pin!(progress_stream);
        let mut world = None;
        while let Some(mut progress) = progress_stream.next().await {
            print!(".");
            world = progress.world.take();
        }
        log::info!("Done with loading.");

        let mut root_finder = RootFinder::new(world.as_mut().unwrap());
        let roots = root_finder.find_roots(world.as_ref().unwrap());

        log::info!("Found {} roots.", roots.len());

        let mut instancer = Instancer::new(world.as_mut().unwrap());
        instancer.select_root(world.as_mut().unwrap(), roots[0]);

        log::info!("Done with instantiation.");

        world.unwrap()
    });

    // Generate and save SVG if output path is provided
    if let Some(ref output_path) = args.output {
        let svg_content = generate_svg(&mut world);

        fs::write(output_path, svg_content)?;
        println!("SVG file written to: {}", output_path.display());
    }

    println!();

    if args.gl {
        spawn_window(world)?;
    }

    Ok(())
}
