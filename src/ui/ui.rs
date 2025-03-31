#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::Storage;
use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use mazegen::{DANGERS, ExitLocation, Maze, MazeError, REWARDS, SolutionType, TRAVERSABLE};
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
static APP_NAME: &str = "Maze";

#[derive(Debug, Serialize, Deserialize)]
struct AppSettings {
    scale: f32,
    room_size: usize,
    exit_type: ExitLocation,
    with_path: SolutionType,
    show_artifacts: bool,
    width: usize,
    height: usize,
    wall_color: Color32,
    pathway_color: Color32,
    solution_stroke: Stroke,
    reward_color: Color32,
    danger_color: Color32,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            scale: 10.0,
            room_size: 3,
            exit_type: ExitLocation::Right,
            with_path: SolutionType::None,
            show_artifacts: true,
            width: 61,
            height: 31,
            wall_color: Color32::from_rgb(35, 35, 40),
            pathway_color: Color32::from_rgb(220, 220, 230),
            solution_stroke: Stroke::new(5.0, Color32::from_rgb(28, 163, 163)),
            reward_color: Color32::from_hex("#22dd11").unwrap(),
            danger_color: Color32::from_hex("#ee4433").unwrap(),
        }
    }
}

struct MazeApp {
    maze: Maze,
    settings: AppSettings,
}

impl Default for MazeApp {
    fn default() -> Self {
        MazeApp::new()
    }
}

impl MazeApp {
    #[cfg(not(target_arch = "wasm32"))]
    fn new() -> Self {
        MazeApp {
            maze: Maze::new(61, 31, 3, ExitLocation::Right),
            settings: AppSettings::default(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let total_width = self.settings.width as f32 * self.settings.scale;
        let total_height = self.settings.height as f32 * self.settings.scale;

        let (response, painter) =
            ui.allocate_painter(Vec2::new(total_width, total_height), egui::Sense::hover());
        let origin = response.rect.min;

        // Draw the walls
        for y in 0..self.settings.height {
            for x in 0..self.settings.width {
                let cell_x = origin.x + x as f32 * self.settings.scale;
                let cell_y = origin.y + y as f32 * self.settings.scale;

                // Draw walls
                let cell = self.maze.get(x, y);
                if TRAVERSABLE.contains(&cell) {
                    // Draw white square for path
                    painter.rect_filled(
                        Rect::from_min_size(
                            Pos2::new(cell_x, cell_y),
                            Vec2::new(self.settings.scale, self.settings.scale),
                        ),
                        0.0,
                        self.settings.pathway_color,
                    );
                } else {
                    // Draw black square for wall
                    painter.rect_filled(
                        Rect::from_min_size(
                            Pos2::new(cell_x, cell_y),
                            Vec2::new(self.settings.scale, self.settings.scale),
                        ),
                        0.0,
                        self.settings.wall_color,
                    );
                }

                // Draw rewards and dangers if enabled
                if self.settings.show_artifacts {
                    if REWARDS.contains(&self.maze.get(x, y)) {
                        let center = Pos2::new(
                            cell_x + self.settings.scale / 2.0,
                            cell_y + self.settings.scale / 2.0,
                        );
                        painter.circle(
                            center,
                            self.settings.scale * 0.3,
                            self.settings.reward_color,
                            Stroke::NONE,
                        );
                    } else if DANGERS.contains(&self.maze.get(x, y)) {
                        let center = Pos2::new(
                            cell_x + self.settings.scale / 2.0,
                            cell_y + self.settings.scale / 2.0,
                        );
                        painter.circle(
                            center,
                            self.settings.scale * 0.3,
                            self.settings.danger_color,
                            Stroke::NONE,
                        );
                    }
                }
            }
        }

        match self.settings.with_path {
            SolutionType::ShortestPath => {
                if let Some(path) = self.maze.shortest_path() {
                    let mut points = Vec::with_capacity(path.len());
                    // Convert all path positions to screen positions
                    for pos in path {
                        points.push(Pos2::new(
                            origin.x + (pos.x as f32 + 0.5) * self.settings.scale,
                            origin.y + (pos.y as f32 + 0.5) * self.settings.scale,
                        ));
                    }

                    painter.add(egui::Shape::line(points, self.settings.solution_stroke));
                }
            }
            SolutionType::MinimumSpanningTree => {}
            _ => {}
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load(&mut self, storage: &dyn Storage) -> Result<(), MazeError> {
        if let Some(path) = eframe::storage_dir(APP_NAME) {
            log::info!("Trying to load settings from {}", path.display());
        }
        if let Some(settings) = eframe::get_value::<AppSettings>(storage, eframe::APP_KEY) {
            log::info!("Loaded settings from storage: {:?}", settings);
            self.settings = settings;
        }
        Ok(())
    }
}

impl eframe::App for MazeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Left panel with controls
        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add(
                    egui::Slider::new(&mut self.settings.width, 7..=999)
                        .step_by(4.0)
                        .text("Width"),
                );
                ui.add(
                    egui::Slider::new(&mut self.settings.height, 7..=999)
                        .step_by(4.0)
                        .text("Height"),
                );

                // Only rebuild maze if dimensions have changed
                if self.settings.width != self.maze.get_size().0
                    || self.settings.height != self.maze.get_size().1
                {
                    self.maze = Maze::new(
                        self.settings.width,
                        self.settings.height,
                        self.settings.room_size,
                        self.settings.exit_type.clone(),
                    );
                }

                if ui.button("Generate New Maze").clicked() {
                    self.maze = Maze::new(
                        self.settings.width,
                        self.settings.height,
                        self.settings.room_size,
                        self.settings.exit_type.clone(),
                    );
                    self.maze.generate();
                    self.maze.place_artifacts(0.1);
                }

                ui.checkbox(&mut self.settings.show_artifacts, "Show Artifacts");

                ui.add(egui::Slider::new(&mut self.settings.scale, 1.0..=20.0).text("Scale"));
                self.settings.solution_stroke.width = self.settings.scale * 0.4;

                egui::ComboBox::from_label("Solution")
                    .selected_text(format!("{:?}", self.settings.with_path))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.settings.with_path,
                            SolutionType::None,
                            "None",
                        );
                        ui.selectable_value(
                            &mut self.settings.with_path,
                            SolutionType::ShortestPath,
                            "Shortest Path",
                        );
                        ui.selectable_value(
                            &mut self.settings.with_path,
                            SolutionType::MinimumSpanningTree,
                            "MST",
                        );
                    });
            });
        });

        // Central panel with the maze
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create scrollable area for the maze
            egui::ScrollArea::both().show(ui, |ui| {
                self.draw(ui);
            });
        });
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        log::info!("Saving settings: {:?}", self.settings);
        eframe::set_value(storage, eframe::APP_KEY, &self.settings);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format_timestamp(None)
        .format_target(false)
        .init();
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            let mut app = MazeApp::default();
            if let Some(storage) = cc.storage {
                app.load(storage)?;
            }
            Ok(Box::new(app))
        }),
    )
}
