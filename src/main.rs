mod screenshots_module;
mod hotkey_module;
mod settings_module;
mod state_module;

use eframe::{App, NativeOptions, egui};
use crate::state_module::state_module::{take_screenshot};
use crate::hotkey_module::hotkey_module::HotkeyManager;
use std::{path::PathBuf};
use std::time::Duration;
use global_hotkey::GlobalHotKeyEvent;
use global_hotkey::hotkey::Modifiers;
use image::{ImageFormat};
use tao::event_loop::{EventLoop,ControlFlow};
use crate::settings_module::settings_module::*;
use crate::state_module::state_module::{ScreenshotStr};

fn build_gui() -> () {
    //APP CONF
    let options = NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 400.0)),
        ..Default::default()
    };


    println!("Starting app");
    eframe::run_native(
        "Screen Capture",
        options,
        Box::new(|_cc| {
            Box::<ScreenshotStr>::new(ScreenshotStr::default())
        }),
    ).unwrap();
}


fn main() {
    //HOTKEYS
    let event_loop = EventLoop::new();
    let mut hotkey_manager_open =HotkeyManager::new().unwrap();
    let mut hotkey_manager_quick =HotkeyManager::new().unwrap();
    let global_hotkey_channel = GlobalHotKeyEvent::receiver();
    //READING FROM FILE
    let startup_settings = read_settings_from_file("settings.json".to_string()).unwrap();
    println!("{:?}", startup_settings);
    let key_open = startup_settings.get_open_hotkey();
    let key_screenshot = startup_settings.get_screenshot_hotkey();
    //REGISTERING THE HOTKEYS FROM FILE
    let mut keyid_open=hotkey_manager_open.register_new_hotkey(Some(Modifiers::CONTROL), key_open.unwrap()).unwrap(); //OPEN APP
    let mut keyid_screenshot=hotkey_manager_quick.register_new_hotkey(Some(Modifiers::CONTROL), key_screenshot.unwrap()).unwrap(); //OPEN APP
    //EVENT LOOP BACKGROUND
    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Ok(event) = global_hotkey_channel.try_recv() {
            if keyid_open==event.id{
                println!("Opening App");
                println!("{:?}", startup_settings);
                hotkey_manager_quick.clean_hotkey().unwrap();
                hotkey_manager_open.clean_hotkey().unwrap();
                //HERE THE BACKGROUND EVENT LOOP IS STOPPED AND THE GUI'S ONE STARTS
                build_gui();
                let startup_settings = read_settings_from_file("settings.json".to_string()).unwrap();
                let key_open = startup_settings.get_open_hotkey();
                let key_screenshot = startup_settings.get_screenshot_hotkey();
                keyid_open=hotkey_manager_open.register_new_hotkey(Some(Modifiers::CONTROL), key_open.unwrap()).unwrap(); //OPEN APP
                keyid_screenshot=hotkey_manager_quick.register_new_hotkey(Some(Modifiers::CONTROL), key_screenshot.unwrap()).unwrap(); //OPEN APP
            }
            if keyid_screenshot==event.id {
                println!("Screenshot taken");
                let startup_settings = read_settings_from_file("settings.json".to_string()).unwrap();
                let ss = take_screenshot(Duration::from_secs(0), 0);
                ss.save_image(&PathBuf::from(startup_settings.path), ImageFormat::Png).unwrap();
            }
        }
    });
}
