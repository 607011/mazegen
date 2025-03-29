use rand::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::sync::LazyLock;

#[allow(dead_code)]
pub enum Exit {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    x: usize,
    y: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellType {
    Wall,
    Path,
    Marshmallows,
    GummyBears,
    Cookies,
    Candy,
    Chocolate,
    Zombie,
    Ghost,
    Witch,
    Fog,
    Shadows,
    Crow,
    BlackCat,
    Skeleton,
    Spider,
    Bat,
    Pumpkin,
}

impl CellType {
    #[allow(dead_code)]
    pub fn to_string(self) -> &'static str {
        match self {
            CellType::Wall => "Wall",
            CellType::Path => "Path",
            CellType::Marshmallows => "Marshmallows",
            CellType::GummyBears => "Gummy Bears",
            CellType::Cookies => "Cookies",
            CellType::Candy => "Candy",
            CellType::Chocolate => "Chocolate",
            CellType::Zombie => "Zombie",
            CellType::Ghost => "Ghost",
            CellType::Witch => "Witch",
            CellType::Fog => "Fog",
            CellType::Shadows => "Shadows",
            CellType::Crow => "Crow",
            CellType::BlackCat => "Black Cat",
            CellType::Skeleton => "Skeleton",
            CellType::Spider => "Spider",
            CellType::Bat => "Bat",
            CellType::Pumpkin => "Pumpkin",
        }
    }
    pub fn weight(&self) -> i32 {
        match self {
            CellType::Wall => 0,
            CellType::Path => 0,
            CellType::Marshmallows => -2,
            CellType::GummyBears => -3,
            CellType::Cookies => -4,
            CellType::Candy => -2,
            CellType::Chocolate => -6,
            CellType::Zombie => 7,
            CellType::Ghost => 6,
            CellType::Witch => 9,
            CellType::Fog => 3,
            CellType::Shadows => 4,
            CellType::Crow => 5,
            CellType::BlackCat => 2,
            CellType::Skeleton => 5,
            CellType::Spider => 3,
            CellType::Bat => 1,
            CellType::Pumpkin => 2,
        }
    }
}

static REWARDS: LazyLock<Vec<CellType>> = LazyLock::new(|| {
    vec![
        CellType::Marshmallows,
        CellType::GummyBears,
        CellType::Cookies,
        CellType::Candy,
        CellType::Chocolate,
    ]
});

static DANGERS: LazyLock<Vec<CellType>> = LazyLock::new(|| {
    vec![
        CellType::Zombie,
        CellType::Ghost,
        CellType::Witch,
        CellType::Fog,
        CellType::Shadows,
        CellType::Crow,
        CellType::BlackCat,
        CellType::Skeleton,
        CellType::Spider,
        CellType::Bat,
        CellType::Pumpkin,
    ]
});

static TRAVERSABLE: LazyLock<HashSet<CellType>> = LazyLock::new(|| {
    [
        CellType::Path,
        CellType::Marshmallows,
        CellType::GummyBears,
        CellType::Cookies,
        CellType::Candy,
        CellType::Chocolate,
        CellType::Zombie,
        CellType::Ghost,
        CellType::Witch,
        CellType::Fog,
        CellType::Shadows,
        CellType::Crow,
        CellType::BlackCat,
        CellType::Skeleton,
        CellType::Spider,
        CellType::Bat,
        CellType::Pumpkin,
    ]
    .into_iter()
    .collect()
});

#[derive(Clone)]
pub struct Maze {
    width: usize,
    height: usize,
    room_size: usize,
    cells: Vec<CellType>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    start_id: usize,
    end_id: usize,
    weight: i32,
}

type Edges = HashSet<Edge>;
type Nodes = HashMap<Pos, usize>; // (position, node_id)

