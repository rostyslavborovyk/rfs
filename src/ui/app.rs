use std::cell::RefCell;
use std::{fs, thread};
use std::ops::Deref;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use eframe::egui;
use eframe::egui::Color32;
use eframe::emath::{Align};
use tinyfiledialogs as tfd;
use crate::peer::file::RFSFile;
use crate::peer::state_container::KnownPeer;
use crate::ui::cilent::get_info;
use crate::values::RECALCULATE_PINGS_DELAY_SECS;

const ACCENT: Color32 = Color32::from_rgb(200, 255, 200);

#[derive(Default)]
pub struct AppState {
    file_id_selected: RefCell<Option<String>>,
    rfs_files: Vec<RFSFile>,
    known_peers: Vec<KnownPeer>,
}

#[derive(Default)]
pub struct AppConfig {
    home_dir: String,
    metafiles_dir: String,
}

pub struct AppChannels {
    sync_rx: Receiver<SyncChannelEvent>,
}


pub struct RFSApp {
    config: AppConfig,
    state: AppState,
    channels: AppChannels
}

#[derive(Debug)]
pub enum SyncChannelEvent {
    RecalculatePings
}

impl RFSApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut config: AppConfig = Default::default();
        let mut state: AppState = Default::default();

        config.home_dir = std::env::var("HOME").unwrap_or_else(|_| "".to_string());
        config.metafiles_dir = config.home_dir.clone() + "/.rfs_metafiles";
        state.rfs_files = fs::read_dir(&config.metafiles_dir).unwrap().into_iter().map(|path| {
            let p = path.unwrap().path().to_str().unwrap().to_owned();
            RFSFile::from_path_sync(&p)
        }).collect();

        if let Err(_) = fs::read_dir(&config.metafiles_dir) {
            if let Err(err) = fs::create_dir(&config.metafiles_dir) {
                println!("Metafiles dir was not found and unable to create it! {err}")
            };
        };

        println!("Loaded rfs files {:#?}", &state.rfs_files);

        // todo: change to oneshot channel
        let (sync_tx, sync_rx) = channel();

        // spawning timer thread that will trigger the recalculation of ping values
        thread::spawn(move || {
            loop {
                if let Err(err) = sync_tx.send(SyncChannelEvent::RecalculatePings) {
                    println!("Error when sending the recalculate pings event: {err}")
                };
                thread::sleep(Duration::from_secs(RECALCULATE_PINGS_DELAY_SECS));
            }
        });

        Self {
            config,
            state,
            channels: AppChannels {
                sync_rx,
            }
        }
    }
}

impl eframe::App for RFSApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_file_panel(ctx);
        self.render_info_panel(ctx);
        render_footer(ctx);
        self.listen_channels(ctx);
    }
}

impl RFSApp {
    fn render_file_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel").exact_width(350.).resizable(false).show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                for file in self.state.rfs_files.iter() {
                    self.render_file(ui, file);
                }
                ui.with_layout(egui::Layout::bottom_up(Align::Center), |ui| {
                    // todo add filter parameter to open_file_dialog
                    if ui.add_sized([ui.available_width(), 0.0], egui::Button::new("Add .rfs file")).clicked() {
                        if let Some(path) = tfd::open_file_dialog("Select .rfs file", &self.config.home_dir, None) {
                            self.add_file(path);
                        }
                    }
                    ui.add_space(5.);
                });
            });
        });
    }

    fn listen_channels(&mut self, ctx: &egui::Context) {
        ctx.request_repaint();
        if let Ok(v) = self.channels.sync_rx.try_recv() {
            match v {
                SyncChannelEvent::RecalculatePings => {
                    let info = get_info();
                    self.state.known_peers = info.known_peers;
                } 
            }
        }
    }

    fn get_selected_file(&self) -> Option<RFSFile> {
        match self.state.file_id_selected.borrow().deref() {
            None => None,
            Some(file_id) => {
                match self.state.rfs_files.iter().find(|f| f.data.id.eq(file_id)) {
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
                        let ping = if let Some(known_peer) = self.state.known_peers.iter().find(|p| p.address.eq(peer)) {
                            if let Some(ping) = known_peer.ping {
                                ping
                            } else {
                                0i64
                            }
                        } else {
                            0i64
                        };
                        self.render_info_panel_field(
                            ui, peer, &ping.to_string(), 20.,
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
        let destination = self.config.metafiles_dir.clone() + &"/" + file_name;
        fs::copy(path, &destination).unwrap_or_else(|err| {
            println!("Unable to copy file to metafiles dir {err}");
            0
        });
        self.state.rfs_files.push(RFSFile::from_path_sync(&destination));
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