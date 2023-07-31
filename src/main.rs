mod screenshots_module;
mod hotkey_module;
mod api_module;
mod settings_module;

use eframe::{egui::{CentralPanel, Layout, Align, TextEdit, Direction, CursorIcon}, egui::{Window, ComboBox, TopBottomPanel, self}, App, NativeOptions, epaint::{ColorImage, Vec2, Pos2}};
use crate::api_module::api_module as api_mod;
use crate::hotkey_module::hotkey_module::HotkeyManager;
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use global_hotkey::GlobalHotKeyEvent;
use global_hotkey::hotkey::Modifiers;
use image::{EncodableLayout, ImageFormat};
use imageproc::drawing::draw_antialiased_line_segment_mut;
use tao::event_loop::{EventLoop,ControlFlow};
use crate::screenshots_module::screenshot_module::Screenshot;
use crate::settings_module::settings_module::*;

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
        Box::new(|_cc| Box::<ScreenshotStr>::default()),
    ).unwrap();
}

struct MyImage {
    texture: Option<egui::TextureHandle>,
}

impl MyImage {
    pub fn ui_resize(&mut self, ui: &mut egui::Ui, image: ColorImage) {
        let texture: &egui::TextureHandle = self.texture.get_or_insert_with(|| {
            // Load the texture only once.
            ui.ctx().load_texture(
                "my-image",
                image.clone(),
                Default::default()
            )
        });

        let available = ui.available_size();
        let w = image.width() as f32;
        let h = image.height() as f32;
        let w_window = available.x;
        let h_window = available.y;
        let height = h_window.min(w_window * h / w);
        let width = height * w / h;
        let fixed_dimensions = Vec2{x: width, y: height};
        // Show the image:
        ui.image(texture, fixed_dimensions);
    }

