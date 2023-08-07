pub mod hotkey_module{
    use std::error::Error;
    use global_hotkey::GlobalHotKeyManager;
    use global_hotkey::hotkey::{Code, HotKey, Modifiers};
    pub enum KeyType{
        Quick,
        Open,
    }
    pub struct HotkeyManager{
        manager:GlobalHotKeyManager,
        quick_key:Option<HotKey>,
        open_key:Option<HotKey>
    }
    impl HotkeyManager {
        pub fn new()->Result<Self,Box<dyn Error>>{
            Ok(HotkeyManager{
                manager:GlobalHotKeyManager::new()?,
                quick_key:None,
                open_key:None
            })
        }
        pub fn register_new_hotkey(&mut self, modifier:Option<Modifiers>,key:Code,key_type:KeyType)->Result<u32,Box<dyn Error>>{
            match key_type {
                KeyType::Open=>{
                    if self.open_key.is_some() {
                        self.manager.unregister(self.open_key.unwrap())?;
                    }
                    let hk=HotKey::new(modifier,key);
                    self.manager.register(hk)?;
                    self.open_key=Some(hk);
                    Ok(hk.id())
                },
                KeyType::Quick=>{
                    if self.quick_key.is_some() {
                        self.manager.unregister(self.quick_key.unwrap())?;
                    }
                    let hk=HotKey::new(modifier,key);
                    self.manager.register(hk)?;
                    self.quick_key=Some(hk);
                    Ok(hk.id())
                }
            }
        }
        pub fn clean_hotkey(&mut self)->Result<(),Box<dyn Error>>{
            if self.quick_key.is_some(){
                self.manager.unregister(self.quick_key.unwrap())?;
            }
            if self.open_key.is_some(){
                self.manager.unregister(self.open_key.unwrap())?;
            }
            self.quick_key.take();
            self.open_key.take();
            Ok(())
        }

        pub fn get_key_open(&self)->Option<u32>{
            if self.open_key.is_some(){
                return Some(self.open_key.unwrap().id())
            }
            None
        }
        pub fn get_key_quick(&self)->Option<u32>{
            if self.quick_key.is_some(){
                return Some(self.quick_key.unwrap().id())
            }
            None
        }
    }
}