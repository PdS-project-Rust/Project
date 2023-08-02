mod screenshots_module;
mod hotkey_module;
mod api_module;
mod settings_module;

use eframe::{egui::{CentralPanel, Layout, Align, TextEdit, Direction, DragValue, Key}, egui::{Window, ComboBox, TopBottomPanel, self}, App, NativeOptions, epaint::{ColorImage, Vec2, Pos2}};
use crate::api_module::api_module as api_mod;
use crate::hotkey_module::hotkey_module::HotkeyManager;
use std::path::PathBuf;
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
        println!("size: {:?}",available);
        let w = image.width() as f32;
        let h = image.height() as f32;
        let w_window = available.x;
        let h_window = available.y;
        //gives the min between the height of the window and the height of the image scaled to the width of the window
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shapes {
    Rectangle,
    Circle,
    //Arrow,
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
    shape_mode:bool,
    text_mode: bool,
    text_color: [u8;3],
    text_size: f32,
    text_edit_dialog: bool,
    text_edit_dialog_position: Pos2,
    text: String,
    shape:Option<Shapes>,
    settings_dialog:bool,
    settings:Settings,
    instant:Instant,
    starting_point:Option<(f32, f32)>,
    brush_color: [u8;3],
    brush_size: f32,
    highlighter_size: f32,
    eraser_size: f32,
    upper_panel_size:Vec2,
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
            shape_mode:false,
            text_mode: false,
            text_color: [255, 0, 0],
            text_size: 16.0,
            text_edit_dialog: false,
            text_edit_dialog_position: Pos2::new(0.0,0.0),
            text: String::new(),        
            shape:None,
            settings_dialog:false,
            settings:Settings::default(),
            instant:Instant::now(),
            starting_point:None,
            brush_color: [255, 0, 0],
            brush_size: 6.0,
            highlighter_size: 18.0,
            eraser_size: 16.0,
            upper_panel_size:Vec2::new(0.0,0.0),
         }
    }
}

impl ScreenshotStr {
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

    fn calculate_texture_coordinates(&self, cursor_pos: Pos2, available: Vec2, total_window:Vec2) -> Pos2 {
        let w = self.screenshot.get_width().unwrap() as f32;
        let h = self.screenshot.get_height().unwrap() as f32;
        println!("size window: {:?}",available);
        //x is 640 out of 640, y is 356 out of 400 (-22*2 equal to borders+top and bottom)
        let w_window = available.x;
        let h_window = available.y;
        let height = h_window.min(w_window * h / w);
        println!("height scaled: {}, other: {}",(w_window * h / w),h_window);
        println!("height: {}",height);
        println!("{}",self.upper_panel_size.y);
        let width = height * w / h;
        let h_scale = height / h;
        let w_scale = width / w;
        let image_pos_x = (total_window.x - width) / 2.0;
        println!("{:?}",Margin::same(1.0).sum());
        println!("cursor pos: {:?}",cursor_pos);
        let image_pos_y = self.upper_panel_size.y + Margin::same(1.0).sum().y +(h_window-height)/2.0;
        let image_cursor_pos = Pos2 {
            x: (cursor_pos.x - image_pos_x)/w_scale,
            y: (cursor_pos.y - image_pos_y)/h_scale,
        };
        image_cursor_pos
    }

