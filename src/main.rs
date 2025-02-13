use eframe::egui;
use egui::plot::{Line, Plot, PlotPoints};
use std::str::FromStr;

/// Structure representing a single function definition.
struct FunctionPlot {
    /// The user-defined function expression (e.g., "sin(x)" or "abs(ln(x-1)/ln(x-2))")
    expression: String,
    /// Any error message produced while parsing or evaluating this function.
    error_message: Option<String>,
    /// The computed (x, y) plot points.
    plot_points: Vec<[f64; 2]>,
    /// The color used when plotting this function.
    color: egui::Color32,
}

impl FunctionPlot {
    fn new(expression: &str, color: egui::Color32) -> Self {
        Self {
            expression: expression.to_owned(),
            error_message: None,
            plot_points: Vec::new(),
            color,
        }
    }

    /// Updates the plot points for this function given the x-range and sample count.
    fn update(&mut self, x_min: f64, x_max: f64, num_points: usize) {
        // Try to parse the expression.
        let expr = match meval::Expr::from_str(&self.expression) {
            Ok(e) => e,
            Err(e) => {
                self.error_message = Some(format!("Parse error: {}", e));
                self.plot_points.clear();
                return;
            }
        };

        // Bind the variable "x" so we get a function f(x).
        let func = match expr.bind("x") {
            Ok(f) => f,
            Err(e) => {
                self.error_message = Some(format!("Binding error: {}", e));
                self.plot_points.clear();
                return;
            }
        };

        // Clear any previous error and compute the new points.
        self.error_message = None;
        self.plot_points.clear();

        for i in 0..=num_points {
            let x = x_min + (x_max - x_min) * (i as f64) / (num_points as f64);
            let y = func(x);
            if y.is_finite() {
                self.plot_points.push([x, y]);
            }
        }
    }
}

/// Main application structure.
struct App {
    // Domain inputs for the x-axis.
    x_min_input: String,
    x_max_input: String,
    // Domain inputs for the y-axis.
    y_min_input: String,
    y_max_input: String,
    // A list of functions to plot.
    functions: Vec<FunctionPlot>,
    // Number of sample points per function.
    num_points: usize,
    // Whether the plot updates automatically as you type.
    auto_update: bool,
    // Error messages for the domain settings.
    domain_error: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        // Prepopulate with two sample functions.
        let default_colors = vec![
            egui::Color32::from_rgb(220, 20, 60),   // Crimson
            egui::Color32::from_rgb(65, 105, 225),   // RoyalBlue
        ];
        let functions = vec![
            FunctionPlot::new("abs(ln(x-1)/ln(x-2))", default_colors[0]),
            FunctionPlot::new("sin(x)", default_colors[1]),
        ];

        Self {
            x_min_input: "-10.0".to_owned(),
            x_max_input: "10.0".to_owned(),
            y_min_input: "-10.0".to_owned(),
            y_max_input: "10.0".to_owned(),
            functions,
            num_points: 1000,
            auto_update: true,
            domain_error: None,
        }
    }
}

