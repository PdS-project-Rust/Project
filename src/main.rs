mod screenshots_module;
mod hotkey_module;
mod api_module;

use eframe::{egui::{CentralPanel, Layout, Align, Button}, egui::{Window, ComboBox, TopBottomPanel, self}, App, run_native, NativeOptions, epaint::ColorImage};
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
    color_image:ColorImage,
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
            color_image:ColorImage::example(),
            show_image:false,
            show_dialog:false
         }
    }
}

impl ScreenshotStr {
    pub fn show_image(&self) -> () {
        let image = self.screenshot.get_image().unwrap();
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        let col_im: ColorImage = ColorImage::from_rgba_unmultiplied(
            size,
            pixels.as_slice(),
        );

    }
}
fn build_gui() -> () {
    //APP CONF
    let options = NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::<ScreenshotStr>::default()),
    );
}

impl App for ScreenshotStr {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

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
                let available = ui.available_size();
                //must go from color image to texture
                ui.image(texture, available);
               
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

                        self.show_image=true;
                    }
    
                    // rotate right
                    if ui.button("Rotate Right").clicked() {
                        self.screenshot.rotate_sx_90().unwrap();
   
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
    
                    // redo
                    if ui.button("Redo").clicked() {
    
                    }
                });
    
            }
            
        });
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
