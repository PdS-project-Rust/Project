mod screenshots_module;

use std::path::PathBuf;
use image::ImageFormat;
use crate::screenshots_module::screenshot_module::Screenshot;

fn main() {
    let screens=screenshots::Screen::all().unwrap();
    let mut count=0;
    for i in screens{
        let ss=Screenshot::new(i).unwrap();
        let mut path=r"C:\Users\miche\Downloads\ProgettoRustProvaImage".to_string();
        path.push_str(count.to_string().as_mut_str());
        path.push_str(".png".to_string().as_str());
        count+=1;
        ss.save_image(PathBuf::from(path),ImageFormat::Png).unwrap();
    }
}