impl Maze {
    pub fn new(width: usize, height: usize, room_size: usize, exit_type: Option<Exit>) -> Self {
        // Ensure dimensions are odd to have proper walls
        let width = if width % 2 == 0 { width + 1 } else { width };
        let height = if height % 2 == 0 { height + 1 } else { height };

        // Initialize all cells as walls
        let mut maze = Maze {
            width,
            height,
            cells: vec![CellType::Wall; width * height],
            room_size,
        };

        // Create center room
        let center_x = width / 2;
        let center_y = height / 2;

        for y in (center_y - room_size / 2)..=(center_y + room_size / 2) {
            for x in (center_x - room_size / 2)..=(center_x + room_size / 2) {
                if x < width && y < height {
                    maze.set(x, y, CellType::Path);
                }
            }
        }

        // Generate maze using recursive backtracking
        maze.generate_from(Pos {
            x: center_x,
            y: center_y,
        });

        // Determine exit position based on exit_type
        let exit_pos = match exit_type {
            Some(Exit::Left) => Pos {
                x: 0,
                y: height / 2,
            },
            Some(Exit::Right) => Pos {
                x: width - 1,
                y: height / 2,
            },
            Some(Exit::Top) => Pos { x: width / 2, y: 0 },
            Some(Exit::Bottom) => Pos {
                x: width / 2,
                y: height - 1,
            },
            None => {
                // Random exit if none specified
                let exit_positions = [
                    Pos {
                        x: 0,
                        y: height / 2,
                    }, // Left
                    Pos {
                        x: width - 1,
                        y: height / 2,
                    }, // Right
                    Pos { x: width / 2, y: 0 }, // Top
                    Pos {
                        x: width / 2,
                        y: height - 1,
                    }, // Bottom
                ];
                exit_positions[rand::rng().random_range(0..4)]
            }
        };

        maze.set(exit_pos.x, exit_pos.y, CellType::Path);

        // Connect exit to maze
        let direction = match (exit_pos.x, exit_pos.y) {
            (0, _) => (1, 0),                    // From left wall: go right
            (x, _) if x == width - 1 => (-1, 0), // From right wall: go left
            (_, 0) => (0, 1),                    // From top wall: go down
            _ => (0, -1),                        // From bottom wall: go up
        };

        let mut x = exit_pos.x as isize + direction.0;
        let mut y = exit_pos.y as isize + direction.1;

        // Ensure we make at least one step inward to break through the wall
        if x >= 0 && x < width as isize && y >= 0 && y < height as isize {
            maze.set(x as usize, y as usize, CellType::Path);
            x += direction.0;
            y += direction.1;
        }

        // Now continue until we hit a path
        while x >= 0
            && x < width as isize
            && y >= 0
            && y < height as isize
            && maze.get(x as usize, y as usize) != CellType::Path
        {
            maze.set(x as usize, y as usize, CellType::Path);
            x += direction.0;
            y += direction.1;
        }

        maze
    }

    fn get(&self, x: usize, y: usize) -> CellType {
        self.cells[y * self.width + x]
    }

    fn set(&mut self, x: usize, y: usize, value: CellType) {
        self.cells[y * self.width + x] = value;
    }

    fn generate_from(&mut self, start: Pos) {
        let mut rng = rand::rng();
        let mut stack = vec![start];
        let mut visited = HashSet::new();
        visited.insert(start);

        while let Some(pos) = stack.pop() {
            let directions = [
                (
                    Pos {
                        x: pos.x + 2,
                        y: pos.y,
                    },
                    Pos {
                        x: pos.x + 1,
                        y: pos.y,
                    },
                ), // Right
                (
                    Pos {
                        x: pos.x.saturating_sub(2),
                        y: pos.y,
                    },
                    Pos {
                        x: pos.x.saturating_sub(1),
                        y: pos.y,
                    },
                ), // Left
                (
                    Pos {
                        x: pos.x,
                        y: pos.y + 2,
                    },
                    Pos {
                        x: pos.x,
                        y: pos.y + 1,
                    },
                ), // Down
                (
                    Pos {
                        x: pos.x,
                        y: pos.y.saturating_sub(2),
                    },
                    Pos {
                        x: pos.x,
                        y: pos.y.saturating_sub(1),
                    },
                ), // Up
            ];

            let valid_directions = directions
                .iter()
                .filter(|(next, _)| {
                    next.x > 0
                        && next.x < self.width - 1
                        && next.y > 0
                        && next.y < self.height - 1
                        && !visited.contains(next)
                })
                .collect::<Vec<_>>();

            if !valid_directions.is_empty() {
                stack.push(pos);

                let (next, wall) = valid_directions.choose(&mut rng).unwrap();

                // Carve a path through the wall
                self.set(wall.x, wall.y, CellType::Path);
                self.set(next.x, next.y, CellType::Path);

                visited.insert(*next);
                stack.push(*next);
            }
        }
    }

