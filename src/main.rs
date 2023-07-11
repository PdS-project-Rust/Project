mod screenshots_module;
use std::path::PathBuf;
use std::time::Duration;
use image::ImageFormat;
use screenshots::Screen;
use crate::screenshots_module::screenshot_module::Screenshot;

fn main() {
    let screens=Screen::all().unwrap();
    let path=PathBuf::from(r"C:\Users\miche\Downloads".to_string());
    for i in screens{
        let mut ss=Screenshot::new(i).unwrap();
        println!("{:?}",i.display_info);
        ss.resize_image(0,ss.get_height().unwrap()/2-1,ss.get_height().unwrap()/2,ss.get_width().unwrap()/2).unwrap();
        ss.rotate_dx_90().unwrap();
        ss.save_to_clipboard().unwrap();
        ss.save_image(&path,ImageFormat::Jpeg).unwrap();
    }
    let ss1=Screenshot::screenshot_after_delay(Duration::from_secs(3),Screen::all().unwrap()[0]).unwrap();
    ss1.save_image(&path,ImageFormat::Jpeg).unwrap();
}
