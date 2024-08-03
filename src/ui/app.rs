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
use crate::domain::config::FSConfig;
use crate::domain::files::{generate_meta_file, refresh_file_status};
use crate::domain::fs::check_folders;
use crate::peer::connection::{ConnectionFrame, GetFileFrame, GetInfoFrame, InfoResponseFrame};
use crate::peer::enums::FileStatus;
use crate::peer::file::RFSFile;
use crate::peer::state::KnownPeer;
use crate::ui::connection::Connection;
use crate::ui::enums::LeftPanelView;
use crate::ui::format::to_readable_size;
use crate::values::{LOCAL_PEER_ADDRESS, SYNC_DELAY_SECS};

const ACCENT: Color32 = Color32::from_rgb(200, 255, 200);
const SUCCESS: Color32 = Color32::from_rgb(150, 255, 150);
const INFO: Color32 = Color32::from_rgb(150, 255, 255);
const WARNING: Color32 = Color32::from_rgb(255, 220, 200);


#[derive(Default)]
pub struct AppState {
    file_id_selected: RefCell<Option<String>>,
    left_panel_view_selected: LeftPanelView,
    rfs_files: Vec<RFSFile>,
    known_peers: Vec<KnownPeer>,
}

// todo: change heap strings to str refs with lifetime
#[derive(Default)]
pub struct AppConfig {
    local_peer_address: String,
    fs: FSConfig,
}

impl AppConfig {
    fn new() -> Self {
        let local_peer_address = LOCAL_PEER_ADDRESS.to_string();
        Self {
            local_peer_address,
            fs: FSConfig::new(None),
        }
    }
}

pub struct AppChannels {
    sync_rx: Receiver<SyncChannelEvent>,
    event_rx: Receiver<EventChannelEvent>,

