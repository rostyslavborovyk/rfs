use std::cell::RefCell;
use std::{fs, thread};
use std::ops::Deref;
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};
use eframe::egui;
use eframe::egui::Color32;
use eframe::emath::{Align};
use tinyfiledialogs as tfd;
use crate::domain::files::{generate_meta_file, refresh_file_status};
use crate::peer::enums::FileStatus;
use crate::peer::file::RFSFile;
use crate::peer::state::KnownPeer;
use crate::ui::client::get_info;
use crate::values::{LOCAL_PEER_ADDRESS, SYNC_DELAY_SECS};

const ACCENT: Color32 = Color32::from_rgb(200, 255, 200);

#[derive(Default)]
pub struct AppState {
    file_id_selected: RefCell<Option<String>>,
    rfs_files: Vec<RFSFile>,
    known_peers: Vec<KnownPeer>,
}

// todo: change heap strings to str refs with lifetime
#[derive(Default)]
pub struct AppConfig {
    local_peer_address: String,
    home_dir: String,
    rfs_dir: String,
    metafiles_dir: String,
    file_parts_dir: String,
    files_dir: String,
}

pub struct AppChannels {
    // todo: change to the channel with capacity 1 to avoid memory leaks?
    sync_rx: Receiver<SyncChannelEvent>,
}


pub struct RFSApp {
    config: AppConfig,
    state: AppState,
    channels: AppChannels
}

#[derive(Debug)]
pub enum SyncChannelEvent {
    RecalculatePings,
    RefreshFileStatus,
}

fn run_sync_scheduler(sync_tx: Sender<SyncChannelEvent>) -> ! {
    loop {
        if let Err(err) = sync_tx.send(SyncChannelEvent::RecalculatePings) {
            println!("Error when sending the recalculate pings sync event: {err}")
        };
        if let Err(err) = sync_tx.send(SyncChannelEvent::RefreshFileStatus) {
            println!("Error when sending the refresh file status sync event: {err}")
        };
        thread::sleep(Duration::from_secs(SYNC_DELAY_SECS));
    }
}

