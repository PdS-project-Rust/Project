mod screenshots_module;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use image::ImageFormat;
use screenshots::Screen;
use show_image::winit::event_loop::{ControlFlow, EventLoopBuilder};
use crate::screenshots_module::screenshot_module::Screenshot;

fn main() {
    let screens=Screen::all().unwrap();
    let path=PathBuf::from(r"C:\Users\miche\Downloads".to_string());
    for i in screens{
        let mut ss=Screenshot::new(i).unwrap();
        println!("{:?}",i.display_info);
        ss.resize_image(0,ss.get_height().unwrap()/2-1,ss.get_height().unwrap()/2,ss.get_width().unwrap()/2).unwrap();
        ss.rotate_dx_90().unwrap();
        ss.save_to_clipboard().unwrap();
        ss.save_image(&path,ImageFormat::Jpeg).unwrap();
    }
    let ss1=Screenshot::screenshot_after_delay(Duration::from_secs(3),Screen::all().unwrap()[0]).unwrap();
    let event_loop=EventLoopBuilder::new().build();
    let hotkeys_manager = GlobalHotKeyManager::new().unwrap();

    let hotkey = HotKey::new(Some(Modifiers::SHIFT), Code::KeyD);

    hotkeys_manager.register(hotkey).unwrap();

    let global_hotkey_channel = GlobalHotKeyEvent::receiver();

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if let Ok(event) = global_hotkey_channel.try_recv() {
            println!("{event:?}");
        }
    })
}