    fn draw_paint(&mut self, ctx: &egui::Context,available: Vec2, size: f32, color: [u8;4]) {
        ctx.input(|is| {
            let pos = is.pointer.interact_pos();
            if let Some(pos) = pos {
                println!("coordinates from Pos2: x {}, y {}",pos.x,pos.y);
                let texture_coordinates = self.calculate_texture_coordinates(pos, available,ctx.used_size());
                let x = texture_coordinates.x;
                let y = texture_coordinates.y;
                println!("coordinates from function: x {}, y {}",x,y);
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

    fn draw_highlight(&mut self, ctx: &egui::Context,available: Vec2, size: f32) {
        ctx.input(|is| {
            let pos = is.pointer.interact_pos();
            if let Some(pos) = pos {
                println!("coordinates from Pos2: x {}, y {}",pos.x,pos.y);
                let texture_coordinates = self.calculate_texture_coordinates(pos, available,ctx.used_size());
                let x = texture_coordinates.x;
                let y = texture_coordinates.y;
                println!("coordinates from function: x {}, y {}",x,y);
                if is.pointer.any_down() {
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
            }else{
                self.starting_point = None;
            }
        });
    }
    fn erase(&mut self, ctx: &egui::Context, available: Vec2, size: f32) {
        ctx.input(|ui| {
            let pos = ui.pointer.interact_pos();
            if let Some(pos) = pos {
                let texture_coordinates = self.calculate_texture_coordinates(pos, available, ctx.used_size());
                let x = texture_coordinates.x;
                let y = texture_coordinates.y;

                if ui.pointer.any_down() {
                    self.screenshot.erase_point(x, y, size);
                    if Instant::now() > self.instant {
                        self._convert_image();
                        self.instant += Duration::from_millis(16);
                    }
                }
            }
        });
    }

}

impl App for ScreenshotStr {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                        .font(egui::FontId::proportional(self.text_size))
                        .text_color(Color32::from_rgb(self.text_color[0], self.text_color[1], self.text_color[2]))
                        .frame(false)
                    );

                    println!("textedit size: {:?}", ui.available_size());
                    let enter_pressed = ctx.input(|is| is.key_pressed(Key::Enter));
                    let shift_pressed = ctx.input(|is| is.modifiers.shift);
                    if enter_pressed && shift_pressed  {
                        //add new line
                        self.text = format!("{}\n", self.text);
                    } else if enter_pressed {
                        self.text_edit_dialog=false;
                        let textbox_pos = self.text_edit_dialog_position;
                        self.screenshot.draw_text(&self.text, textbox_pos.x, textbox_pos.y, self.text_color, self.text_size);
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
            }
        )
            .show(ctx, |ui| {
            let timer = self.timer;
            let screen = self.screen;

            let timer_str = format!("{} Seconds", timer);
            let screen_str = format!("Screen {}", screen);
            self.upper_panel_size=ui.available_size();
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
                        if self.drawing_mode == Some(DrawingMode::Paint){
                            //color picker
                            ui.color_edit_button_srgb(&mut self.brush_color);
                            //brush size
                            ui.add(Slider::new(&mut self.brush_size, 1.0..=35.0).text("Size"));
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
                            self.shape_mode = !self.shape_mode;
                        }
                        if self.shape_mode {
                            //chose shape
                            ComboBox::from_label("Shape")
                                .selected_text("Shape")
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.shape, Some(Shapes::Rectangle), "Rectangle");
                                    ui.selectable_value(&mut self.shape, Some(Shapes::Circle), "Circle");
                                });
                        }

                        // text
                        if ui.button("\u{1F1F9}").clicked() {
                            self.text_mode = !self.text_mode;
                        }
                        if self.text_mode {
                            //chose size with drag value
                            ui.add(DragValue::new(&mut self.text_size).speed(1.0).clamp_range(1.0..=100.0));
                            
                            //chose color
                            ui.color_edit_button_srgb(&mut self.text_color);
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
                                ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
                                let brush_color_rgba = [self.brush_color[0], self.brush_color[1], self.brush_color[2], 255];
                                self.draw_paint(ctx, available, self.brush_size, brush_color_rgba);
                            }
                            Some(DrawingMode::Highlight) => {
                                ctx.set_cursor_icon(egui::CursorIcon::VerticalText);
                                self.draw_highlight(ctx, available, self.highlighter_size);
                            }
                            Some(DrawingMode::Erase) => {
                                self.erase(ctx, available,self.eraser_size);
                            }
                            _ => {}
                        }
                    }

                    // if text mode is active and if image is clicked open text dialog
                    if self.text_mode && self.show_image {
                        ctx.input(|ui| {
                            if ui.pointer.any_down() && !self.text_edit_dialog {
                                self.text_edit_dialog = true;
                                self.text_edit_dialog_position = ui.pointer.interact_pos().unwrap();
                                println!("text edit dialog opened");
                                println!("text edit dialog position: {:?}", self.text_edit_dialog_position);
                            }
                        });
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
