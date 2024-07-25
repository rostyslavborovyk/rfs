use std::cell::RefCell;
use std::fs;
use std::ops::Deref;
use eframe::egui;
use eframe::egui::Color32;
use eframe::emath::{Align};
use tinyfiledialogs as tfd;
use crate::peer::file::RFSFile;

const ACCENT: Color32 = Color32::from_rgb(200, 255, 200);

#[derive(Default)]
pub struct AppState {
    file_id_selected: RefCell<Option<String>>,
}

#[derive(Default)]
pub struct RFSApp {
    home_dir: String,
    metafiles_dir: String,
    rfs_files: Vec<RFSFile>,
    state: AppState,
}

impl RFSApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut options = Self::default();

        options.home_dir = std::env::var("HOME").unwrap_or_else(|_| "".to_string());
        options.metafiles_dir = options.home_dir.clone() + "/.rfs_metafiles";
        options.rfs_files = fs::read_dir(&options.metafiles_dir).unwrap().into_iter().map(|path| {
            let p = path.unwrap().path().to_str().unwrap().to_owned();
            RFSFile::from_path_sync(&p)
        }).collect();

        if let Err(_) = fs::read_dir(&options.metafiles_dir) {
            if let Err(err) = fs::create_dir(&options.metafiles_dir) {
                println!("Metafiles dir was not found and unable to create it! {err}")
            };
        };

        println!("Loaded rfs files {:#?}", &options.rfs_files);

        Self {
            ..options
        }
    }
}

impl eframe::App for RFSApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_file_panel(ctx);
        self.render_info_panel(ctx);
        render_footer(ctx);
    }
}

impl RFSApp {
    fn render_file_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel").exact_width(350.).resizable(false).show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                for file in self.rfs_files.iter() {
                    self.render_file(ui, file);
                }
                ui.with_layout(egui::Layout::bottom_up(Align::Center), |ui| {
                    // todo add filter parameter to open_file_dialog
                    if ui.add_sized([ui.available_width(), 0.0], egui::Button::new("Add .rfs file")).clicked() {
                        if let Some(path) = tfd::open_file_dialog("Select .rfs file", &self.home_dir, None) {
                            self.add_file(path);
                        }
                    }
                    ui.add_space(5.);
                });
            });
        });
    }

    fn get_selected_file(&self) -> Option<RFSFile> {
        match self.state.file_id_selected.borrow().deref() {
            None => None,
            Some(file_id) => {
                match self.rfs_files.iter().find(|f| f.data.id.eq(file_id)) {
                    None => None,
                    Some(f) => Some(f.clone())
                }
            }
        }
    }

    fn render_info_panel(&mut self, ctx: &egui::Context) {
        // Central Panel with nested layouts
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.get_selected_file() {
                None => {
                    ui.heading("Select a file");
                }
                Some(file) => {
                    self.render_info_panel_field(ui, "id", &file.data.id.clone(), 0.);
                    self.render_info_panel_field(ui, "hash", &file.data.hash.clone(), 0.);
                    self.render_info_panel_field(ui, "piece size", &file.data.piece_size.to_string(), 0.);
                    self.render_info_panel_field(ui, "peers", &file.data.peers.len().to_string(), 0.);
                    for peer in file.data.peers.iter() {
                        self.render_info_panel_field(
                            ui, peer, 0.to_string().as_ref(), 20.,
                        );
                    }
                }
            }
        });
    }


    fn render_info_panel_field(&mut self, ui: &mut egui::Ui, name: &str, value: &str, space: f32) {
        ui.horizontal(|ui| {
            ui.add_space(space);
            ui.label(name);
            ui.with_layout(egui::Layout::right_to_left(Align::TOP), |ui| {
                ui.label(format!("{}", value));
            });
        });
    }

    fn add_file(&mut self, path: String) {
        let path = path.clone();
        let file_name = path.split('/').last().unwrap();
        let destination = self.metafiles_dir.clone() + &"/" + file_name;
        fs::copy(path, &destination).unwrap_or_else(|err| {
            println!("Unable to copy file to metafiles dir {err}");
            0
        });
        self.rfs_files.push(RFSFile::from_path_sync(&destination));
    }

    fn render_file(&self, ui: &mut egui::Ui, file: &RFSFile) {
        ui.horizontal(|ui| {
            ui.add_sized([ui.available_width() * 0.5, 0.0], egui::Label::new(format!("{}", file.data.name)));
            ui.add_sized([ui.available_width() * 0.7, 0.0], egui::Label::new(format!("{}", file.data.length)));
            let btn = ui.add_sized([ui.available_width(), 0.0], {
                let btn = egui::Button::new("⏵");
                if let Some(file_id) = &self.state.file_id_selected.borrow().deref() {
                    if file_id.eq(&file.data.id) {
                        btn.fill(ACCENT)
                    } else {
                        btn
                    }
                } else {
                    btn
                }
            });
            if btn.clicked() {
                self.state.file_id_selected.replace(Some(file.data.id.clone()));
            }
        });
        ui.add_space(1.);
        ui.separator();
    }
}


fn render_footer(ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.);
            ui.add(egui::Label::new("Source code"));
            ui.add(
                egui::Hyperlink::new("https://github.com/rostyslavborovyk/rfs")
            );
            ui.add_space(10.);
        })
    });
}

fn render_header(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("headlines");
    });
    ui.add_space(1.);
    let sep = egui::Separator::default().spacing(20.);
    ui.add(sep);
}