impl App {
    /// Update all functions (and validate the domain settings).
    fn update_functions(&mut self) {
        // Parse x-domain.
        let x_min: f64 = match self.x_min_input.trim().parse() {
            Ok(val) => val,
            Err(_) => {
                self.domain_error = Some("Invalid x_min value".to_owned());
                return;
            }
        };
        let x_max: f64 = match self.x_max_input.trim().parse() {
            Ok(val) => val,
            Err(_) => {
                self.domain_error = Some("Invalid x_max value".to_owned());
                return;
            }
        };
        if x_min >= x_max {
            self.domain_error = Some("x_min must be less than x_max".to_owned());
            return;
        }

        // Parse y-domain.
        let y_min: f64 = match self.y_min_input.trim().parse() {
            Ok(val) => val,
            Err(_) => {
                self.domain_error = Some("Invalid y_min value".to_owned());
                return;
            }
        };
        let y_max: f64 = match self.y_max_input.trim().parse() {
            Ok(val) => val,
            Err(_) => {
                self.domain_error = Some("Invalid y_max value".to_owned());
                return;
            }
        };
        if y_min >= y_max {
            self.domain_error = Some("y_min must be less than y_max".to_owned());
            return;
        }
        self.domain_error = None;

        // Update each function's plot points.
        for f in &mut self.functions {
            f.update(x_min, x_max, self.num_points);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // When real-time update is enabled, update on every frame.
        if self.auto_update {
            self.update_functions();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // --- Header ---
            ui.heading("Rust Function Plotter - Multiple Functions");

            // --- Domain Settings ---
            ui.group(|ui| {
                ui.label("Domain Settings:");
                ui.horizontal(|ui| {
                    ui.label("x min:");
                    let changed1 = ui.text_edit_singleline(&mut self.x_min_input).changed();
                    ui.label("x max:");
                    let changed2 = ui.text_edit_singleline(&mut self.x_max_input).changed();
                    if (changed1 || changed2) && self.auto_update {
                        self.update_functions();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("y min:");
                    let changed1 = ui.text_edit_singleline(&mut self.y_min_input).changed();
                    ui.label("y max:");
                    let changed2 = ui.text_edit_singleline(&mut self.y_max_input).changed();
                    if (changed1 || changed2) && self.auto_update {
                        self.update_functions();
                    }
                });
                if let Some(ref err) = self.domain_error {
                    ui.colored_label(egui::Color32::RED, err);
                }
            });
            ui.separator();

            // --- Additional Controls ---
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Sample Points:");
                    let changed = ui
                        .add(egui::Slider::new(&mut self.num_points, 100..=5000).text("points"))
                        .changed();
                    if changed && self.auto_update {
                        self.update_functions();
                    }
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.auto_update, "Real-time update");
                    if !self.auto_update {
                        if ui.button("Plot").clicked() {
                            self.update_functions();
                        }
                    }
                });
            });
            ui.separator();

            // --- Functions List ---
            ui.group(|ui| {
                ui.heading("Functions:");
                let mut remove_indices = Vec::new();
                let functions_len = self.functions.len();
                for (i, func) in self.functions.iter_mut().enumerate() {
                    ui.collapsing(format!("Function {}", i + 1), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("f(x) = ");
                            ui.text_edit_singleline(&mut func.expression);
                            if functions_len > 1 {
                                if ui.button("Remove").clicked() {
                                    remove_indices.push(i);
                                }
                            }
                        });
                        if let Some(ref err) = func.error_message {
                            ui.colored_label(egui::Color32::RED, err);
                        }
                    });
                }
                for &i in remove_indices.iter().rev() {
                    self.functions.remove(i);
                }
                if ui.button("Add Function").clicked() {
                    let default_colors = vec![
                        egui::Color32::from_rgb(220, 20, 60),
                        egui::Color32::from_rgb(65, 105, 225),
                        egui::Color32::from_rgb(34, 139, 34),
                        egui::Color32::from_rgb(255, 140, 0),
                        egui::Color32::from_rgb(138, 43, 226),
                    ];
                    let color = default_colors[self.functions.len() % default_colors.len()];
                    self.functions.push(FunctionPlot::new("x", color));
                    if self.auto_update {
                        self.update_functions();
                    }
                }
            });
            ui.separator();

            // --- Plot Area ---
            let x_min = self.x_min_input.trim().parse::<f64>().unwrap_or(-10.0);
            let x_max = self.x_max_input.trim().parse::<f64>().unwrap_or(10.0);
            let y_min = self.y_min_input.trim().parse::<f64>().unwrap_or(-10.0);
            let y_max = self.y_max_input.trim().parse::<f64>().unwrap_or(10.0);
            Plot::new("Function Plot")
                .data_aspect(1.0)
                .include_x(x_min)
                .include_x(x_max)
                .include_y(y_min)
                .include_y(y_max)
                .show(ui, |plot_ui| {
                    for func in &self.functions {
                        if !func.plot_points.is_empty() {
                            let line = Line::new(PlotPoints::from_iter(func.plot_points.iter().copied()))
                                .color(func.color)
                                .width(2.0);
                            plot_ui.line(line);
                        }
                    }
                });
        });

        if self.auto_update {
            ctx.request_repaint();
        }
    }
}

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::vec2(1000.0, 700.0));

    eframe::run_native(
        "Rust Function Plotter - Multiple Functions",
        native_options,
        Box::new(|cc| {
            // --- Custom Styling ---
            let mut style: egui::Style = (*cc.egui_ctx.style()).clone();
            style.visuals.dark_mode = false; // Light mode
            style.visuals.window_fill = egui::Color32::from_rgb(250, 250, 250);
            style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(230, 230, 230);
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(220, 220, 255);
            style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(200, 200, 250);
            cc.egui_ctx.set_style(style);
            Box::new(App::default())
        }),
    );
}
