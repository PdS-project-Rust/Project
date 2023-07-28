mod screenshots_module;
mod hotkey_module;
mod api_module;

use eframe::{egui::{CentralPanel, Layout, Align, Button, Color32, TextureId}, egui::{Window, ComboBox, TopBottomPanel}, epi::App, run_native, NativeOptions};
use crate::api_module::api_module as api_mod;
use crate::hotkey_module::hotkey_module::HotkeyManager;
use std::path::PathBuf;
use std::time::Duration;
use global_hotkey::GlobalHotKeyEvent;
use global_hotkey::hotkey::Code::KeyD;
use global_hotkey::hotkey::Modifiers;
use image::ImageFormat;
use show_image::winit::event_loop::{ControlFlow, EventLoopBuilder};
use crate::screenshots_module::screenshot_module::Screenshot;

struct ScreenshotStr {
    timer:usize,
    screen:usize,
    screenshot:Screenshot,
    path:PathBuf,
    format:ImageFormat,
    texture:TextureId,
    show_image:bool,
    show_dialog:bool
}

impl Default for ScreenshotStr {
    fn default() -> Self {
        Self {
            timer:0,
            screen:0,
            screenshot:Screenshot::new_empty(),
            path:PathBuf::from(r"C:\Users\giuli\Downloads".to_string()),
            format:ImageFormat::Png,
            texture:TextureId::default(),
            show_image:false,
            show_dialog:false
         }
    }
}

impl ScreenshotStr {

    pub fn get_texture(&mut self, frame: &mut eframe::epi::Frame<'_>) -> () {
        let pixels = self.screenshot.get_image().unwrap();
        let h: usize = self.screenshot.get_height().unwrap() as usize;
        let w: usize = self.screenshot.get_width().unwrap() as usize;
        let size = (w, h);
        let pixels: Vec<_> = pixels
            .chunks_exact(4)
            .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();

        let texture = frame
            .tex_allocator()
            .alloc_srgba_premultiplied(size, &pixels);
        self.texture = texture;

    }
}

fn build_gui() -> () {
    //APP CONF
    let app = ScreenshotStr::default();
    let options = NativeOptions::default();
    run_native(
        Box::new(app),
        options,
    );
}

impl App for ScreenshotStr {
    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {

        // test dialog
        if self.show_dialog {
            Window::new("Screenshot")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Screenshot taken!");
                        ui.separator();
                        if ui.button("Close").clicked() {
                            self.show_dialog=false;
                        }
                    });
                });
        }

        // header of the app
        TopBottomPanel::top("header").show(ctx, |ui| {
            let timer = self.timer;
            let screen = self.screen;

            let timer_str = format!("{} Seconds", timer);
            let screen_str = format!("Screen {}", screen);

            ui.horizontal(|ui| {
                if ui.button("New Screenshot").clicked() {
                    let duration = Duration::from_secs(self.timer as u64);
                    
                    self.screenshot=api_mod::take_screenshot(duration,self.screen);
                    self.screenshot.save_image(&self.path,self.format).unwrap();
                    self.get_texture(frame);
                    self.show_image=true;
                }

                ui.separator();

                // combo box timer for the screenshot
                ComboBox::from_label("Timer")
                    .selected_text(timer_str)
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut 0, 0, "No Timer").clicked() {
                            self.timer=0;
                        }
                        if ui.selectable_value(&mut 3, 3, "3 Seconds").clicked() {
                            self.timer=3;
                        }
                        if ui.selectable_value(&mut 5, 5, "5 Seconds").clicked() {
                            self.timer=5;
                        }
                        if ui.selectable_value(&mut 10, 10, "10 Seconds").clicked() {
                            self.timer=10;
                        }

                    });

                // combo box screen for the screenshot
                ComboBox::from_label("Screen")
                    .selected_text(screen_str)
                    .show_ui(ui, |ui| {
                        let screens=api_mod::get_screens();
                        for screen in screens {
                            if ui.selectable_value(&mut self.screen, screen, &format!("Screen {}",screen)).clicked() {
                                self.screen=screen;
                            }
                        }
                    });

                ui.separator();

                // save button
                if ui.button("Save").clicked() {
                    self.screenshot.save_image(&self.path,self.format).unwrap();
                }

                // settings button in the top right corner
                ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                    ui.add(Button::new("Settings"));
                });

            });

        });
        CentralPanel::default().show(ctx, |ui| {
            //Show screenshot here
            if self.show_image {
                let texture = self.texture;
                let available = ui.available_size();
                ui.image(texture, available);
                self.show_dialog=true;
               
            }
        });

        // footer of the app
        TopBottomPanel::bottom("footer")
            .resizable(false)
            .show(ctx, |ui| {
            if self.show_image {
                ui.horizontal(|ui| {
                    // rotate left
                    if ui.button("Rotate Left").clicked() {
                        self.screenshot.rotate_dx_90().unwrap();
                        self.get_texture(frame);
                        self.show_image=true;
                    }
    
                    // rotate right
                    if ui.button("Rotate Right").clicked() {
                        self.screenshot.rotate_sx_90().unwrap();
                        self.get_texture(frame);
                        self.show_image=true;
                    }
    
                    // crop
                    if ui.button("Crop").clicked() {
    
                    }
    
                    // draw
                    if ui.button("Draw").clicked() {
    
                    }
    
                    // highlight
                    if ui.button("Highlight").clicked() {
    
                    }
    
                    // erase
                    if ui.button("Erase").clicked() {
    
                    }
    
                    // shapes
                    if ui.button("Shapes").clicked() {
    
                    }
    
                    // text
                    if ui.button("Text").clicked() {
    
                    }
    
                    // undo
                    if ui.button("Undo").clicked() {
    
                    }
    
                    // redoz
                    if ui.button("Redo").clicked() {
    
                    }
                });
    
            }
            
        });
    }

    fn name(&self) -> &str {
        "Progetto PDS"
    }

}

fn main() {

    //HOTKEYS
    let event_loop=EventLoopBuilder::new().build();
    let mut hotkey_manager =HotkeyManager::new().unwrap();
    let keyid=hotkey_manager.register_new_hotkey(Some(Modifiers::CONTROL), KeyD).unwrap(); //OPEN APP

    //ADDING THE CALLBACK TO AN EVENT
    GlobalHotKeyEvent::set_event_handler(Option::Some(move |event:GlobalHotKeyEvent|{
        if keyid==event.id{
            build_gui();
        }
    }));

    //EVENT LOOP HANDLER
    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
    });    
}
