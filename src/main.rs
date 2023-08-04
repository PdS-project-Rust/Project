mod screenshots_module;
mod hotkey_module;
mod settings_module;
mod state_module;

use eframe::{egui::{CentralPanel, Layout, Align, TextEdit, Direction, Key, Window, ComboBox, TopBottomPanel, CursorIcon}, App, NativeOptions, epaint::{ColorImage, Vec2, Pos2}, egui};
use crate::state_module::state_module::{take_screenshot,get_screens};
use crate::hotkey_module::hotkey_module::HotkeyManager;
use std::{cmp, path::PathBuf, thread};
use std::time::Duration;
use eframe::egui::{Color32, Frame, Margin, Rect, Slider};
use eframe::epaint::Stroke;
use global_hotkey::GlobalHotKeyEvent;
use global_hotkey::hotkey::Modifiers;
use image::{ImageFormat, DynamicImage};
use rusttype::Scale;
use tao::event_loop::{EventLoop,ControlFlow};
use crate::settings_module::settings_module::*;
use crate::state_module::state_module::{DrawingMode, ScreenshotStr, Shape};

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
        // gives the min between the height of the window and the height of the image scaled to the width of the window
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

impl App for ScreenshotStr {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
                            //error handling
                            let result= self.screenshot.save_image(&PathBuf::from(&self.settings.path), self.format);
                            self.manage_errors(result);
                            self.save_dialog=false;
                        }
                        if ui.button("JPG").clicked() {
                            self.format=ImageFormat::Jpeg;
                            let result=self.screenshot.save_image(&PathBuf::from(&self.settings.path), self.format);
                            self.manage_errors(result);
                            self.save_dialog=false;
                        }
                        if ui.button("GIF").clicked() {
                            self.format=ImageFormat::Gif;
                            let result=self.screenshot.save_image(&PathBuf::from(&self.settings.path), self.format);
                            self.manage_errors(result);
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
                            self.drawing_mode=self.previous_drawing_mode;
                            self.settings_dialog=false;
                        }
                        if ui.button("Save").clicked() {
                            let result=write_settings_to_file("settings.json".to_string(), &self.settings);
                            self.drawing_mode=self.previous_drawing_mode;
                            self.manage_errors(result);
                            self.settings_dialog=false;
                        }
                    });
                });
        }
       
        // error dialog 
        if self.error_dialog {
            Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Error: {}", self.error_message));
                    ui.horizontal(|ui| {
                        if ui.button("Ok").clicked() {
                            self.drawing_mode=self.previous_drawing_mode;
                            self.error_dialog=false;
                        }
                    });
                });
        }
       
        // header of the app
        TopBottomPanel::top("header").frame(
            Frame {
                inner_margin: Margin::same(1.0),
                outer_margin: Margin::same(0.0),
                fill: ctx.style().visuals.panel_fill,
                ..Default::default()
            }).show(ctx, |ui| {
                let timer = self.timer;
                let screen = self.screen;

                let timer_str = format!("{} Seconds", timer);
                let screen_str = format!("Screen {}", screen);
                self.upper_panel_size=ui.available_size();

                self.check_minimization(frame);

                ui.horizontal(|ui| {
                    if ui.button("New Screenshot").clicked() {
                        self.window_size=frame.info().window_info.size;
                        self.window_pos=frame.info().window_info.position.unwrap();
                        self.screenshot_taken=true;

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
                        let screens=get_screens();
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
                        self.previous_drawing_mode=self.drawing_mode;
                        self.drawing_mode=None;
                        self.settings_dialog=true;
                    }
                });

            });

        });

