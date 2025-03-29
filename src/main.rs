use clap::Parser;

mod maze;
use maze::{Exit, Maze};

#[derive(clap::Parser, Debug)]
#[command(name = "maze", version = "0.1.0", about = "Generate and solve mazes")]
struct Cli {
    #[arg(short, long, default_value_t = 60, help = "Width of the maze")]
    width: usize,
    #[arg(short, long, default_value_t = 30, help = "Height of the maze")]
    height: usize,
    #[arg(short, long, default_value_t = 3, help = "Size if the central room")]
    room_size: usize,
    #[arg(short, long, help = "Percentage of the maze to fill with artifacts")]
    artifacts_percentage: Option<f32>,
    #[arg(short, long, help = "Output maze to DOT file for GraphViz")]
    dot_file: Option<String>,
    #[arg(short, long, help = "Output maze to SVG file")]
    svg_file: Option<String>,
    #[arg(long, default_value_t = 10.0)]
    scale: f32,
    #[arg(
        long,
        default_value_t = true,
        help = "Show solution path in SVG output"
    )]
    with_solution: bool,
    #[arg(short, long, default_value_t = false, help = "Enable verbose output")]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut maze = Maze::new(cli.width, cli.height, cli.room_size, Some(Exit::Right));
    if let Some(artifacts_percentage) = cli.artifacts_percentage {
        maze.place_artifacts(artifacts_percentage);
    }
    if let Some(dot_file) = cli.dot_file {
        maze.export_to_dot(&dot_file)?;
    }
    if let Some(svg_file) = cli.svg_file {
        maze.export_to_svg(&svg_file, cli.scale, cli.with_solution)?;
    }
    Ok(())
}
