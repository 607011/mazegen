use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

fn main() {
    use rand::prelude::*;

    #[allow(dead_code)]
    enum Exit {
        Left,
        Right,
        Top,
        Bottom,
    }

    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    struct Pos {
        x: usize,
        y: usize,
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum CellType {
        Wall,
        Path,
    }

    #[derive(Clone)]
    struct Maze {
        width: usize,
        height: usize,
        room_size: usize,
        cells: Vec<CellType>,
    }

    type Edge = (usize, usize, usize); // (start_node_id, end_node_id, path_length)
    type Edges = HashSet<Edge>;
    type Nodes = HashMap<Pos, usize>; // (position, node_id)

    impl Maze {
        fn new(width: usize, height: usize, room_size: usize, exit_type: Option<Exit>) -> Self {
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

            // // Fix top and bottom walls to ensure uniform thickness
            // for x in 0..width {
            //     maze.set(x, 0, CellType::Wall); // Top wall
            //     maze.set(x, height - 1, CellType::Wall); // Bottom wall
            // }

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

        #[allow(dead_code)]
        fn place_letters(&mut self, fill_percentage: f64) -> HashMap<Pos, char> {
            use rand::prelude::*;
            let mut rng = rand::rng();
            let mut letter_positions = HashMap::new();

            // Create a weighted distribution of letters
            // C and T are four times more common
            let letters = ['S', 'P', 'G', 'W', 'F', 'C', 'Z', 'G', 'T', 'C'];
            let weighted_letters: Vec<char> = letters
                .iter()
                .flat_map(|&letter| {
                    let weight = match letter {
                        'C' | 'T' => 4, // C and T are 4x more likely
                        _ => 1,
                    };
                    std::iter::repeat(letter).take(weight)
                })
                .collect();

            // Count open cells (paths) that are not dead ends or intersections
            let mut valid_cells = Vec::new();
            for y in 1..self.height - 1 {
                for x in 1..self.width - 1 {
                    if self.get(x, y) == CellType::Path {
                        // Count neighboring paths
                        let neighbors = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
                            .iter()
                            .filter(|&&(nx, ny)| self.get(nx, ny) == CellType::Path)
                            .count();

                        // Only include cells that are part of a corridor (exactly 2 neighbors)
                        if neighbors == 2 {
                            valid_cells.push(Pos { x, y });
                        }
                    }
                }
            }

            // Calculate how many letters to place
            let num_cells_to_fill = (valid_cells.len() as f64 * fill_percentage) as usize;

            // Shuffle the valid cells
            valid_cells.shuffle(&mut rng);

            // Place letters in randomly selected cells
            for pos in valid_cells
                .iter()
                .take(num_cells_to_fill.min(valid_cells.len()))
            {
                let letter_idx = rng.random_range(0..weighted_letters.len());
                letter_positions.insert(*pos, weighted_letters[letter_idx]);
            }

            letter_positions
        }

        fn solve(&mut self) -> Option<Vec<Pos>> {
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
                                && self.get(nx, ny) == CellType::Path
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
                    if next.x < self.width
                        && next.y < self.height
                        && self.get(next.x, next.y) == CellType::Path
                        && !visited.contains(next)
                    {
                        let mut new_path = path.clone();
                        new_path.push(*next);
                        queue.insert(0, (*next, new_path));
                        visited.insert(*next);
                    }
                }
            }

            None // No solution found
        }

        fn export_to_svg(
            &self,
            filename: &str,
            scale: f64,
            with_solution: bool,
        ) -> std::io::Result<()> {
            let mut maze = self.clone();
            let mut file = File::create(filename)?;

            // Write SVG header with scaled dimensions
            writeln!(
                file,
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",
                maze.width as f64 * scale,
                maze.height as f64 * scale,
                maze.width as f64 * scale,
                maze.height as f64 * scale
            )?;

            writeln!(
                file,
                "<rect width=\"100%\" height=\"100%\" fill=\"#222\" />"
            )?;
            writeln!(file, "  <g transform=\"scale({})\" fill=\"#eee\" >", scale)?;

            // Draw the maze
            for y in 0..maze.height {
                for x in 0..maze.width {
                    if maze.get(x, y) == CellType::Path {
                        writeln!(
                            file,
                            "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" />",
                            x, y, 1, 1
                        )?;
                    }
                }
            }

            if with_solution {
                if let Some(solution) = maze.solve() {
                    writeln!(
                        file,
                        "    <polyline fill=\"none\" stroke=\"red\" stroke-width=\"0.5\" points=\"",
                    )?;
                    for pos in solution {
                        write!(file, "{},{} ", (pos.x as f64 + 0.5), (pos.y as f64 + 0.5))?;
                    }
                    writeln!(file, "\" />")?;
                }
            }

            writeln!(file, "  </g>")?;
            writeln!(file, "</svg>")?;
            Ok(())
        }

        fn build_graph(&self) -> (Nodes, Edges) {
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
                    if self.get(x, y) == CellType::Path {
                        let current_pos = Pos { x, y };
                        let neighbors = [
                            Pos { x: x + 1, y },
                            Pos { x: x - 1, y },
                            Pos { x, y: y + 1 },
                            Pos { x, y: y - 1 },
                        ]
                        .iter()
                        .filter(|pos| self.get(pos.x, pos.y) == CellType::Path)
                        .count();

                        // Create a node if this is an intersection (>2 neighbors) or dead end (1 neighbor)
                        if neighbors != 2
                            && current_pos != center_pos
                            && Some(current_pos) != exit_pos
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

                    if x < 0
                        || x >= self.width as isize
                        || y < 0
                        || y >= self.height as isize
                        || self.get(x as usize, y as usize) != CellType::Path
                    {
                        continue;
                    }

                    let mut path_length = 1;
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
                                edges.insert((start_id, end_id, path_length));
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
                                if self.get(next_pos.x, next_pos.y) == CellType::Path
                                    && !visited.contains(&next_pos)
                                {
                                    x = nx;
                                    y = ny;
                                    path_length += 1;
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

        fn export_to_dot(&self, filename: &str) -> std::io::Result<()> {
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
            for &(start, end, length) in &edges {
                writeln!(
                    file,
                    "    n{} -- n{} [len={:.1}, label=\"{}\"];",
                    start,
                    end,
                    length as f64 * 0.5,
                    length
                )?;
            }

            writeln!(file, "}}")?;
            Ok(())
        }
    }

    // Main function to generate and display a maze
    let maze_width = 160;
    let maze_height = 90;
    let room_size = 5;
    let maze = Maze::new(maze_width, maze_height, room_size, Some(Exit::Right));
    maze.export_to_dot("maze.dot")
        .expect("Failed to export maze to DOT file");
    maze.export_to_svg("maze.svg", 10.0, true)
        .expect("Failed to export maze to SVG file");
}
