#![windows_subsystem = "windows"]

use eframe::egui;
use gui_spike_common as common;
use std::path::PathBuf;

struct SpikeApp {
    shows: Vec<(String, String)>,
    selected: Option<usize>,
    progress: f32,
    screenshot: Option<PathBuf>,
    frames: u32,
}

impl SpikeApp {
    fn capture(&mut self, ctx: &egui::Context) {
        self.frames += 1;
        if self.frames != 3 {
            ctx.request_repaint();
            return;
        }
        let Some(path) = self.screenshot.take() else { return };
        let ctx_for_close = ctx.clone();
        let callback = egui::ScreenshotCallback::new(move |image| {
            let [width, height] = image.size;
            common::save_png(&path, width as u32, height as u32, image.as_raw());
            ctx_for_close.send_viewport_cmd(egui::ViewportCommand::Close);
            ctx_for_close.request_repaint();
        });
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(callback));
    }
}

impl eframe::App for SpikeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(common::SHOWS_HEADING);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                for (index, (name, publisher)) in self.shows.iter().enumerate() {
                    let selected = self.selected == Some(index);
                    if ui.selectable_label(selected, format!("{name}  ·  {publisher}")).clicked() {
                        self.selected = Some(index);
                    }
                }
            });
            ui.horizontal(|ui| {
                if ui.button(common::ADD).clicked() {
                    let (name, publisher) = common::ADDED_SHOW;
                    self.shows.push((name.to_string(), publisher.to_string()));
                }
                if ui.add_enabled(self.selected.is_some(), egui::Button::new(common::REMOVE)).clicked() {
                    if let Some(index) = self.selected.take() {
                        self.shows.remove(index);
                    }
                }
            });
            ui.separator();
            ui.heading(common::PROGRESS_HEADING);
            ui.label(common::status(self.progress));
            ui.add(egui::ProgressBar::new(self.progress).show_percentage());
            ui.horizontal(|ui| {
                if ui.button(common::RUN).clicked() {
                    self.progress = common::advance(self.progress);
                }
                if ui.button(common::OPEN_PDF).clicked() {
                    common::open_pdf();
                }
            });
            ui.separator();
            ui.label(common::ACCENTS);
        });
        if self.screenshot.is_some() {
            self.capture(&ctx);
        }
    }
}

fn main() -> eframe::Result {
    let (width, height) = common::WINDOW_SIZE;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([width, height]),
        ..Default::default()
    };
    let app = SpikeApp {
        shows: common::shows(),
        selected: Some(0),
        progress: 0.45,
        screenshot: common::screenshot_arg(),
        frames: 0,
    };
    eframe::run_native(common::WINDOW_TITLE, options, Box::new(|_| Ok(Box::new(app))))
}