    pub fn place_artifacts(&mut self, fill_percentage: f32) {
        let mut rng = rand::rng();

        // Calculate how many cells should have artifacts
        let path_cells = self.cells.iter().filter(|&&c| c == CellType::Path).count();
        let artifacts_count = (path_cells as f32 * fill_percentage) as usize;

        let center_x = self.width / 2;
        let center_y = self.height / 2;

        // Collect all valid positions
        let mut valid_positions: Vec<Pos> = (0..self.height)
            .flat_map(|y| (0..self.width).map(move |x| Pos { x, y }))
            .filter(|pos| {
                let in_center_room = pos.x >= center_x - self.room_size / 2
                    && pos.x <= center_x + self.room_size / 2
                    && pos.y >= center_y - self.room_size / 2
                    && pos.y <= center_y + self.room_size / 2;

                self.get(pos.x, pos.y) == CellType::Path && !in_center_room
            })
            .collect();

        // Shuffle positions
        valid_positions.shuffle(&mut rng);

        // Place artifacts
        let reward_ratio = 0.4; // 40% rewards, 60% dangers
        let reward_count = (artifacts_count as f32 * reward_ratio) as usize;
        let danger_count = artifacts_count - reward_count;

        // Place rewards
        for i in 0..reward_count {
            if i < valid_positions.len() {
                let pos = valid_positions[i];
                let reward = REWARDS[rng.random_range(0..REWARDS.len())];
                self.set(pos.x, pos.y, reward);
            }
        }

        // Place dangers
        for i in reward_count..reward_count + danger_count {
            if i < valid_positions.len() {
                let pos = valid_positions[i];
                let danger = DANGERS[rng.random_range(0..DANGERS.len())];
                self.set(pos.x, pos.y, danger);
            }
        }
    }

    pub fn solve(&mut self) -> Option<Vec<Pos>> {
        let center_x = self.width / 2;
        let center_y = self.height / 2;
        let start = Pos {
            x: center_x,
            y: center_y,
        };

        let mut visited = HashSet::new();
        let mut queue = Vec::new();

        queue.push((start, vec![start]));
        visited.insert(start);

        // For the center room, add all edge cells that lead outside the room
        // Calculate the boundaries of the center room
        let room_min_x = center_x - self.room_size / 2;
        let room_max_x = center_x + self.room_size / 2;
        let room_min_y = center_y - self.room_size / 2;
        let room_max_y = center_y + self.room_size / 2;

        // Check all cells at the edge of the room
        for y in room_min_y..=room_max_y {
            for x in room_min_x..=room_max_x {
                if x == room_min_x || x == room_max_x || y == room_min_y || y == room_max_y {
                    // This is an edge cell of the room
                    let pos = Pos { x, y };

                    // Check if there's a path leading out from this edge
                    let directions = [
                        (x + 1, y),
                        (x.saturating_sub(1), y),
                        (x, y + 1),
                        (x, y.saturating_sub(1)),
                    ];

                    for (nx, ny) in directions {
                        if nx < self.width
                            && ny < self.height
                            && TRAVERSABLE.contains(&self.get(nx, ny))
                            && !(nx >= room_min_x
                                && nx <= room_max_x
                                && ny >= room_min_y
                                && ny <= room_max_y)
                        {
                            // This edge cell has a path leading outside the room
                            let path = vec![pos];
                            queue.insert(0, (pos, path));
                            visited.insert(pos);
                            break;
                        }
                    }
                }
            }
        }
        while let Some((pos, path)) = queue.pop() {
            // Check if we've reached an exit
            if pos.x == 0 || pos.x == self.width - 1 || pos.y == 0 || pos.y == self.height - 1 {
                return Some(path);
            }

            // Explore neighbors
            let directions = [
                Pos {
                    x: pos.x + 1,
                    y: pos.y,
                }, // Right
                Pos {
                    x: pos.x.saturating_sub(1),
                    y: pos.y,
                }, // Left
                Pos {
                    x: pos.x,
                    y: pos.y + 1,
                }, // Down
                Pos {
                    x: pos.x,
                    y: pos.y.saturating_sub(1),
                }, // Up
            ];

            for next in directions.iter() {
                if next.x < self.width && next.y < self.height && !visited.contains(next) {
                    let cell_type = self.get(next.x, next.y);
                    if TRAVERSABLE.contains(&cell_type) {
                        let mut new_path = path.clone();
                        new_path.push(*next);
                        queue.insert(0, (*next, new_path));
                        visited.insert(*next);
                    }
                }
            }
        }

        None // No solution found
    }

