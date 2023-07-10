pub mod screenshot_module{
    use std::error::Error;
    use std::path::{PathBuf};
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use screenshots::{Screen};
    use show_image::create_window;

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
            Result::Ok(
                Screenshot{
                    screenshot:image_obj,
                }
            )
        }
        pub fn save_image(&self,path:PathBuf,format:ImageFormat)->Result<(),Box<dyn Error>>{
            self.screenshot.save_with_format(path,format)?;
            return Ok(());
        }
    }
}