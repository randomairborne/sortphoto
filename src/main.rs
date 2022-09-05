#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use std::{path::PathBuf, str::FromStr};

fn main() {
    let native_options = eframe::NativeOptions {
        follow_system_theme: true,
        ..Default::default()
    };
    eframe::run_native(
        "SortPhoto",
        native_options,
        Box::new(|cc| Box::new(Application::new(cc))),
    );
}

struct Application {
    pub input_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub error: Option<String>,
    pub home: PathBuf,
    pub working: bool,
    pub done: bool,
}

impl Application {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        #[cfg(target_family = "unix")]
        let home_str = std::env::var("HOME").unwrap_or_else(|_| "/".into());
        #[cfg(target_family = "windows")]
        let home_str = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into());
        Self {
            input_path: None,
            error: None,
            output_path: None,
            home: std::path::PathBuf::from_str(&home_str).unwrap(),
            working: false,
            done: false,
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(e) = self.error.clone() {
            egui::Window::new("SortPhoto error")
                .open(&mut true)
                .title_bar(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(e);
                    if ui.button("Ok").clicked() {
                        self.error = None;
                    }
                });
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!self.working);
            ui.vertical_centered(|ui| {
                ui.group(|ui| {
                    if ui.button("Choose current photo location").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory(&self.home)
                            .set_title("Unsorted photo location")
                            .pick_folder()
                        {
                            self.input_path = Some(path);
                        }
                    }
                    if let Some(inpath) = &self.input_path {
                        ui.label(inpath.display().to_string());
                    }
                });

                ui.group(|ui| {
                    if ui.button("Choose photo destination").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory(&self.home)
                            .set_title("Sorted photo location")
                            .pick_folder()
                        {
                            self.output_path = Some(path);
                        }
                    }
                    if let Some(outpath) = &self.output_path {
                        ui.label(outpath.display().to_string());
                    }
                });
                if ui.button("Sort!").clicked() {
                    if let (Some(inpath), Some(outpath)) = (&self.input_path, &self.output_path) {
                        self.working = true;
                        if let Err(e) = sortphoto::sort(inpath, outpath) {
                            self.error = Some(format!("Error sorting photos: {e}"))
                        }
                        self.working = false;
                        self.done = true;
                    } else {
                        self.error = Some(
                            "Oops! You need to select input and output folders before sorting."
                                .to_string(),
                        )
                    }
                }
                if self.working {
                    ui.heading("Working...");
                }
                if let Some(path) = &self.output_path {
                    if self.done && opener::open(path).is_ok() {
                        self.done = false;
                    }
                }
                ui.label(egui::RichText::new("Warning! Clicking multiple times with the same input files will duplicate the output files!").color(egui::Color32::GOLD))
            })
        });
    }
}