    pub fn export_to_svg(
        &self,
        filename: &str,
        scale: f32,
        with_solution: bool,
    ) -> std::io::Result<()> {
        let mut maze = self.clone();
        let mut file = File::create(filename)?;

        // Write SVG header with scaled dimensions
        writeln!(
            file,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",
            maze.width as f32 * scale,
            maze.height as f32 * scale,
            maze.width as f32 * scale,
            maze.height as f32 * scale
        )?;

        writeln!(
            file,
            "<rect width=\"100%\" height=\"100%\" fill=\"#eee\" />"
        )?;
        writeln!(file, "  <g transform=\"scale({})\" fill=\"#eee\" >", scale)?;

        if with_solution {
            if let Some(solution) = maze.solve() {
                writeln!(
                    file,
                    "    <polyline fill=\"none\" stroke=\"rgb(28, 163, 163)\" stroke-width=\"0.35\" points=\"",
                )?;
                for pos in solution {
                    write!(file, "{},{} ", (pos.x as f32 + 0.5), (pos.y as f32 + 0.5))?;
                }
                writeln!(file, "\" />")?;
            }
        }

        // Draw the maze
        for y in 0..maze.height {
            for x in 0..maze.width {
                match maze.get(x, y) {
                    CellType::Zombie
                    | CellType::Ghost
                    | CellType::Witch
                    | CellType::Fog
                    | CellType::Shadows
                    | CellType::Crow
                    | CellType::BlackCat
                    | CellType::Skeleton
                    | CellType::Spider
                    | CellType::Bat
                    | CellType::Pumpkin => {
                        writeln!(
                            file,
                            "    <circle cx=\"{}\" cy=\"{}\" r=\"0.4\" fill=\"#e43\" title=\"{}\" />",
                            x as f32 + 0.5,
                            y as f32 + 0.5,
                            maze.get(x, y).to_string()
                        )?;
                    }
                    CellType::Marshmallows
                    | CellType::GummyBears
                    | CellType::Cookies
                    | CellType::Candy
                    | CellType::Chocolate => {
                        writeln!(
                            file,
                            "    <circle cx=\"{}\" cy=\"{}\" r=\"0.4\" fill=\"#2d1\" title=\"{}\" />",
                            x as f32 + 0.5,
                            y as f32 + 0.5,
                            maze.get(x, y).to_string()
                        )?;
                    }
                    CellType::Wall => {
                        writeln!(
                            file,
                            "    <rect x=\"{}\" y=\"{}\" width=\"1\" height=\"1\" fill=\"#222\" />",
                            x, y
                        )?;
                    }
                    _ => {}
                }
            }
        }

        writeln!(file, "  </g>")?;
        writeln!(file, "</svg>")?;
        Ok(())
    }

