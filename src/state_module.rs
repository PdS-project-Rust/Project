pub mod state_module{
    use std::error::Error;
    use eframe::{egui::{Context}, epaint::{ColorImage, Vec2, Pos2}};
    use std::time::{Duration, Instant};
    use eframe::egui::{Margin};
    use image::{EncodableLayout, ImageFormat};
    use crate::screenshots_module::screenshot_module::Screenshot;
    use crate::settings_module::settings_module::*;
    use screenshots::Screen;



    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum DrawingMode {
        Paint,
        Highlight,
        Erase,
        Shape,
        Text,
        Pause,
        Crop
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Shape {
        Rectangle,
        Circle
    }

    pub struct ScreenshotStr {
        pub timer:usize,
        pub screen:usize,
        pub screenshot:Screenshot,
        pub format:ImageFormat,
        pub color_image:ColorImage,
        pub show_image:bool,
        pub error_dialog:bool,
        pub error_message:String,
        pub save_dialog:bool,
        pub drawing_mode:Option<DrawingMode>,
        pub previous_drawing_mode:Option<DrawingMode>,
        pub text_edit_dialog: bool,
        pub text_edit_dialog_position: Pos2,
        pub text: String,
        pub shape:Option<Shape>,
        pub tool_color:[u8;3],
        pub tool_size:f32,
        pub settings_dialog:bool,
        pub settings:Settings,
        pub instant:Instant,
        pub starting_point:Option<(f32, f32)>,
        pub upper_panel_size:Vec2,
        pub screen_state:u8,
        pub screenshot_taken:bool,
        pub image_converted:bool,
        pub window_pos:Pos2,
        pub window_size:Vec2,
        pub crop_screenshot_tmp:Screenshot,
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
                error_dialog:false,
                error_message:String::new(),
                save_dialog:false,
                drawing_mode:None,
                previous_drawing_mode:Some(DrawingMode::Pause),
                text_edit_dialog: false,
                text_edit_dialog_position: Pos2::new(0.0,0.0),
                text: String::new(),
                shape:Some(Shape::Rectangle),
                tool_color:[0,0,0],
                tool_size:10.0,
                settings_dialog:false,
                settings:Settings::default(),
                instant:Instant::now(),
                starting_point:None,
                upper_panel_size:Vec2::new(0.0,0.0),
                screen_state:0,
                screenshot_taken:false,
                image_converted:false,
                window_pos:Pos2::new(0.0,0.0),
                window_size:Vec2::new(0.0,0.0),
                crop_screenshot_tmp:Screenshot::new_empty(),
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
            self.image_converted = true;
        }

        pub fn calculate_texture_coordinates(&self, cursor_pos: Pos2, available: Vec2, total_window:Vec2, return_always :bool) -> Option<Pos2> {
            let w = self.screenshot.get_width().unwrap() as f32;
            let h = self.screenshot.get_height().unwrap() as f32;
            //x is 640 out of 640, y is 356 out of 400 (-22*2 equal to borders+top and bottom)
            let w_window = available.x;
            let h_window = available.y;
            let height = h_window.min(w_window * h / w);
            let width = height * w / h;
            let h_scale = height / h;
            let w_scale = width / w;
            let image_pos_x = (total_window.x - width) / 2.0;
            let image_pos_y = self.upper_panel_size.y + Margin::same(1.0).sum().y +(h_window-height)/2.0;
            let image_cursor_pos = Pos2 {
                x: (cursor_pos.x - image_pos_x)/w_scale,
                y: (cursor_pos.y - image_pos_y)/h_scale,
            };
            if image_cursor_pos.x>w || image_cursor_pos.y>h || image_cursor_pos.y < 0.0 || image_cursor_pos.x < 0.0 {
                if !return_always {
                    None
                }else{
                    Some(image_cursor_pos)
                }
            }else{
                Some(image_cursor_pos)
            }
        }

        pub fn calculate_rect_image(&self, available: Vec2, total_window:Vec2)->(f32,f32,f32,f32,f32,f32){
            let w = self.screenshot.get_width().unwrap() as f32;
            let h = self.screenshot.get_height().unwrap() as f32;
            //x is 640 out of 640, y is 356 out of 400 (-22*2 equal to borders+top and bottom)
            let w_window = available.x;
            let h_window = available.y;
            let height = h_window.min(w_window * h / w);
            let width = height * w / h;
            let h_scale = height / h;
            let w_scale = width / w;
            let image_pos_x = (total_window.x - width) / 2.0;
            let image_pos_y = self.upper_panel_size.y + Margin::same(1.0).sum().y +(h_window-height)/2.0;
            return (image_pos_x,image_pos_y,width,height,w_scale,h_scale)
        }

        pub fn draw_paint(&mut self, ctx: &Context, available: Vec2, size: f32, color: [u8;4]) -> bool {
            ctx.input(|is| -> bool {
                let pos = is.pointer.interact_pos();
                if let Some(pos) = pos {
                    let texture_coordinates = self.calculate_texture_coordinates(pos, available,ctx.used_size(),false);
                    if texture_coordinates.is_some() {
                        let texture_coordinates=texture_coordinates.unwrap();
                        let x = texture_coordinates.x;
                        let y = texture_coordinates.y;
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

        pub fn draw_highlight(&mut self, ctx: &Context, available: Vec2, size: f32, color: [u8;3]) -> bool {
            ctx.input(|is| -> bool {
                let pos = is.pointer.interact_pos();
                if let Some(pos) = pos {
                    let texture_coordinates = self.calculate_texture_coordinates(pos, available,ctx.used_size(),false);
                    if texture_coordinates.is_some() {
                        let texture_coordinates=texture_coordinates.unwrap();
                        let x = texture_coordinates.x;
                        let y = texture_coordinates.y;
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

        pub fn erase(&mut self, ctx: &Context, available: Vec2, size: f32) -> bool{
            ctx.input(|ui| -> bool{
                let pos = ui.pointer.interact_pos();
                if let Some(pos) = pos {
                    let texture_coordinates = self.calculate_texture_coordinates(pos, available, ctx.used_size(),false);
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

        pub fn draw_rectangle(&mut self, ctx: &Context, available: Vec2, size: f32, color: [u8;4]) -> Option<((f32, f32), (f32, f32))> {
            return ctx.input(|is| {
                let pos = is.pointer.interact_pos();
                if let Some(pos) = pos {
                    let texture_coordinates = self.calculate_texture_coordinates(pos, available, ctx.used_size(),false);
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
                                return None;
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
                                return Some((tmp,(x,y)));
                            }
                            return None;
                        }
                    } else {
                        self.starting_point = None;
                    }
                }
                return None;
            });
        }

        pub fn draw_circle(&mut self, ctx: &Context, available: Vec2, size: f32, color: [u8;4]) {
            ctx.input(|is| {
                let pos = is.pointer.interact_pos();
                if let Some(pos) = pos {
                    let texture_coordinates = self.calculate_texture_coordinates(pos, available, ctx.used_size(),false);
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

        pub fn check_minimization(&mut self, frame: &mut eframe::Frame) {
            if self.screenshot_taken {
                match self.screen_state {
                    0 => {
                        frame.set_window_pos(Pos2::new(0.0,0.0));
                        frame.set_window_size(Vec2::new(0.0,0.0));
                        if frame.info().window_info.position.unwrap().x==0.0 && frame.info().window_info.position.unwrap().y==0.0 {
                            self.screen_state=1;
                        }
                        
                    },
                    1 => {
                        let duration = Duration::from_secs(self.timer as u64);
                        self.screenshot=take_screenshot(duration,self.screen);
                        self._convert_image();
                        self.show_image=true;
                        if self.image_converted {
                            self.screen_state=2;
                        }
                      
                    },
                    2 => {
                        frame.set_window_pos(self.window_pos);
                        frame.set_window_size(self.window_size);
                        
                        self.screen_state=0;
                        self.screenshot_taken=false;
                    },
                    _ => {

                    }
                }
            }
        }
        pub fn manage_errors <E>(&mut self,result: Result<E,Box<dyn Error>>)->Option<E>{
            match result {
                Ok(E)=>Some(E),
                Err(e)=> {
                    self.previous_drawing_mode=self.drawing_mode;
                    self.drawing_mode=None;
                    self.error_message = e.to_string();
                    self.error_dialog=true;
                    None
                }
            }
        }

    }

    pub fn get_screens() -> Vec<Screen> {
        let screens= Screen::all().unwrap();
        screens
    }

    pub fn take_screenshot(timer:Duration, screen:usize) -> Screenshot {
        let screens=Screen::all().unwrap();
        let screen=screens[screen].clone();
        //screenshot after delay
        let ss1=Screenshot::screenshot_after_delay(timer,screen).unwrap();
        //save image to clipboard
        ss1.save_to_clipboard().unwrap();
        ss1
    }
}