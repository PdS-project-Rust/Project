pub mod screenshot_module{
    use std::borrow::Cow;
    use std::error::Error;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;
    use arboard::{Clipboard, ImageData};
    use chrono::Local;
    use eframe::egui::Pos2;
    use image::{DynamicImage, GenericImage, GenericImageView, ImageFormat, Rgba, RgbaImage, ImageBuffer, imageops::overlay};
    use imageproc::drawing::{Blend, draw_antialiased_line_segment_mut, draw_line_segment_mut};
    use screenshots::Screen;
    use thiserror::Error;

    #[derive(Error,Debug)]
    enum ScreenShotError{
        #[error("sizes are not compatible")]
        ResizeSize,
        #[error("Path is not a dir")]
        PathError,
        #[error("extension error")]
        ExtensionError,
    }

    pub struct Screenshot {
        screenshot: DynamicImage,
    }

    impl Screenshot {
        pub fn new_empty() -> Screenshot {
            Screenshot{
                screenshot: DynamicImage::new_rgba8(0,0),
            }
        }

        pub fn new(screen:Screen) -> Result<Screenshot,Box<dyn Error>> {
            let image_captured=screen.capture()?;
            let width= image_captured.width();
            let height= image_captured.height();
            let image_rgba=image_captured.rgba().to_owned();
            let rgba_image=RgbaImage::from_raw(width,height,image_rgba).unwrap();
            let image_obj=DynamicImage::from(rgba_image);
            Ok(
                Screenshot{
                    screenshot: image_obj,
                }
            )
        }

        pub fn save_image(&self,path:&PathBuf,format:ImageFormat)->Result<(),Box<dyn Error>>{
            if path.is_dir()==false {
                return Err(Box::new(ScreenShotError::PathError))
            }
            let mut file_name ="screenshot-".to_string();
            file_name.push_str(Local::now().format("%d-%m-%Y-%H-%M-%S_%3f").to_string().as_str());
            let mut path_with_file_name =path.join(PathBuf::from(file_name));
            match format {
                ImageFormat::Png=>{
                    path_with_file_name = path_with_file_name.with_extension(PathBuf::from("png"));
                }
                ImageFormat::Gif=>{
                    path_with_file_name = path_with_file_name.with_extension(PathBuf::from("gif"));
                }
                ImageFormat::Jpeg=>{
                    path_with_file_name = path_with_file_name.with_extension(PathBuf::from("jpg"));
                }
                _=>{
                    return Err(Box::new(ScreenShotError::ExtensionError));
                }
            }
            self.screenshot.save_with_format(path_with_file_name, format)?;
            return Ok(());
        }

        pub fn save_to_clipboard(&self)->Result<(),Box<dyn Error>>{
            let mut clipboard=Clipboard::new()?;
            clipboard.set_image(ImageData{
                width: self.screenshot.width() as usize,
                height: self.screenshot.height() as usize,
                bytes: Cow::from(self.screenshot.as_bytes()),
            })?;
            Ok(())
        }

        pub fn get_image(&self)->Result<DynamicImage,Box<dyn Error>>{
            Ok(self.screenshot.clone())
        }

        pub fn resize_image(&mut self, x:u32, y:u32, height: u32, width: u32) -> Result<(),Box<dyn Error>>{
            if self.screenshot.width()<(x+width)||self.screenshot.height()<(y+height) {
                return Err(Box::new(ScreenShotError::ResizeSize));
            }
            self.screenshot=self.screenshot.crop(x,y,width,height);
            Ok(())
        }
        pub fn get_width(&self)->Result<u32,Box<dyn Error>>{
            return Ok(self.screenshot.width());
        }

        pub fn get_height(&self)->Result<u32,Box<dyn Error>>{
            return Ok(self.screenshot.height());
        }

        pub fn rotate_sx_90(&mut self)->Result<(),Box<dyn Error>>{
            self.screenshot=self.screenshot.rotate90();
            Ok(())
        }
        pub fn rotate_dx_90(&mut self)->Result<(),Box<dyn Error>>{
            self.screenshot=self.screenshot.rotate270();
            Ok(())
        }

        pub fn screenshot_after_delay(duration:Duration, screen:Screen) -> Result<Screenshot, Box<dyn Error>> {
            thread::sleep(duration);
            Screenshot::new(screen)
        }

        pub fn draw_point(&mut self, x: i32, y: i32, r: u32, color: [u8;4]) {
            let width = self.screenshot.width() as i32;
            let height = self.screenshot.height() as i32;
            let r = r as i32;
            if x > 0 && x < width && y > 0 && y < height {
                // println!("{}{}",x,y);
                for i in (x - r)..=(x + r) {
                    for j in (y - r)..=(y + r) {
                        // Full circles
                        if ((x-i)*(x-i) + (y-j)*(y-j)) <= r {
                            if i >= 0 && i < width && j >= 0 && j < height {
                                let color_pixel = Rgba(color);
                                self.screenshot.put_pixel(i as u32, j as u32, color_pixel);
                            }
                        }
                    }
                }
            }
        }

        pub fn draw_line(&mut self, starting_point: (f32, f32), ending_point: (f32, f32), color: [u8; 4], size: f32) {
            let color_pixel = Rgba::from(color);
            // main line
            draw_line_segment_mut(&mut self.screenshot, starting_point, ending_point, color_pixel);
            // additional lines to simulate the brush size
            let num_lines = (size + 0.5) as i32; // Round to the nearest integer
            for i in 1..=num_lines {
                let offset = i as f32;
                draw_line_segment_mut(&mut self.screenshot,
                                      (starting_point.0 - offset, starting_point.1 - offset),
                                      (ending_point.0 - offset, ending_point.1 - offset),
                                      color_pixel
                );
            }
        }

        pub fn highlight(&mut self, starting_point: (f32, f32), ending_point: (f32, f32), size: f32) {
            // yellow semi-transparent
            let color_pixel = Rgba::from([255, 255, 0, 16]);
            // draw horizontal stripes
            draw_line_segment_mut(&mut self.screenshot, starting_point, ending_point, color_pixel);
            let num_lines = (size + 0.5) as i32; // Round to the nearest integer
            for i in 1..=num_lines {
                let offset = i as f32;
                draw_line_segment_mut(&mut self.screenshot,
                                      (starting_point.0, starting_point.1 - offset),
                                      (ending_point.0, ending_point.1 - offset),
                                      color_pixel
                );
            }
            // create an overlay image with the same size as the screenshot
            let mut overlay_image = RgbaImage::new(self.screenshot.width(), self.screenshot.height());
            // draw the highlighted stripes on the overlay image
            draw_line_segment_mut(&mut overlay_image, starting_point, ending_point, color_pixel);
            for i in 1..=num_lines {
                let offset = i as f32;
                draw_line_segment_mut(&mut overlay_image,
                                      (starting_point.0, starting_point.1 - offset),
                                      (ending_point.0, ending_point.1 - offset),
                                      color_pixel
                );
            }
            // blend the overlay image with the screenshot
            overlay(&mut self.screenshot, &overlay_image, 0, 0);
        }

    }
}