impl RFSApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut config: AppConfig = Default::default();
        let mut state: AppState = Default::default();

        config.local_peer_address = LOCAL_PEER_ADDRESS.to_string();
        config.home_dir = std::env::var("HOME").unwrap_or_else(|_| "".to_string());
        config.rfs_dir = config.home_dir.clone() + "/.rfs";
        config.metafiles_dir = config.rfs_dir.clone() + "/metafiles";
        config.file_parts_dir = config.rfs_dir.clone() + "/file_parts";
        config.files_dir = config.rfs_dir.clone() + "/files";
        state.rfs_files = fs::read_dir(&config.metafiles_dir).unwrap().into_iter().map(|path| {
            let p = path.unwrap().path().to_str().unwrap().to_owned();
            RFSFile::from_path_sync(&p)
        }).collect();

        if let Err(_) = fs::read_dir(&config.metafiles_dir) {
            if let Err(err) = fs::create_dir(&config.metafiles_dir) {
                println!("Metafiles dir was not found and unable to create it! {err}")
            };
        };

        // todo: change to oneshot channel
        let (sync_tx, sync_rx) = channel();

        // spawning a thread that will trigger synchronization events in the app
        thread::spawn(move || run_sync_scheduler(sync_tx));
        
        // spawning a background thread that will handle interactions with local peer without
        // blocking main ui thread
        // todo

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
        self.render_button_panel(ctx);
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
            });
        });
    }
    fn render_button_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("right_panel").exact_width(100.).resizable(false).show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                if ui.add_sized([100., 0.0], egui::Button::new("Add .rfs file")).clicked() {
                    if let Some(path) = tfd::open_file_dialog("Select .rfs file", &self.config.home_dir, Some((&["*.rfs"], ""))) {
                        self.add_rfs_file(path);
                    }
                }
                if ui.add_sized([100., 0.0], egui::Button::new("Generate .rfs file")).clicked() {
                    if let Some(path) = tfd::open_file_dialog("Select a file to generate .rfs file", &self.config.home_dir, None) {
                        self.generate_rfs_file(path);
                    }
                }
                if ui.add_sized([100., 0.0], egui::Button::new("Open files dir")).clicked() {
                    Command::new("open")
                        .arg(&self.config.files_dir)
                        .spawn()
                        .unwrap();
                }
                if let Some(file) = self.get_selected_file() {
                    match file.status {
                        None => {}
                        Some(v) => {
                            match v {
                                FileStatus::Downloaded => {}
                                FileStatus::NotDownloaded => {
                                    if ui.add_sized([100., 0.0], egui::Button::new("Download")).clicked() {
                                        println!("Downloading file!")
                                    }
                                }
                            }
                        }
                    }
                }
                ui.add_space(5.);
            });
        });
    }

    fn listen_channels(&mut self, ctx: &egui::Context) {
        // todo: spawn a separate thread for that?
        // for now it works quite fast <1ms
        
        ctx.request_repaint();
        
        if let Ok(v) = self.channels.sync_rx.try_recv() {
            let start = Instant::now();
            
            match v {
                SyncChannelEvent::RecalculatePings => {
                    let info = get_info();
                    self.state.known_peers = info.known_peers;
                }
                SyncChannelEvent::RefreshFileStatus => {
                    for file in self.state.rfs_files.iter_mut() {
                        refresh_file_status(file, self.config.files_dir.clone());
                    }
                }
            }
            println!("Time spent for syncing {:?}: {}μ", v, start.elapsed().as_micros())
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
                    self.render_info_panel_field(ui, "number of pieces", &file.data.hashes.len().to_string(), 0.);
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
                    let downloaded_status = if let Some(s) = &file.status {
                        match s {
                            FileStatus::Downloaded => "Downloaded",
                            FileStatus::NotDownloaded => "Not downloaded"
                        }
                    } else {
                        "Not verified"
                    };
                    self.render_info_panel_field(ui, "Downloaded", downloaded_status, 0.);
                }
            }
        });
    }

    fn render_info_panel_field(&mut self, ui: &mut egui::Ui, name: &str, value: &str, space: f32) {
        ui.horizontal(|ui| {
            ui.add_space(space);
            ui.label(name);
            ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                ui.label(format!("{}", value));
            });
        });
    }

    // todo: move to domain/files.rs
    fn add_rfs_file(&mut self, path: String) {
        let path = path.clone();
        let file_name = path.split('/').last().unwrap();
        let destination = self.config.metafiles_dir.clone() + &"/" + file_name;
        fs::copy(path, &destination).unwrap_or_else(|err| {
            println!("Unable to copy file to metafiles dir {err}");
            0
        });
        self.state.rfs_files.push(RFSFile::from_path_sync(&destination));
    }

    fn generate_rfs_file(&mut self, path: String) -> Result<(), String> {
        let meta_file_path = self.config.metafiles_dir.clone()
            + "/"
            + path.clone().split('/').last().unwrap().split('.').next()
            .ok_or("Failed to parse the file name, should be in format {name}.{extension}!")?
            + ".rfs";
        if let Ok(rfs_file) = generate_meta_file(self.config.local_peer_address.clone(), &path) {
            rfs_file.save(meta_file_path.clone())?;
            self.state.rfs_files.push(RFSFile::from_path_sync(&meta_file_path));
            
            fs::copy(path, self.config.files_dir.clone() + "/" + &rfs_file.data.name).unwrap_or_else(|err| {
                println!("Unable to copy file to metafiles dir {err}");
                0
            });
        };
        Ok(())
    }

    fn render_file(&self, ui: &mut egui::Ui, file: &RFSFile) {
        ui.horizontal(|ui| {
            ui.label(format!("{}", file.data.name));
            ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                ui.label(format!("{}", file.data.length));
            });

            let btn = ui.with_layout(egui::Layout::right_to_left(Align::TOP), |ui| {
                ui.add_sized([30., 0.0], {
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
                })
            });
            
            if btn.inner.clicked() {
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