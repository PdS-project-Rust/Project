pub mod hotkey_module{
    use std::error::Error;
    use tauri_hotkey::HotkeyManager;
    use tauri_hotkey::Hotkey as HotKeyTauri;
    use thiserror::Error;

    #[derive(Error,Debug)]
    enum HotkeyError{
        #[error("hotkey is already assigned")]
        HotkeyAssigned,
        #[error("hotkey wasn't assigned")]
        HotkeyNotAssigned,
    }
    pub struct Hotkey{
        hotkey_manager:HotkeyManager,
        hotkey_present:Option<HotKeyTauri>,
    }

    impl Hotkey {
        pub fn new()->Result<Hotkey,Box<dyn Error>>{
            Ok(
                Hotkey{
                    hotkey_manager:HotkeyManager::new(),
                    hotkey_present:None,
                }
            )
        }
        pub fn register_hotkey(&mut self, hk:HotKeyTauri) ->Result<(),Box<dyn Error>>{
            if self.hotkey_manager.is_registered(&hk) {
                return Err(Box::new(HotkeyError::HotkeyAssigned));
            }
            if self.hotkey_present.is_some() {
                self.hotkey_manager.unregister(&self.hotkey_present.as_ref().unwrap())?;
            }
            self.hotkey_manager.register(hk.clone(),||{
                println!("ciao");
                unimplemented!();
            })?;
            self.hotkey_present=Some(hk);
            Ok(())
        }
        pub fn remove_hotkey(&mut self)->Result<(),Box<dyn Error>>{
            if self.hotkey_present.is_none() {
                return Err(Box::new(HotkeyError::HotkeyNotAssigned))
            }
            self.hotkey_manager.unregister(self.hotkey_present.as_ref().unwrap())?;
            self.hotkey_present.take();
            Ok(())
        }
    }
}