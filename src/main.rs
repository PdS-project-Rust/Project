mod screenshots_module;

use std::path::PathBuf;
use image::ImageFormat;
use crate::screenshots_module::screenshot_module::Screenshot;

fn main() {
    let screens=screenshots::Screen::all().unwrap();
    for i in screens{
        let mut ss=Screenshot::new(i).unwrap();
        let path=PathBuf::from(r"C:\Users\miche\Downloads".to_string());
        ss.resize_image(0,0,ss.get_height().unwrap()/2,ss.get_width().unwrap()/2).unwrap();
        ss.rotate_dx_90().unwrap();
        ss.save_image(path,ImageFormat::Jpeg).unwrap();
    }
}
