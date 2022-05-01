#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod open;
mod boxes;

use boxes::BoxesWindow;
use eframe::{egui::{self}, emath::Vec2};

fn main() {
    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        initial_window_size: Some(Vec2{x: 1280.0, y: 720.0}),
        ..Default::default()
    };
    eframe::run_native(
        "GGST Collision Editor Rust (alpha)",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

#[derive(Default)]
struct MyApp {
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Option<String>,
    success: bool,
    boxes_window: BoxesWindow,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    self.nested_menus(ui)
                });
            });    

            ui.label("Open from the File menu, or drag-and-drop the file here.");

            if let Some(picked_path) = &self.picked_path {
                if self.success == false{
                    ui.horizontal(|ui| {
                        ui.label("Failed to open file!
Make sure that your file is a valid Team Red format PAC.
PACs rebuilt using the GeoArcSys tools are not compatible!
                        ");
                    });        
                }
                else {
                    ui.horizontal(|ui| {
                        ui.label("Picked file:");
                        ui.monospace(picked_path);
                    });
                    self.boxes_window.ui(ui);
                }
            }

            // Show dropped files (if any):
            if !self.dropped_files.is_empty() {
                for file in &self.dropped_files {
                    let &path = &file.path.as_ref().unwrap();
                    self.success = self.boxes_window.open_file(path);
                    self.picked_path = Some(path.display().to_string());
                }
                self.dropped_files.clear();
            }
        });

        self.detect_files_being_dropped(ctx);
    }
}

impl MyApp {
    fn detect_files_being_dropped(&mut self, ctx: &egui::Context) {
        use egui::*;

        // Preview hovering files:
        if !ctx.input().raw.hovered_files.is_empty() {
            let mut text = "Dropping files:\n".to_owned();
            for file in &ctx.input().raw.hovered_files {
                if let Some(path) = &file.path {
                    text += &format!("\n{}", path.display());
                } else if !file.mime.is_empty() {
                    text += &format!("\n{}", file.mime);
                } else {
                    text += "\n???";
                }
            }

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let screen_rect = ctx.input().screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            self.dropped_files = ctx.input().raw.dropped_files.clone();
        }
    }
    fn nested_menus(&mut self, ui: &mut egui::Ui) {
        if ui.button("Open").clicked() {
            if let Some(path) = rfd::FileDialog::new()
            .add_filter("PAC File", &["pac"])
            .pick_file() {
                self.success = self.boxes_window.open_file(&path);
                self.picked_path = Some(path.display().to_string());
            };
            ui.close_menu();
        }
        if ui.button("Save").clicked() {
            if !self.boxes_window.jonbins.is_empty() {
                if let Some(path) = rfd::FileDialog::new()
                .add_filter("PAC File", &["pac"])
                .save_file() {
                    self.boxes_window.write_pac(&path).expect("Failed to save file!");
                    self.picked_path = Some(path.display().to_string());
                };
                ui.close_menu();    
            }
        }
        ui.menu_button("Modify boxes", |ui| {
            if ui.button("Add hurtbox").clicked() {
                self.boxes_window.add_hurtbox();
            }
            if ui.button("Add hitbox").clicked() {
                self.boxes_window.add_hitbox();
            }            
            if ui.button("Remove hurtbox").clicked() {
                self.boxes_window.remove_hurtbox();
            }
            if ui.button("Remove hitbox").clicked() {
                self.boxes_window.remove_hitbox();
            }
        });
    }
}
