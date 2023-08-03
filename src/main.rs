mod screenshots_module;
mod hotkey_module;
mod api_module;
mod settings_module;

use eframe::{egui::{CentralPanel, Layout, Align, TextEdit, Direction, Key, Context, Window, ComboBox, TopBottomPanel, self, CursorIcon}, App, NativeOptions, epaint::{ColorImage, Vec2, Pos2}};
use crate::api_module::api_module as api_mod;
use crate::hotkey_module::hotkey_module::HotkeyManager;
use std::{cmp, path::PathBuf, thread};
use std::time::{Duration, Instant};
use eframe::egui::{Color32, Frame, Margin, Slider};
use eframe::epaint::Stroke;
use global_hotkey::GlobalHotKeyEvent;
use global_hotkey::hotkey::Modifiers;
use image::{EncodableLayout, ImageFormat};
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
        // println!("size: {:?}",available);
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DrawingMode {
    Paint,
    Highlight,
    Erase,
    Shape,
    Text,
    Pause,
    Crop
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    Rectangle,
    Circle
}

struct ScreenshotStr {
    timer:usize,
    screen:usize,
    screenshot:Screenshot,
    format:ImageFormat,
    color_image:ColorImage,
    show_image:bool,
    save_dialog:bool,
    drawing_mode:Option<DrawingMode>,
    previous_drawing_mode:Option<DrawingMode>,
    text_edit_dialog: bool,
    text_edit_dialog_position: Pos2,
    text: String,
    shape:Option<Shape>,
    tool_color:[u8;3],
    tool_size:f32,
    settings_dialog:bool,
    settings:Settings,
    instant:Instant,
    starting_point:Option<(f32, f32)>,
    upper_panel_size:Vec2,
    test:bool,
}

impl Default for ScreenshotStr {
    fn default() -> Self {
        Self {
            timer:0,
            screen:0,
            screenshot:Screenshot::new_empty(),
            format:ImageFormat::Png,
            color_image:ColorImage::example(),
            show_image:false,
            save_dialog:false,
            drawing_mode:None,
            previous_drawing_mode:Some(DrawingMode::Pause),
            text_edit_dialog: false,
            text_edit_dialog_position: Pos2::new(0.0,0.0),
            text: String::new(),
            shape:Some(Shape::Rectangle),
            tool_color:[255,0,0],
            tool_size:10.0,
            settings_dialog:false,
            settings:Settings::default(),
            instant:Instant::now(),
            starting_point:None,
            upper_panel_size:Vec2::new(0.0,0.0),
            test:false,
         }
    }
}

