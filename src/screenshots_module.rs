pub mod screenshot_module{
    use std::borrow::Cow;
    use std::error::Error;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;
    use arboard::{Clipboard, ImageData};
    use chrono::Local;
    use image::{DynamicImage, GenericImage, GenericImageView, ImageFormat, Rgba, RgbaImage};
    use imageproc::drawing::draw_line_segment_mut;
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
        original_image: DynamicImage,
    }

    impl Screenshot {
        pub fn new_empty() -> Screenshot {
            Screenshot{
                screenshot: DynamicImage::new_rgba8(0,0),
                original_image: DynamicImage::new_rgba8(0,0),
            }
        }

        pub fn new(screen:Screen) -> Result<Screenshot,Box<dyn Error>> {
            let image_captured=screen.capture()?;
            let width= image_captured.width();
            let height= image_captured.height();
            let image_rgba=image_captured.rgba().to_owned();
            let rgba_image=RgbaImage::from_raw(width,height,image_rgba).unwrap();
            let image_obj=DynamicImage::from(rgba_image);
            let original_obj=image_obj.clone();
            Ok(
                Screenshot{
                    screenshot: image_obj,
                    original_image: original_obj,
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

        pub fn draw_point(&mut self, x: f32, y: f32, r: f32, color: [u8;4]) {
            let width = self.screenshot.width() as i32;
            let height = self.screenshot.height() as i32;
            let x = x as i32;
            let y = y as i32;
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
            // calculate the direction vector of the line
            let dx = ending_point.0 - starting_point.0;
            let dy = ending_point.1 - starting_point.1;
            let length = (dx * dx + dy * dy).sqrt();
            // calculate the normalized perpendicular vector to the line
            let nx = dy / length;
            let ny = -dx / length;
            // calculate the step size for the brush strokes
            let step_size =  0.5;
            let thickness = 2 * (size + 0.5) as i32;
            for i in 0..thickness {
                // calculate the offset along the perpendicular vector
                let offset = (i as f32 - size) * step_size;
                // calculate the starting and ending points for each brush stroke
                let start_x = starting_point.0 + nx * offset;
                let start_y = starting_point.1 + ny * offset;
                let end_x = ending_point.0 + nx * offset;
                let end_y = ending_point.1 + ny * offset;
                // draw the brush stroke
                draw_line_segment_mut(&mut self.screenshot, (start_x, start_y), (end_x, end_y), color_pixel);
            }
        }

        pub fn highlight_line(&mut self, starting_point: (f32, f32), ending_point: (f32, f32), size: f32) {
            // yellow color with transparency
            let highlight_color = Rgba([255, 255, 0, 64]);
            // create a temporary overlay image with the same size as the screenshot
            let mut overlay_image = RgbaImage::new(self.screenshot.width(), self.screenshot.height());
            // draw the highlighted stripes on the overlay image
            draw_line_segment_mut(&mut overlay_image, starting_point, ending_point, highlight_color);
            for i in 1..=(size as i32) {
                let offset = i as f32;
                draw_line_segment_mut(&mut overlay_image,
                                      (starting_point.0, starting_point.1 - offset),
                                      (ending_point.0, ending_point.1 - offset),
                                      highlight_color
                );
            }
            // combine the overlay image with the screenshot using alpha blending
            let (width, height) = self.screenshot.dimensions();
            for y in 0..height {
                for x in 0..width {
                    let screenshot_pixel = self.screenshot.get_pixel(x, y);
                    let overlay_pixel = overlay_image.get_pixel(x, y);
                    // calculate the blended pixel color manually
                    let blended_pixel = Self::blend_colors(screenshot_pixel, *overlay_pixel);
                    self.screenshot.put_pixel(x, y, blended_pixel);
                }
            }
        }

        pub fn erase_point(&mut self, x: f32, y: f32, r: f32) {
            let width = self.screenshot.width() as i32;
            let height = self.screenshot.height() as i32;
            let (x, y) = (x as i32, y as i32);
            let r = r as i32;

            if x > 0 && x < width && y > 0 && y < height {
                for dx in -r..r {
                    for dy in -r..r {
                        let src_x = x + dx;
                        let src_y = y + dy;
                        if src_x >= 0 && src_x < width && src_y >= 0 && src_y < height {
                            let src_pixel = self.original_image.get_pixel(src_x as u32, src_y as u32);
                            if (dx*dx + dy*dy) <= r*r {
                                self.screenshot.put_pixel(src_x as u32, src_y as u32, src_pixel);
                            }
                        }
                    }
                }
            }
        }


        fn blend_colors(background: Rgba<u8>, foreground: Rgba<u8>) -> Rgba<u8> {
            let alpha = foreground[3] as f32 / 255.0;
            let inv_alpha = 1.0 - alpha;
            let r = (foreground[0] as f32 * alpha + background[0] as f32 * inv_alpha) as u8;
            let g = (foreground[1] as f32 * alpha + background[1] as f32 * inv_alpha) as u8;
            let b = (foreground[2] as f32 * alpha + background[2] as f32 * inv_alpha) as u8;
            let a = background[3];
            Rgba([r, g, b, a])
        }

    }
}