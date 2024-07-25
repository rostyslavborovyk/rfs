use std::fs;
use eframe::egui;
use eframe::egui::Button;
use eframe::emath::{Align};
use tinyfiledialogs as tfd;
use crate::peer::file::RFSFile;

pub struct RFSAppOptions {
    
}

#[derive(Default)]
pub struct RFSApp {
    home_dir: String,
    metafiles_dir: String,
    rfs_files: Vec<RFSFile>,
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

        // Central Panel with stretched widgets
        // CentralPanel::default().show(ctx, |ui| {
        //     ui.with_layout(egui::Layout::top_down(Align::Min), |ui| {
        //         ui.heading("Central Panel");
        // 
        //         // Stretch horizontally
        //         ui.horizontal(|ui| {
        //             ui.add_sized([ui.available_width() * 0.5, 0.0], egui::Label::new("Left Half Stretch"));
        //             ui.add_sized([ui.available_width(), 0.0], egui::Label::new("Right Half Stretch"));
        //         });
        // 
        //         ui.separator();
        // 
        //         // Stretch vertically
        //         ui.vertical_centered_justified(|ui| {
        //             ui.add_sized([0.0, ui.available_height() * 0.5], egui::Label::new("Top Half Stretch"));
        //             ui.add_sized([0.0, ui.available_height()], egui::Label::new("Bottom Half Stretch"));
        //         });
        // 
        //         ui.separator();
        // 
        //         // Full stretch in both directions
        //         ui.vertical_centered(|ui| {
        //             ui.add_sized([ui.available_width(), ui.available_height()], egui::Label::new("Full Stretch"));
        //         });
        //     });
        // });
        // Top Panel
        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        //     ui.horizontal_centered(|ui| {
        //         ui.heading("Files");
        //     });
        // });

        render_footer(ctx);
        self.render_file_panel(ctx);

        // Central Panel with nested layouts
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(Align::Min), |ui| {
                ui.label("Central area");
        
                ui.horizontal(|ui| {
                    ui.label("Nested horizontal layout 1");
                    ui.label("Nested horizontal layout 2");
                });
        
                ui.vertical(|ui| {
                    ui.label("Nested vertical layout 1");
                    ui.label("Nested vertical layout 2");
        
                    egui::Grid::new("nested_grid").show(ui, |ui| {
                        ui.label("Grid Row 1, Column 1");
                        ui.label("Grid Row 1, Column 2");
                        ui.end_row();
                        ui.label("Grid Row 2, Column 1");
                        ui.label("Grid Row 2, Column 2");
                        ui.end_row();
                    });
                });
            });
        });
    }
}

impl RFSApp {
    fn render_file_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel").exact_width(350.).resizable(false).show(ctx, |ui| {

            ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                for file in self.rfs_files.iter() {
                    render_file(ui, file);
                }

                let add_file_btn = ui.add_sized([ui.available_width(), 0.0], Button::new("Add file"));
                if add_file_btn.clicked() {
                    if let Some(path) = tfd::open_file_dialog("New", &self.home_dir, None) {
                        self.add_file(path);
                    }
                }
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
}


fn render_file(ui: &mut egui::Ui, file: &RFSFile) {
    // Stretch horizontally
    ui.horizontal(|ui| {
        ui.add_sized([ui.available_width() * 0.5, 0.0], egui::Label::new(format!("{}", file.data.name)));
        ui.add_sized([ui.available_width(), 0.0], egui::Label::new(format!("{}", file.data.length)));
    });
    ui.add_space(1.);
    ui.separator();
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