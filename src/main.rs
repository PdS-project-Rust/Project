mod screenshots_module;
mod hotkey_module;
mod settings_module;
mod state_module;

use eframe::{NativeOptions, egui, IconData};
use crate::state_module::state_module::ScreenshotStr;

fn build_gui() -> () {
    let icon = image::open("./resources/icon.png").expect("Failed to open icon path").to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();

    //APP CONF
    let options = NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 400.0)),
        min_window_size: Some(egui::vec2(640.0, 400.0)),
        icon_data: Some(IconData{
            rgba: icon.into_raw(),
            width: icon_width,
            height: icon_height,
        }),
        ..Default::default()
    };


    println!("Starting app");
    eframe::run_native(
        "Rusty Capture",
        options,
        Box::new(|_cc| {
            Box::<ScreenshotStr>::new(ScreenshotStr::default())
        }),
    ).unwrap();
    println!("closing eframe");
}


fn main() {
    //HOTKEYS
    build_gui();
}
