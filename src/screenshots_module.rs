pub mod screenshot_module{
    use std::error::Error;
    use std::path::{PathBuf};
    use chrono::Local;
    use image::{DynamicImage, ImageFormat, RgbaImage};
    use screenshots::{Screen};
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
        pub fn save_image(&self,path:PathBuf,format:ImageFormat)->Result<(),Box<dyn Error>>{
            if path.is_dir()==false {
                return Err(Box::new(ScreenShotError::PathError))
            }
            let mut string_path_with_file_name ="screenshot-".to_string();
            string_path_with_file_name.push_str(Local::now().format("%d-%m-%Y-%H-%M-%S_%3f").to_string().as_str());
            let mut path_with_file_name =path.join(PathBuf::from(string_path_with_file_name));
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
        pub fn resize_image(&mut self, x:u32, y:u32, height: u32, width: u32) -> Result<(),Box<dyn Error>>{
            if self.screenshot.width()<(x+width)||self.screenshot.height()<(y+width) {
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
    }
}