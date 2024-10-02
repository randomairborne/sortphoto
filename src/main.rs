#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::collapsible_else_if)]
use std::{path::PathBuf, str::FromStr};

use eframe::egui;
use sortphoto::SortProgress;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    eframe::run_native(
        "SortPhoto",
        Default::default(),
        Box::new(|cc| Ok(Box::new(Application::new(cc)))),
    )
    .unwrap();
}

struct Application {
    pub input_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub message: Option<String>,
    pub home: PathBuf,
    pub working: bool,
    pub sort_finished: (
        watch::WatchSender<SortProgress>,
        watch::WatchReceiver<SortProgress>,
    ),
}

impl Application {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        #[cfg(target_family = "unix")]
        let home_str = std::env::var("HOME").unwrap_or_else(|_| "/".into());
        #[cfg(target_family = "windows")]
        let home_str = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into());
        Self {
            input_path: None,
            message: None,
            output_path: None,
            home: PathBuf::from_str(&home_str).unwrap(),
            working: false,
            sort_finished: watch::channel(SortProgress::Started),
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(SortProgress::Error(e)) = self.sort_finished.1.get_if_new() {
            self.message = Some(e.to_string());
        }
        if let Some(e) = self.message.clone() {
            egui::Window::new("SortPhoto error")
                .open(&mut true)
                .title_bar(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(e);
                    if ui.button("Ok").clicked() {
                        self.message = None;
                        self.working = false;
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
                if self.working {
                    ui.heading("Working...");
                    ui.add(egui::widgets::ProgressBar::new(
                        self.sort_finished.1.get().completion(),
                    ));
                    if let SortProgress::Done(msg) = self.sort_finished.1.get() {
                        self.message = Some(msg);
                    }
                } else {
                    if ui.button("Sort!").clicked() {
                        if let (Some(inpath), Some(outpath)) = (&self.input_path, &self.output_path)
                        {
                            let inpath = inpath.clone();
                            let outpath = outpath.clone();
                            let tx = self.sort_finished.0.clone();
                            let etx = self.sort_finished.0.clone();
                            std::thread::spawn(move || {
                                if let Err(e) = sortphoto::sort(inpath, outpath, tx) {
                                    etx.send(SortProgress::Error(e));
                                }
                            });
                            self.working = true;
                        } else {
                            self.message = Some(
                                "Oops! You need to select input and output folders before sorting."
                                    .to_string(),
                            )
                        }
                    }
                }
            });
            ctx.request_repaint();
        });
    }
}
