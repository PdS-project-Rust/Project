pub mod hotkey_module{
    use std::error::Error;
    use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyEventReceiver, GlobalHotKeyManager};
    use global_hotkey::hotkey::{Code, HotKey, Modifiers};
    pub struct HotkeyManager{
        manager:GlobalHotKeyManager,
        present_key:Option<HotKey>,
    }
    impl HotkeyManager {
        pub fn new()->Result<Self,Box<dyn Error>>{
            Ok(HotkeyManager{
                manager:GlobalHotKeyManager::new()?,
                present_key:Option::None,
            })
        }
        pub fn register_new_hotkey(&mut self, modifier:Option<Modifiers>,key:Code)->Result<u32,Box<dyn Error>>{
            if self.present_key.is_some() {
                self.manager.unregister(self.present_key.unwrap())?;
            }
            let hk=HotKey::new(modifier,key);
            self.manager.register(hk)?;
            self.present_key=Option::Some(hk);
            Ok(hk.id())
        }
        pub fn clean_hotkey(&mut self)->Result<(),Box<dyn Error>>{
            if self.present_key.is_some(){
                self.manager.unregister(self.present_key.unwrap())?;
            }
            self.present_key.take();
            Ok(())
        }
    }
}