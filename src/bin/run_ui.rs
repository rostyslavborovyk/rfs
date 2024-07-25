use eframe::Theme;
use distributed_fs::ui::app::RFSApp;

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.default_theme = Theme::Light;
    native_options.follow_system_theme = false;

    
    eframe::run_native(
        "Rfs",
        native_options, Box::new(|cc| Ok(Box::new(RFSApp::new(cc)))),
    );
}
