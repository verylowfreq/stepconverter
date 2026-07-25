use std::process::ExitCode;

use crate::{App, OutputFormat, ProgramOptions};

pub fn run(prefill_input: Option<String>) -> ExitCode {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([480.0, 360.0]),
        ..Default::default()
    };

    let gui_app = GuiApp::new(prefill_input);

    let result = eframe::run_native(
        "Simple STEP Converter",
        options,
        Box::new(|_cc| Ok(Box::new(gui_app))),
    );

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(5),
    }
}

struct GuiApp {
    input_filepath: String,
    output_filepath: String,
    format: OutputFormat,
    tolerance: f64,
    allow_overwrite: bool,
    status: Option<Result<String, String>>,
}

impl GuiApp {
    fn new(prefill_input: Option<String>) -> Self {
        Self {
            input_filepath: prefill_input.unwrap_or_default(),
            output_filepath: String::new(),
            format: OutputFormat::Stl,
            tolerance: 0.1,
            allow_overwrite: false,
            status: None,
        }
    }

    fn convert(&mut self) {
        if self.input_filepath.is_empty() {
            self.status = Some(Err(String::from("Input STEP file is not specified.")));
            return;
        }
        if self.output_filepath.is_empty() {
            self.status = Some(Err(String::from("Output file is not specified.")));
            return;
        }

        let options = ProgramOptions {
            input_filepath: Some(self.input_filepath.clone()),
            output_filepath: Some(self.output_filepath.clone()),
            allow_overwrite: self.allow_overwrite,
            tolerance: self.tolerance,
            format: Some(self.format),
        };
        let app = App { options };

        let result = (|| -> Result<String, String> {
            app.check_option()?;
            let mesh = app.load_step()?;
            app.export(&mesh, self.format)?;
            Result::Ok(app.format_result(&mesh))
        })();

        self.status = Some(result);
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Simple STEP Converter");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Input STEP file:");
                ui.text_edit_singleline(&mut self.input_filepath);
                if ui.button("Open...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                            .add_filter("STEP", &["step", "stp"])
                            .pick_file() {
                        self.input_filepath = path.display().to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Output file:");
                ui.text_edit_singleline(&mut self.output_filepath);
                if ui.button("Open...").clicked() {
                    let ext = match self.format {
                        OutputFormat::Stl => "stl",
                        OutputFormat::Glb => "glb",
                    };
                    if let Some(path) = rfd::FileDialog::new()
                            .add_filter(ext, &[ext])
                            .save_file() {
                        self.output_filepath = path.display().to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Output format:");
                ui.radio_value(&mut self.format, OutputFormat::Stl, "STL");
                ui.radio_value(&mut self.format, OutputFormat::Glb, "GLB");
            });

            ui.horizontal(|ui| {
                ui.label("Tolerance:");
                ui.add(egui::DragValue::new(&mut self.tolerance).speed(0.01).range(0.0001..=100.0));
            });

            ui.checkbox(&mut self.allow_overwrite, "Allow overwrite");

            ui.separator();

            if ui.button("Convert").clicked() {
                self.convert();
            }

            ui.separator();

            if let Some(status) = &self.status {
                match status {
                    Ok(message) => {
                        ui.colored_label(egui::Color32::from_rgb(0, 150, 0), "Success");
                        ui.monospace(message);
                    }
                    Err(message) => {
                        ui.colored_label(egui::Color32::from_rgb(200, 0, 0), message);
                    }
                }
            }
        });
    }
}
