use clap::Parser;

mod maze;
use maze::{Exit, Maze};

#[derive(clap::Parser, Debug)]
#[command(name = "maze", version = "0.1.0", about = "Generates and solves mazes")]
struct Cli {
    #[arg(short, long, default_value_t = 40)]
    width: usize,
    #[arg(short, long, default_value_t = 30)]
    height: usize,
    #[arg(short, long, default_value_t = 3)]
    room_size: usize,
    #[arg(short, long, default_value_t = 0.07)]
    fill_percentage: f32,
    #[arg(long, default_value_t = 10.0)]
    scale: f32,
    #[arg(long)]
    solution: bool,
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();
    let mut maze = Maze::new(cli.width, cli.height, cli.room_size, Some(Exit::Right));
    maze.place_artifacts(cli.fill_percentage);
    maze.export_to_dot("maze.dot")
        .expect("Failed to export maze to DOT file");
    maze.export_to_svg("maze.svg", cli.scale, cli.solution)
        .expect("Failed to export maze to SVG file");
}
