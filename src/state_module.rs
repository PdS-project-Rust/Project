pub mod state_module{
    use eframe::{egui::{Context}, egui, epaint::{ColorImage, Vec2, Pos2}};
    use std::time::{Duration, Instant};
    use eframe::egui::{Margin};
    use image::{EncodableLayout, ImageFormat};
    use crate::screenshots_module::screenshot_module::Screenshot;
    use crate::settings_module::settings_module::*;



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
        pub test:bool,
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

        pub fn draw_paint(&mut self, ctx: &egui::Context, available: Vec2, size: f32, color: [u8;4]) -> bool {
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

        pub fn draw_highlight(&mut self, ctx: &egui::Context, available: Vec2, size: f32, color: [u8;3]) -> bool {
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

        pub fn erase(&mut self, ctx: &egui::Context, available: Vec2, size: f32) -> bool{
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

        pub fn draw_rectangle(&mut self, ctx: &Context, available: Vec2, size: f32, color: [u8;4]) -> Option<((f32, f32), (f32, f32))> {
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

        pub fn draw_circle(&mut self, ctx: &Context, available: Vec2, size: f32, color: [u8;4]) {
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
}