// footer of the app
        TopBottomPanel::bottom("footer")
            .frame(
                Frame {
                    inner_margin: Margin::same(1.0),
                    outer_margin: Margin::same(0.0),
                    fill: ctx.style().visuals.panel_fill,
                    ..Default::default()
                }
            )
            .resizable(false)
            .show(ctx, |ui| {
                if self.show_image {
                    ui.horizontal(|ui| {
                        // rotate left
                        if ui.button("\u{27F3}").clicked() {
                            let result=self.screenshot.rotate_sx_90();
                            self.manage_errors(result);
                            self._convert_image();
                            self.show_image=true;
                        }

                        // rotate right
                        if ui.button("\u{27F2}").clicked() {
                            let result=self.screenshot.rotate_dx_90();
                            self.manage_errors(result);
                            self._convert_image();
                            self.show_image=true;
                        }

                        // crop
                        if ui.button("\u{2702}").clicked() {
                            self.toggle_drawing_mode(DrawingMode::Crop);
                            let result=self.screenshot.save_intermediate_image();
                            self.crop_screenshot_tmp=self.screenshot.clone();
                            self.manage_errors(result);
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
                            self.toggle_drawing_mode(DrawingMode::Erase);
                        }

                        // shapes
                        if ui.button("\u{2B55}").clicked() {
                            self.toggle_drawing_mode(DrawingMode::Shape);
                            let result=self.screenshot.save_intermediate_image();
                            self.manage_errors(result);
                        }

                        // text
                        if ui.button("\u{1F1F9}").clicked() {
                            self.toggle_drawing_mode(DrawingMode::Text);
                        }

                        if self.drawing_mode.is_some() {
                            // Color Picker, Size Picker for Brush, Highlight, Erase, Shapes, Text
                            ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                                //SIZE FOR ALL
                                if self.drawing_mode!=Some(DrawingMode::Crop) && self.drawing_mode!= Some(DrawingMode::Erase) && self.drawing_mode!=None{
                                    //with color picker
                                    let picker = ui.color_edit_button_srgb(&mut self.tool_color).clone();
                                    match self.drawing_mode {
                                        Some(DrawingMode::Paint) => {
                                            ui.add(Slider::new(&mut self.tool_size, 1.0..=50.0));
                                            if picker.clicked() {
                                                self.previous_drawing_mode=Some(DrawingMode::Paint);
                                                self.drawing_mode=Some(DrawingMode::Pause);
                                            }
                                        },
                                        Some(DrawingMode::Highlight) => {
                                            ui.add(Slider::new(&mut self.tool_size, 1.0..=50.0));
                                            if picker.clicked() {
                                                self.previous_drawing_mode=Some(DrawingMode::Highlight);
                                                self.drawing_mode=Some(DrawingMode::Pause);
                                            }
                                        },
                                        Some(DrawingMode::Shape) => {
                                            ui.add(Slider::new(&mut self.tool_size, 1.0..=50.0));
                                            if ui.button("\u{25AD}").clicked() { self.shape=Some(Shape::Rectangle); }
                                            if ui.button("\u{2B55}").clicked() { self.shape=Some(Shape::Circle); }
                                        },
                                        Some(DrawingMode::Text) => {
                                            ui.add(Slider::new(&mut self.tool_size, 1.0..=50.0));
                                            self.drawing_mode=Some(DrawingMode::Text);
                                        },
                                        Some(DrawingMode::Pause) => {
                                            if picker.clicked_elsewhere() {
                                                self.drawing_mode=self.previous_drawing_mode;
                                            }
                                        }
                                        _ => {}
                                    }
                                }else{
                                    //without color picker
                                    match self.drawing_mode {
                                        Some(DrawingMode::Erase) => {
                                            ui.add(Slider::new(&mut self.tool_size, 1.0..=50.0));
                                        },
                                        _=>{}
                                    }
                                }
                            });
                                
                        }


                    });

                }
            });
            CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
            ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                    let values_window=self.calculate_rect_image(ui.available_size(),ctx.used_size());
                    // text edit window
                    if self.text_edit_dialog {
                        //text edit window without titlebar
                        Window::new("TextEdit")
                            .default_pos(self.text_edit_dialog_position)
                            .title_bar(false)
                            .collapsible(false)
                            .resizable(false)
                            .default_width(values_window.3/4.0)
                            .movable(true)
                            .drag_bounds(Rect::from_min_size(Pos2::new(values_window.0,values_window.1),Vec2::new(values_window.2,values_window.3)))
                            .frame(
                                egui::Frame {
                                    fill: Color32::from_rgba_unmultiplied(0, 0, 0, 50),
                                    stroke: Stroke::new(1.0, Color32::WHITE),
                                    ..Default::default()
                                })
                            .show(ctx, |ui_window| {
                                let w=ui_window.add(
                                    TextEdit::multiline(&mut self.text)
                                        .font(egui::FontId::proportional(self.tool_size))
                                        .text_color(Color32::from_rgb(self.tool_color[0], self.tool_color[1], self.tool_color[2]))
                                        .frame(false)
                                );
                                let enter_pressed = ctx.input(|is| is.key_pressed(Key::Enter));
                                let shift_pressed = ctx.input(|is| is.modifiers.shift);
                                if enter_pressed && shift_pressed  {
                                    //add new line
                                    self.text = format!("{}\n", self.text);
                                } else if enter_pressed {
                                    self.text_edit_dialog=false;
                                    let textbox_pos = self.calculate_texture_coordinates(w.rect.left_top(),
                                                                                         ui.available_size(), ctx.used_size(),true).unwrap();
                                    println!("real pos: {:?}",textbox_pos);
                                    let x=self.tool_size/values_window.4;
                                    let y=self.tool_size/values_window.5;
                                    self.screenshot.draw_text(&self.text, textbox_pos.x.max(0.0), textbox_pos.y.max(0.0), self.tool_color, Scale{x,y});
                                    self._convert_image();

                                }
                                println!("position: {:?}", w.rect);
                                println!("available: {:?}, ctx: {:?}",ui.available_size(),ctx.used_size());
                                let textbox_pos=self.calculate_texture_coordinates(w.rect.left_top(), ui.available_size(), ctx.used_size(),true).unwrap();
                                println!("real pos: {:?}",textbox_pos);
                            });
                    }
                    if self.show_image {
                        let available=ui.available_size();
                        let mut my_image = MyImage::new();
                        my_image.ui_resize(ui, self.color_image.clone());
                        // drawing
                        match self.drawing_mode {
                            Some(DrawingMode::Paint) => {
                                match self.draw_paint(ctx, available, self.tool_size, [self.tool_color[0], self.tool_color[1], self.tool_color[2], 255]){
                                    true=>{
                                        ctx.set_cursor_icon(CursorIcon::Crosshair);
                                    },
                                    false=>{
                                        ctx.set_cursor_icon(CursorIcon::Default);
                                    }
                                }
                            }
                            Some(DrawingMode::Highlight) => {
                                match self.draw_highlight(ctx, available, self.tool_size, self.tool_color)  {
                                    true=>{
                                        ctx.set_cursor_icon(CursorIcon::VerticalText);
                                    },
                                    false=>{
                                        ctx.set_cursor_icon(CursorIcon::Default);
                                    }
                                }
                            }
                            Some(DrawingMode::Erase) => {
                                match self.erase(ctx, available, self.tool_size) {
                                    true=>{
                                        ctx.set_cursor_icon(CursorIcon::NotAllowed);
                                    },
                                    false=>{
                                        ctx.set_cursor_icon(CursorIcon::Default);
                                    }
                                }
                            }
                            Some(DrawingMode::Shape) => {
                                match self.shape {
                                    Some(Shape::Rectangle) => {
                                        self.draw_rectangle(ctx, available, self.tool_size, [self.tool_color[0],self.tool_color[1],self.tool_color[2], 255]);
                                        
                                    },
                                    Some(Shape::Circle) => {
                                        self.draw_circle(ctx, available, self.tool_size, [self.tool_color[0],self.tool_color[1],self.tool_color[2], 255]);
                                    },
                                    _ => {}
                                }
                           }
                            Some(DrawingMode::Crop)=>{
                                let coordinates= self.draw_rectangle(ctx, available,2.0,[255,255,255,255]);
                                if coordinates.is_some() {
                                    let coordinates=coordinates.unwrap();
                                    let height = (coordinates.0.1 - coordinates.1.1).abs() as i32;
                                    let width = (coordinates.0.0 - coordinates.1.0).abs() as i32;
                                    let min_x=cmp::min(coordinates.0.0 as u32,coordinates.1.0 as u32);
                                    let min_y=cmp::min(coordinates.0.1 as u32,coordinates.1.1 as u32);
                                    let result=self.screenshot.resize_image(min_x+2, min_y+2, height-4, width-4);
                                    if self.manage_errors(result).is_none(){
                                        self.screenshot=self.crop_screenshot_tmp.clone();
                                    }
                                    self._convert_image();
                                }
                            },
                            Some(DrawingMode::Text)=>{
                                ctx.input(|ui| {
                                    if ui.pointer.any_down() {
                                        self.text_edit_dialog_position = ui.pointer.interact_pos().unwrap();
                                        self.text_edit_dialog = true;
                                    }
                                });
                            },
                            _ => {}
                        }
                    }
                });
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
