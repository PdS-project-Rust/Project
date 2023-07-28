pub mod api_module {
    use screenshots::Screen;
    use crate::screenshots_module::screenshot_module::Screenshot;
    use std::time::Duration;

    pub fn get_screens() -> Vec<usize> {
        let screens= Screen::all().unwrap();
        let mut screen_indeces:Vec<usize>=Vec::new();
    
        for (index,_screen) in screens.iter().enumerate() {
            screen_indeces.push(index);
        }
        screen_indeces
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