impl ScreenshotStr {
    //front
    pub fn toggle_drawing_mode(&mut self, mode: DrawingMode) {
        if self.drawing_mode == Some(mode) {
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

    fn calculate_texture_coordinates(&self, cursor_pos: Pos2, available: Vec2, total_window:Vec2) -> Option<Pos2> {
        let w = self.screenshot.get_width().unwrap() as f32;
        let h = self.screenshot.get_height().unwrap() as f32;
        //println!("size window: {:?}",available);
        //x is 640 out of 640, y is 356 out of 400 (-22*2 equal to borders+top and bottom)
        let w_window = available.x;
        let h_window = available.y;
        let height = h_window.min(w_window * h / w);
        //println!("height scaled: {}, other: {}",(w_window * h / w),h_window);
        //println!("height: {}",height);
        //println!("{}",self.upper_panel_size.y);
        let width = height * w / h;
        let h_scale = height / h;
        let w_scale = width / w;
        let image_pos_x = (total_window.x - width) / 2.0;
        //println!("{:?}",Margin::same(1.0).sum());
        //println!("cursor pos: {:?}",cursor_pos);
        let image_pos_y = self.upper_panel_size.y + Margin::same(1.0).sum().y +(h_window-height)/2.0;
        let image_cursor_pos = Pos2 {
            x: (cursor_pos.x - image_pos_x)/w_scale,
            y: (cursor_pos.y - image_pos_y)/h_scale,
        };
        if image_cursor_pos.x>w || image_cursor_pos.y>h || image_cursor_pos.y < 0.0 || image_cursor_pos.x < 0.0 {
            None
        }else{
            Some(image_cursor_pos)
        }
    }

    fn draw_paint(&mut self, ctx: &egui::Context,available: Vec2, size: f32, color: [u8;4]) -> bool {
        ctx.input(|is| -> bool {
            let pos = is.pointer.interact_pos();
            if let Some(pos) = pos {
                // println!("coordinates from Pos2: x {}, y {}",pos.x,pos.y);
                let texture_coordinates = self.calculate_texture_coordinates(pos, available,ctx.used_size());
                if texture_coordinates.is_some() {
                    let texture_coordinates=texture_coordinates.unwrap();
                    let x = texture_coordinates.x;
                    let y = texture_coordinates.y;
                    // println!("coordinates from function: x {}, y {}",x,y);
                    if is.pointer.any_down() {
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
                            self.conversion();
                        }
                    } else {
                        self.starting_point = None;
                    }
                    return true;
                } else {
                    self.starting_point = None;
                }
            }
            return false;
        })
    }

    fn draw_highlight(&mut self, ctx: &egui::Context,available: Vec2, size: f32, color: [u8;3]) -> bool {
        ctx.input(|is| -> bool {
            let pos = is.pointer.interact_pos();
            if let Some(pos) = pos {
                //println!("coordinates from Pos2: x {}, y {}",pos.x,pos.y);
                let texture_coordinates = self.calculate_texture_coordinates(pos, available,ctx.used_size());
                if texture_coordinates.is_some() {
                    let texture_coordinates=texture_coordinates.unwrap();
                    let x = texture_coordinates.x;
                    let y = texture_coordinates.y;
                    //println!("coordinates from function: x {}, y {}",x,y);
                    if is.pointer.any_down() {
                        if self.starting_point.is_none() {
                            self.starting_point = Some((x, y));
                        } else {
                            self.screenshot.highlight_line(
                                self.starting_point.unwrap(),
                                (x, y),
                                size,
                                color
                            );
                            let mut dx = 1.0;
                            if self.starting_point.unwrap().0 > x { dx = -1.0 }
                            self.starting_point = Some((x+dx, y));
                            self.conversion();
                        }
                    } else {
                        self.starting_point = None;
                    }
                    return true;
                } else {
                    self.starting_point = None;
                }
            }else{
                self.starting_point = None;
            }
            return false;
        })
    }

    fn erase(&mut self, ctx: &egui::Context, available: Vec2, size: f32) -> bool{
        ctx.input(|ui| -> bool{
            let pos = ui.pointer.interact_pos();
            if let Some(pos) = pos {
                let texture_coordinates = self.calculate_texture_coordinates(pos, available, ctx.used_size());
                if texture_coordinates.is_some(){
                    let texture_coordinates=texture_coordinates.unwrap();
                    let x = texture_coordinates.x;
                    let y = texture_coordinates.y;

                    if ui.pointer.any_down() {
                        self.screenshot.erase_point(x, y, size);
                        self.conversion();
                    }
                    return true;
                }
            }
            self.starting_point = None;
            return false;
        })
    }

    fn draw_rectangle(&mut self, ctx: &Context, available: Vec2, size: f32, color: [u8;4]) -> Option<((f32,f32),(f32,f32))> {
        return ctx.input(|is| {
            let pos = is.pointer.interact_pos();
            if let Some(pos) = pos {
                let texture_coordinates = self.calculate_texture_coordinates(pos, available, ctx.used_size());
                if let Some(texture_coordinates) = texture_coordinates {
                    let x = texture_coordinates.x;
                    let y = texture_coordinates.y;
                    if is.pointer.any_down() {
                        if self.starting_point.is_none() {
                            self.starting_point = Some((x, y));
                        } else {
                            let start = (
                                self.starting_point.unwrap().0,
                                self.starting_point.unwrap().1,
                            );
                            let end = (x, y);
                            self.screenshot.rectangle(start, end, size, color);
                            self.conversion();
                            return Option::None;
                        }
                    } else {
                        if self.starting_point.is_some() {
                            let start = (
                                self.starting_point.unwrap().0,
                                self.starting_point.unwrap().1,
                            );
                            let end = (x, y);
                            self.screenshot.rectangle(start, end, size, color);
                            self.conversion();
                            let tmp=self.starting_point.take().unwrap();
                            self.screenshot.save_intermediate_image().unwrap();
                            return Option::Some((tmp,(x,y)));
                        }
                        return None;
                    }
                } else {
                    self.starting_point = None;
                }
            }
            return Option::None;
        });
    }

    fn draw_circle(&mut self, ctx: &Context, available: Vec2, size: f32, color: [u8;4]) {
        ctx.input(|is| {
            let pos = is.pointer.interact_pos();
            if let Some(pos) = pos {
                let texture_coordinates = self.calculate_texture_coordinates(pos, available, ctx.used_size());
                if let Some(texture_coordinates) = texture_coordinates {
                    let x = texture_coordinates.x;
                    let y = texture_coordinates.y;
                    if is.pointer.any_down() {
                        if self.starting_point.is_none() {
                            self.starting_point = Some((x, y));
                        } else {
                            let start = (
                                self.starting_point.unwrap().0,
                                self.starting_point.unwrap().1,
                            );
                            let end = (x, y);
                            self.screenshot.circle(start, end, size, color);
                            self.conversion();
                        }
                    } else {
                        if self.starting_point.is_some() {
                            let start = (
                                self.starting_point.unwrap().0,
                                self.starting_point.unwrap().1,
                            );
                            let end = (x, y);
                            self.screenshot.circle(start, end, size, color);
                            self.conversion();
                        }
                        self.starting_point = None;
                        self.screenshot.save_intermediate_image().unwrap();
                    }
                } else {
                    self.starting_point = None;
                }
            }
        });
    }
    fn conversion(&mut self) {
        if Instant::now() > self.instant {
            self._convert_image();
            self.instant += Duration::from_millis(16);
        }
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

        // text edit window
        if self.text_edit_dialog {
            //text edit window without titlebar
            Window::new("TextEdit")
                .default_pos(self.text_edit_dialog_position)
                .title_bar(false)
                .collapsible(false)
                .resizable(true)
                //slightly transparent but with borders
                .frame(
                    egui::Frame {
                        fill: Color32::TRANSPARENT,
                        stroke: Stroke::new(1.0, Color32::WHITE),
                        ..Default::default()
                    })
                .show(ctx, |ui| {
                    ui.add(
                        TextEdit::multiline(&mut self.text)
                        .font(egui::FontId::proportional(self.tool_size))
                        .text_color(Color32::from_rgb(self.tool_color[0], self.tool_color[1], self.tool_color[2]))
                        .frame(false)
                    );

                    // println!("textedit size: {:?}", ui.available_size());
                    let enter_pressed = ctx.input(|is| is.key_pressed(Key::Enter));
                    let shift_pressed = ctx.input(|is| is.modifiers.shift);
                    if enter_pressed && shift_pressed  {
                        //add new line
                        self.text = format!("{}\n", self.text);
                    } else if enter_pressed {
                        self.text_edit_dialog=false;
                        let textbox_pos = self.text_edit_dialog_position;
                        self.screenshot.draw_text(&self.text, textbox_pos.x, textbox_pos.y, self.tool_color, self.tool_size);
                        self._convert_image();

                    }

                });

        }
        
        // header of the app
        TopBottomPanel::top("header").frame(
            egui::Frame {
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
                if self.test {
                    let duration = Duration::from_secs(self.timer as u64);
                    if frame.info().window_info.focused {
                        // println!("now minimized");
                        self.screenshot=api_mod::take_screenshot(duration,self.screen);
                        self._convert_image();
                        self.show_image=true;
                        frame.set_minimized(false);
                        self.test=false;
                    
                    } else {
                        // println!("not minimized");
                        thread::sleep(Duration::from_millis(10));
                    }
                }


                ui.horizontal(|ui| {
                    if ui.button("New Screenshot").clicked() {
                        frame.set_minimized(true);
                        self.test = true;
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

// footer of the app
        TopBottomPanel::bottom("footer")
            .frame(
                egui::Frame {
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
                            self.screenshot.rotate_sx_90().unwrap();
                            self._convert_image();
                            self.show_image=true;
                        }

                        // rotate right
                        if ui.button("\u{27F2}").clicked() {
                            self.screenshot.rotate_dx_90().unwrap();
                            self._convert_image();
                            self.show_image=true;
                        }

                        // crop
                        if ui.button("\u{2702}").clicked() {
                            self.toggle_drawing_mode(DrawingMode::Crop);
                            self.screenshot.save_intermediate_image().unwrap();
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
                            self.screenshot.save_intermediate_image().unwrap();
                        }

                        // text
                        if ui.button("\u{1F1F9}").clicked() {
                            self.toggle_drawing_mode(DrawingMode::Text);
                        }

                        if self.drawing_mode.is_some() {
                            // Color Picker, Size Picker for Brush, Highlight, Erase, Shapes, Text
                            ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                                //SIZE FOR ALL
                                if self.drawing_mode!=Some(DrawingMode::Crop) && self.drawing_mode!= Some(DrawingMode::Erase) {
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
                                            ui.color_edit_button_srgb(&mut self.tool_color);
                                        },
                                        Some(DrawingMode::Pause) => {
                                            if picker.clicked_elsewhere() {
                                                println!("before dm: {:?}", self.drawing_mode);
                                                self.drawing_mode=self.previous_drawing_mode.clone();
                                                println!("after Drawing Mode: {:?}", self.drawing_mode);
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
                                        Some(DrawingMode::Crop)=>{

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
                                        ctx.set_cursor_icon(egui::CursorIcon::VerticalText);
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
                                let coordinates=self.draw_rectangle(ctx,available,2.0,[255,255,255,255]);
                                if coordinates.is_some(){
                                    let coordinates=coordinates.unwrap();
                                    let min_x=cmp::min(coordinates.0.0 as u32,coordinates.1.0 as u32);
                                    let min_y=cmp::min(coordinates.0.1 as u32,coordinates.1.1 as u32);
                                    self.screenshot.resize_image(min_x+2, min_y+2, (coordinates.0.1 - coordinates.1.1).abs() as i32 -4, (coordinates.0.0 - coordinates.1.0).abs() as i32 -4).unwrap();
                                    self._convert_image();
                                }
                            }
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
