pub mod screenshot_module{
    use std::borrow::Cow;
    use std::error::Error;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;
    use arboard::{Clipboard, ImageData};
    use chrono::Local;
    use image::{DynamicImage, ImageFormat, RgbaImage};
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
        screenshot:DynamicImage,
    }
    impl Screenshot {
        pub fn new_empty() -> Screenshot {
            Screenshot{
                screenshot:DynamicImage::new_rgba8(0,0),
            }
        }
        pub fn new(screen:Screen) -> Result<Screenshot,Box<dyn Error>> {
            let image_captured=screen.capture()?;
            let width=image_captured.width();
            let height=image_captured.height();
            let image_rgba=image_captured.rgba().to_owned();
            let rgba_image=RgbaImage::from_raw(width,height,image_rgba).unwrap();
            let image_obj=DynamicImage::from(rgba_image);
            Ok(
                Screenshot{
                    screenshot:image_obj,
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
        pub fn get_image(&self)->Result<Vec<u8>,Box<dyn Error>>{
            let image_buffer = self.screenshot.to_rgba8();
            Ok(image_buffer.to_vec())
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
    }
}