    pub fn build_graph(&self) -> (Nodes, Edges) {
        let mut nodes: Nodes = HashMap::new();
        let mut edges: Edges = HashSet::new();
        let mut node_id = 0;

        // Special nodes: center (start) and exit
        let center_x: usize = self.width / 2;
        let center_y: usize = self.height / 2;
        let center_pos: Pos = Pos {
            x: center_x,
            y: center_y,
        };
        nodes.insert(center_pos, node_id);
        node_id += 1;

        // Find exit node
        let mut exit_pos: Option<Pos> = None;
        for x in [0, self.width - 1].iter() {
            for y in 0..self.height {
                if self.get(*x, y) == CellType::Path {
                    exit_pos = Some(Pos { x: *x, y });
                    break;
                }
            }
        }
        if exit_pos.is_none() {
            for y in [0, self.height - 1].iter() {
                for x in 0..self.width {
                    if self.get(x, *y) == CellType::Path {
                        exit_pos = Some(Pos { x, y: *y });
                        break;
                    }
                }
            }
        }

        if let Some(pos) = exit_pos {
            nodes.insert(pos, node_id);
            node_id += 1;
        }

        // Scan the maze to find all intersections and dead ends
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let cell_type = self.get(x, y);
                // Check if the cell is a path, reward or danger (traversable)
                if cell_type == CellType::Path
                    || REWARDS.contains(&cell_type)
                    || DANGERS.contains(&cell_type)
                {
                    let current_pos = Pos { x, y };
                    let neighbors = [
                        Pos { x: x + 1, y },
                        Pos { x: x - 1, y },
                        Pos { x, y: y + 1 },
                        Pos { x, y: y - 1 },
                    ]
                    .iter()
                    .filter(|pos| {
                        let pos_type = self.get(pos.x, pos.y);
                        pos_type == CellType::Path
                            || REWARDS.contains(&pos_type)
                            || DANGERS.contains(&pos_type)
                    })
                    .count();

                    // Create a node if this is an intersection (>2 neighbors) or dead end (1 neighbor)
                    if neighbors != 2 && current_pos != center_pos && Some(current_pos) != exit_pos
                    {
                        nodes.insert(current_pos, node_id);
                        node_id += 1;
                    }
                }
            }
        }

        // Create edges between nodes by following paths
        for (&start_pos, &start_id) in &nodes {
            // For each direction, follow the path until another node is found
            let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];

            for &(dx, dy) in &directions {
                let mut x = start_pos.x as isize + dx;
                let mut y = start_pos.y as isize + dy;

                if x < 0 || x >= self.width as isize || y < 0 || y >= self.height as isize {
                    continue;
                }

                let cell_type = self.get(x as usize, y as usize);
                if cell_type == CellType::Wall {
                    continue;
                }

                let mut weight = cell_type.weight(); // Start with the weight of the first cell
                let mut visited = HashSet::new();
                visited.insert(start_pos);

                // Follow the path
                while x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
                    let current_pos = Pos {
                        x: x as usize,
                        y: y as usize,
                    };

                    // If we've found another node, create an edge
                    if let Some(&end_id) = nodes.get(&current_pos) {
                        if start_id < end_id {
                            // Only add each edge once
                            edges.insert(Edge {
                                start_id,
                                end_id,
                                weight,
                            });
                        }
                        break;
                    }

                    // If not a node, check neighboring cells to continue the path
                    visited.insert(current_pos);

                    let mut next_found = false;
                    for &(ndx, ndy) in &directions {
                        let nx = x + ndx;
                        let ny = y + ndy;

                        if nx >= 0
                            && nx < self.width as isize
                            && ny >= 0
                            && ny < self.height as isize
                        {
                            let next_pos = Pos {
                                x: nx as usize,
                                y: ny as usize,
                            };
                            let next_cell_type = self.get(next_pos.x, next_pos.y);

                            if next_cell_type != CellType::Wall && !visited.contains(&next_pos) {
                                x = nx;
                                y = ny;
                                weight += next_cell_type.weight();
                                next_found = true;
                                break;
                            }
                        }
                    }

                    if !next_found {
                        break;
                    }
                }
            }
        }

        (nodes, edges)
    }

    pub fn export_to_dot(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        let (nodes, edges) = self.build_graph();

        // Write DOT file header
        writeln!(file, "graph Maze {{")?;
        writeln!(file, "    node [shape=point];")?;
        writeln!(file, "    edge [len=1.0];")?;

        // Write nodes
        let center_pos = Pos {
            x: self.width / 2,
            y: self.height / 2,
        };

        // Find the exit pos
        let mut exit_pos = None;
        for x in [0, self.width - 1].iter() {
            for y in 0..self.height {
                if self.get(*x, y) == CellType::Path {
                    exit_pos = Some(Pos { x: *x, y });
                    break;
                }
            }
        }
        if exit_pos.is_none() {
            for y in [0, self.height - 1].iter() {
                for x in 0..self.width {
                    if self.get(x, *y) == CellType::Path {
                        exit_pos = Some(Pos { x, y: *y });
                        break;
                    }
                }
            }
        }

        for (&pos, &node_id) in &nodes {
            if pos == center_pos {
                writeln!(
                    file,
                    "    n{} [color=green, shape=circle, label=\"Start\"];",
                    node_id
                )?;
            } else if Some(pos) == exit_pos {
                writeln!(
                    file,
                    "    n{} [color=red, shape=box, label=\"Exit\"];",
                    node_id
                )?;
            } else {
                // Determine if node is a dead end or junction
                let neighbors = [
                    Pos {
                        x: pos.x + 1,
                        y: pos.y,
                    },
                    Pos {
                        x: pos.x.saturating_sub(1),
                        y: pos.y,
                    },
                    Pos {
                        x: pos.x,
                        y: pos.y + 1,
                    },
                    Pos {
                        x: pos.x,
                        y: pos.y.saturating_sub(1),
                    },
                ]
                .iter()
                .filter(|p| self.get(p.x, p.y) == CellType::Path)
                .count();

                let label = if neighbors == 1 {
                    "Dead End"
                } else {
                    "Junction"
                };
                writeln!(file, "    n{} [label=\"{}\"];", node_id, label)?;
            }
        }

        // Write edges
        for &edge in &edges {
            writeln!(
                file,
                "    n{} -- n{} [len={:.1}, label=\"{}\"];",
                edge.start_id, edge.end_id, edge.weight, edge.weight
            )?;
        }

        writeln!(file, "}}")?;
        Ok(())
    }
}