    pub fn new() -> Self {
        Self { texture: None }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DrawingMode {
    Paint,
    Highlight,
}

struct ScreenshotStr {
    timer:usize,
    screen:usize,
    screenshot:Screenshot,
    path:PathBuf,
    format:ImageFormat,
    color_image:ColorImage,
    show_image:bool,
    save_dialog:bool,
    drawing_mode:Option<DrawingMode>,
    settings_dialog:bool,
    settings:Settings,
    instant:Instant,
    starting_point:Option<(f32, f32)>,
}

impl Default for ScreenshotStr {
    fn default() -> Self {
        Self {
            timer:0,
            screen:0,
            screenshot:Screenshot::new_empty(),
            path:PathBuf::from(r"./".to_string()),
            format:ImageFormat::Png,
            color_image:ColorImage::example(),
            show_image:false,
            save_dialog:false,
            drawing_mode:None,
            settings_dialog:false,
            settings:Settings::default(),
            instant:Instant::now(),
            starting_point:None,
         }
    }
}

impl ScreenshotStr {
    pub fn toggle_drawing_mode(&mut self, mode: DrawingMode) {
        if self.drawing_mode == Some(mode.clone()) {
            self.drawing_mode = None;
        } else {
            self.drawing_mode = Some(mode);
        }
        self.show_image = true;
    }

    pub fn _convert_image(&mut self) -> () {
        let image = self.screenshot.get_image().unwrap();
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let col_im: ColorImage = ColorImage::from_rgba_unmultiplied(
            size,
            image_buffer.as_bytes()
        );
        
        self.color_image = col_im;
    }

    fn calculate_texture_coordinates(&self, cursor_pos: Pos2, ctx: &egui::Context) -> Pos2 {
        let available = ctx.available_rect(); // Get the available rectangle for drawing
        let w = self.screenshot.get_width().unwrap() as f32;
        let h = self.screenshot.get_height().unwrap() as f32;
        let w_window = available.width();
        let h_window = available.height();
        let height = h_window.min(w_window * h / w);
        let width = height * w / h;
        let h_scale = height / h;
        let w_scale = width / w;
        let image_pos_x = (w_window - width) / 2.0;
        let image_pos_y = (h_window - height) / 2.0;
        let image_cursor_pos = Pos2 {
            x: (cursor_pos.x - image_pos_x) / w_scale,
            y: (cursor_pos.y - image_pos_y) / h_scale,
        };
        image_cursor_pos
    }

    fn draw_paint(&mut self, ctx: &egui::Context, size: f32, color: [u8;4]) {
        ctx.input(|ui| {
            let pos = ui.pointer.interact_pos();
            if let Some(pos) = pos {
                let texture_coordinates = self.calculate_texture_coordinates(pos, ctx);
                let x = texture_coordinates.x;
                let y = texture_coordinates.y;

                if ui.pointer.any_down() {
                    if self.starting_point.is_none() {
                        self.starting_point = Some((x, y));
                    } else {
                        self.screenshot.draw_line(
                            self.starting_point.unwrap(),
                            (x, y),
                            color,
                            size,
                        );
                        self.starting_point = Some((x, y));
                        if Instant::now() > self.instant {
                            self._convert_image();
                            self.instant += Duration::from_millis(16);
                        }
                    }
                } else {
                    self.starting_point = None;
                }
            }
        });
    }

    fn draw_highlight(&mut self, ctx: &egui::Context, size: f32) {
        ctx.input(|ui| {
            let pos = ui.pointer.interact_pos();
            if let Some(pos) = pos {
                let texture_coordinates = self.calculate_texture_coordinates(pos, ctx);
                let x = texture_coordinates.x;
                let y = texture_coordinates.y;

                if ui.pointer.any_down() {
                    if self.starting_point.is_none() {
                        self.starting_point = Some((x, y));
                    } else {
                        self.screenshot.highlight_line(
                            self.starting_point.unwrap(),
                            (x, y),
                            size,
                        );
                        let mut dx = 1.0;
                        if self.starting_point.unwrap().0 > x { dx = -1.0 }
                        self.starting_point = Some((x+dx, y));
                        if Instant::now() > self.instant {
                            self._convert_image();
                            self.instant += Duration::from_millis(16);
                        }
                    }
                } else {
                    self.starting_point = None;
                }
            }
        });
    }


}

impl App for ScreenshotStr {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // drawing
        match self.drawing_mode {
            Some(DrawingMode::Paint) => {
                let size = 5.0;
                let color: [u8; 4] = [255, 0, 0, 255];
                self.draw_paint(ctx, size, color);
            }
            Some(DrawingMode::Highlight) => {
                let size = 20.0;
                self.draw_highlight(ctx, size);
            }
            None => {
                // nessuna modalità selezionata
            }
        }

        // save dialog
        if self.save_dialog {
            Window::new("Save Screenshot")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    //close button 
                    ui.horizontal(|ui| {
                        ui.label("Save as?");
                        if ui.button("PNG").clicked() {
                            self.format=ImageFormat::Png;
                            self.screenshot.save_image(&PathBuf::from(&self.settings.path), self.format).unwrap();
                            self.save_dialog=false;
                        }
                        if ui.button("JPG").clicked() {
                            self.format=ImageFormat::Jpeg;
                            self.screenshot.save_image(&PathBuf::from(&self.settings.path), self.format).unwrap();
                            self.save_dialog=false;
                        }
                        if ui.button("GIF").clicked() {
                            self.format=ImageFormat::Gif;
                            self.screenshot.save_image(&PathBuf::from(&self.settings.path), self.format).unwrap();
                            self.save_dialog=false;
                        }

                    });

                    //close
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.save_dialog=false;
                        }
                    });
                });
        }

        // settings dialog
        if self.settings_dialog {
            Window::new("Settings")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Change Hotkeys");
                    ui.horizontal(|ui| {
                        ui.label("Open App");
                        ui.label("CTRL + ");
                        ui.add(TextEdit::singleline(&mut self.settings.open).desired_width(10.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Quick Screenshot");
                        ui.label("CTRL + ");
                        ui.add(TextEdit::singleline(&mut self.settings.quick).desired_width(10.0));
                    }); 
                    ui.horizontal(|ui| {
                        ui.label("Path");
                        //turn pathbuf into string
                        ui.add(TextEdit::singleline(&mut self.settings.path));
                    });

                    //close
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.settings_dialog=false;
                        }
                        if ui.button("Save").clicked() {
                            write_settings_to_file("settings.json".to_string(), &self.settings).unwrap();
                            self.settings_dialog=false;
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
                    self._convert_image();
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
                        for (index,screen) in screens.iter().enumerate() {
                            if ui.selectable_value(&mut self.screen, index, &format!("Screen {} ({}x{})",index,screen.display_info.height,screen.display_info.width)).clicked() {
                                self.screen=index;
                            }
                        }
                    });

                ui.separator();

                // save button
                if ui.button("\u{1F4BE}").clicked() {
                    self.save_dialog=true;
                }

                // settings button in the top right corner
                ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                    if ui.button("\u{2699}").clicked() {
                        self.settings_dialog=true;
                    }
                });

            });

        });
        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                if self.show_image {
                    let mut my_image = MyImage::new();
                    my_image.ui_resize(ui, self.color_image.clone()); 
                   
                }
            });
        });