    command_tx: Sender<CommandChannelEvent>,
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

#[derive(Debug)]
pub enum EventChannelEvent {
    PeersInfoUpdate(InfoResponseFrame)
}

#[derive(Debug)]
pub struct DownloadFileCommandPayload {
    file_id: String,
}

#[derive(Debug)]
pub enum CommandChannelEvent {
    GetPeersInfo,
    DownloadFile(DownloadFileCommandPayload)
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


fn run_background_worker(
    command_rx: Receiver<CommandChannelEvent>,
    event_tx: Sender<EventChannelEvent>,
) -> ! {
    let mut connection = Connection::from_address(&LOCAL_PEER_ADDRESS.to_string()).unwrap();
    loop {
        if let Ok(command) = command_rx.try_recv() {
            match command {
                CommandChannelEvent::GetPeersInfo => {
                    connection.write_frame(ConnectionFrame::GetInfo(GetInfoFrame {}));
                }
                CommandChannelEvent::DownloadFile(payload) => {
                    let file_id = payload.file_id;
                    connection.write_frame(ConnectionFrame::GetFile(GetFileFrame { file_id }));
                }
            }
        }
        if let Ok(response) = connection.read_frame() {
            match response {
                ConnectionFrame::InfoResponse(frame) => {
                    event_tx.send(EventChannelEvent::PeersInfoUpdate(frame)).unwrap()
                }
                _ => {}
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
}


impl RFSApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = AppConfig::new();
        let mut state: AppState = Default::default();

        state.rfs_files = fs::read_dir(&config.fs.metafiles_dir).unwrap()
            .into_iter().map(|path| {
                let p = path.unwrap().path().to_str().unwrap().to_owned();
                if p.ends_with(".rfs") {
                    Some(RFSFile::from_path_sync(&p))
                } else {
                    None
                }
        }).flatten().collect();

        check_folders(&config.fs);

        // todo: change to oneshot channel
        let (sync_tx, sync_rx) = channel();
        let (command_tx, command_rx) = channel();
        let (event_tx, event_rx) = channel();

        // spawning a thread that will trigger synchronization events in the app
        thread::spawn(move || run_sync_scheduler(sync_tx));
        
        // spawning a background thread that will handle interactions with local peer without
        // blocking main ui thread
        thread::spawn(move || run_background_worker(command_rx, event_tx));

        Self {
            config,
            state,
            channels: AppChannels {
                sync_rx,
                command_tx,
                event_rx,
            }
        }
    }
}

impl eframe::App for RFSApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_left_panel(ctx);
        self.render_info_panel(ctx);
        self.render_button_panel(ctx);
        render_footer(ctx);
        self.listen_channels(ctx);
    }
}

impl RFSApp {
    fn render_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel").exact_width(350.).resizable(false).show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                let mut files_btn = egui::Button::new("Files");
                let mut known_peers_btn = egui::Button::new("Known peers");
                match self.state.left_panel_view_selected {
                    LeftPanelView::Files => {
                        files_btn = files_btn.fill(ACCENT);
                    },
                    LeftPanelView::KnownPeers => {
                        known_peers_btn = known_peers_btn.fill(ACCENT);
                    }
                };
                if ui.add_sized([ui.available_width() * 0.5, 0.0], files_btn).clicked() {
                    self.state.left_panel_view_selected = LeftPanelView::Files;
                };
                if ui.add_sized([ui.available_width(), 0.0], known_peers_btn).clicked() {
                    self.state.left_panel_view_selected = LeftPanelView::KnownPeers;
                };
            });
            match self.state.left_panel_view_selected {
                LeftPanelView::Files => {
                    ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                        for file in self.state.rfs_files.iter() {
                            self.render_file(ui, file);
                        }
                    });
                }
                LeftPanelView::KnownPeers => {}
            }
        });
    }
    fn render_button_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("right_panel").exact_width(100.).resizable(false).show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                if ui.add_sized([100., 0.0], egui::Button::new("Add .rfs file")).clicked() {
                    if let Some(path) = tfd::open_file_dialog("Select .rfs file", &self.config.fs.home_dir, Some((&["*.rfs"], ""))) {
                        self.add_rfs_file(path);
                    }
                }
                if ui.add_sized([100., 0.0], egui::Button::new("Generate .rfs file")).clicked() {
                    if let Some(path) = tfd::open_file_dialog("Select a file to generate .rfs file", &self.config.fs.home_dir, None) {
                        self.generate_rfs_file(path);
                    }
                }
                if ui.add_sized([100., 0.0], egui::Button::new("Open files dir")).clicked() {
                    Command::new("open")
                        .arg(&self.config.fs.files_dir)
                        .spawn()
                        .unwrap();
                }
                if ui.add_sized([100., 0.0], egui::Button::new("Open metafiles dir")).clicked() {
                    Command::new("open")
                        .arg(&self.config.fs.metafiles_dir)
                        .spawn()
                        .unwrap();
                }
                ui.add_space(5.);
            });
        });
    }

    fn listen_channels(&mut self, ctx: &egui::Context) {
        ctx.request_repaint();
        
        if let Ok(v) = self.channels.sync_rx.try_recv() {
            let start = Instant::now();
            
            match v {
                SyncChannelEvent::RecalculatePings => {
                    self.channels.command_tx.send(CommandChannelEvent::GetPeersInfo).unwrap();
                }
                SyncChannelEvent::RefreshFileStatus => {
                    for file in self.state.rfs_files.iter_mut() {
                        refresh_file_status(file, self.config.fs.files_dir.clone());
                    }
                }
            }
            println!("Time spent for syncing {:?}: {}μ", v, start.elapsed().as_micros())
        }

        if let Ok(v) = self.channels.event_rx.try_recv() {
            let start = Instant::now();

            match v {
                EventChannelEvent::PeersInfoUpdate(frame) => {
                    self.state.known_peers = frame.known_peers;
                }
            }
            println!("Time spent for syncing events: {}μ",  start.elapsed().as_micros())
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
                    self.render_info_panel_field(ui, "size", &to_readable_size(file.data.length), 0.);
                    self.render_info_panel_field(ui, "piece size", &file.data.piece_size.to_string(), 0.);
                    self.render_info_panel_field(ui, "number of pieces", &file.data.hashes.len().to_string(), 0.);
                    
                    let peers_count = file.data.peers.iter().filter(|p| !p.eq(&&self.config.local_peer_address)).count();
                    self.render_info_panel_field(ui, "peers", &peers_count.to_string(), 0.);
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
                        if peer == LOCAL_PEER_ADDRESS {
                            continue;
                        }
                        self.render_info_panel_field(ui, peer, &ping.to_string(), 20.);
                    }
                    let downloaded_status = if let Some(s) = &file.status {
                        match s {
                            FileStatus::Downloaded => "Downloaded",
                            FileStatus::NotDownloaded => "Not downloaded",
                            FileStatus::Downloading => "Downloading",
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
        let destination = self.config.fs.metafiles_dir.clone() + &"/" + file_name;
        fs::copy(path, &destination).unwrap_or_else(|err| {
            println!("Unable to copy file to metafiles dir {err}");
            0
        });
        self.state.rfs_files.push(RFSFile::from_path_sync(&destination));
    }

    fn generate_rfs_file(&mut self, path: String) -> Result<(), String> {
        let meta_file_path = self.config.fs.metafiles_dir.clone()
            + "/"
            + path.clone().split('/').last().unwrap().split('.').next()
            .ok_or("Failed to parse the file name, should be in format {name}.{extension}!")?
            + ".rfs";
        if let Ok(rfs_file) = generate_meta_file(self.config.local_peer_address.clone(), &path) {
            rfs_file.save(meta_file_path.clone())?;
            self.state.rfs_files.push(RFSFile::from_path_sync(&meta_file_path));
            
            fs::copy(path, self.config.fs.files_dir.clone() + "/" + &rfs_file.data.name).unwrap_or_else(|err| {
                println!("Unable to copy file to metafiles dir {err}");
                0
            });
        };
        Ok(())
    }

    fn render_file(&self, ui: &mut egui::Ui, file: &RFSFile) {
        ui.horizontal(|ui| {
            ui.label(format!("{}", file.data.name));

            ui.with_layout(egui::Layout::right_to_left(Align::TOP), |ui| {
                if ui.add_sized([30., 0.0], {
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
                }).clicked() {
                    self.state.file_id_selected.replace(Some(file.data.id.clone()));
                };
                
                match file.status.clone() {
                    None => {}
                    Some(v) => {
                        match v {
                            FileStatus::Downloaded => {
                                ui.add_sized([30., 0.0], egui::Button::new("⬇").fill(SUCCESS));
                            }
                            FileStatus::NotDownloaded => {
                                let has_accessible_peers = file.data.peers.iter().map(|p| {
                                    let kp = self.state.known_peers.iter().find(|kp| kp.address.eq(p));
                                    if let Some(v) = kp {
                                        v.accessible()
                                    } else {
                                        false
                                    }
                                }).any(|v| v);
                                
                                if has_accessible_peers {
                                    if ui.add_sized([30., 0.0], egui::Button::new("⬇")).clicked() {
                                        let command = CommandChannelEvent::DownloadFile(
                                            DownloadFileCommandPayload {
                                                file_id: file.data.id.clone()
                                            }
                                        );
                                        self.channels.command_tx.send(command).unwrap()
                                    }
                                } else {
                                    ui.add_sized([30., 0.0], egui::Button::new("⬇").fill(WARNING)).clicked();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            });
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