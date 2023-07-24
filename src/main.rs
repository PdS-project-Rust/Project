mod screenshots_module;
mod hotkey_module;
use crate::hotkey_module::hotkey_module::HotkeyManager;
use std::path::PathBuf;
use std::time::Duration;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyEventReceiver};
use global_hotkey::hotkey::Code::KeyD;
use global_hotkey::hotkey::Modifiers;
use image::ImageFormat;
use screenshots::Screen;
use show_image::winit::event_loop::{ControlFlow, EventLoopBuilder};
use crate::screenshots_module::screenshot_module::Screenshot;

fn main() {
    let screens=Screen::all().unwrap();
    let path=PathBuf::from(r"C:\Users\giuli\Downloads".to_string());
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

    //HOTKEY WRAPPER, SET THE HOTKEY AND REMOVE THE OLDER IF NEEDED
    let mut hotkey_manager =HotkeyManager::new().unwrap();
    let keyid=hotkey_manager.register_new_hotkey(Some(Modifiers::CONTROL), KeyD).unwrap();


    //ADDING THE CALLBACK TO AN EVENT
    let global_hotkey_channel = GlobalHotKeyEvent::receiver();
    GlobalHotKeyEvent::set_event_handler(Option::Some(move |event:GlobalHotKeyEvent|{
        if keyid==event.id{
            println!("{:?}",event);
        }
    }));

    //EVENT LOOP HANDLER
    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
    })
}