// footer of the app
        TopBottomPanel::bottom("footer")
            .resizable(false)
            .show(ctx, |ui| {
                if self.show_image {
                    ui.horizontal(|ui| {
                        // rotate left
                        if ui.button("\u{27F3}").clicked() {
                            self.screenshot.rotate_sx_90().unwrap();
                            self.show_image=true;
                        }

                        // rotate right
                        if ui.button("\u{27F2}").clicked() {
                            self.screenshot.rotate_dx_90().unwrap();
                            self.show_image=true;
                        }

                        // crop
                        if ui.button("\u{2702}").clicked() {
                            self.screenshot.resize_image(190, 200, 300,  200).unwrap();
                            self.show_image=true;
                        }

                        // draw
                        if ui.button("\u{270F}").clicked() {
                           self.toggle_drawing_mode(DrawingMode::Paint);
                        }

                        // highlight
                        if ui.button("\u{1F526}").clicked() {
                           self.toggle_drawing_mode(DrawingMode::Highlight);
                        }

                        // erase
                        if ui.button("\u{1F4D8}").clicked() {

                        }

                        // shapes
                        if ui.button("\u{2B55}").clicked() {

                        }

                        // text
                        if ui.button("\u{1F1F9}").clicked() {

                        }

                        // undo
                        if ui.button("\u{21A9}").clicked() {

                        }

                        // redo
                        if ui.button("\u{21AA}").clicked() {

                        }
                    });

                }

            });
    }
}

fn main() {
    //HOTKEYS
    let event_loop = EventLoop::new();
    let mut hotkey_manager_open =HotkeyManager::new().unwrap();
    let mut hotkey_manager_quick =HotkeyManager::new().unwrap();
    let global_hotkey_channel = GlobalHotKeyEvent::receiver();
    //READING FROM FILE
    let startup_settings = read_settings_from_file("settings.json".to_string()).unwrap();
    let key_open = startup_settings.get_open_hotkey();
    let key_screenshot = startup_settings.get_screenshot_hotkey();
    //REGISTERING THE HOTKEYS FROM FILE
    let mut keyid_open=hotkey_manager_open.register_new_hotkey(Some(Modifiers::CONTROL), key_open.unwrap()).unwrap(); //OPEN APP
    let mut keyid_screenshot=hotkey_manager_quick.register_new_hotkey(Some(Modifiers::CONTROL), key_screenshot.unwrap()).unwrap(); //OPEN APP
    //EVENT LOOP BACKGROUND
    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Ok(event) = global_hotkey_channel.try_recv() {
            println!("{},{}",event.id,keyid_open);
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
                let ss = api_mod::take_screenshot(Duration::from_secs(0), 0);
                ss.save_image(&PathBuf::from(startup_settings.path), ImageFormat::Png).unwrap();
            }
        }
    });
}
