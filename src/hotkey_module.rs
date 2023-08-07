pub mod hotkey_module{
    use std::error::Error;
    use global_hotkey::GlobalHotKeyManager;
    use global_hotkey::hotkey::{Code, HotKey, Modifiers};
    use thiserror::Error;
    use crate::state_module::state_module::DrawingMode;

    pub enum KeyType{
        Quick,
        NewScreenshot,
        Save,
        Pen,
        Rubber,
    }
    pub struct HotkeyManager{
        manager:GlobalHotKeyManager,
        quick_screenshot:Option<HotKey>,
        new_screenshot:Option<HotKey>,
        save:Option<HotKey>,
        pen:Option<HotKey>,
        rubber:Option<HotKey>,
    }
    impl HotkeyManager {
        pub fn new()->Result<Self,Box<dyn Error>>{
            Ok(HotkeyManager{
                manager:GlobalHotKeyManager::new()?,
                quick_screenshot:None,
                new_screenshot:None,
                save:None,
                pen:None,
                rubber:None,
            })
        }
        pub fn register_new_hotkey(&mut self, modifier:Option<Modifiers>,key:Code,key_type:KeyType)->Result<u32,Box<dyn Error>>{
            match key_type {
                KeyType::Quick=>{
                    if self.quick_screenshot.is_some() {
                        self.manager.unregister(self.quick_screenshot.unwrap())?;
                    }
                    let hk=HotKey::new(modifier,key);
                    self.manager.register(hk)?;
                    self.quick_screenshot=Some(hk);
                    Ok(hk.id())
                },
                KeyType::NewScreenshot=>{
                    if self.new_screenshot.is_some() {
                        self.manager.unregister(self.new_screenshot.unwrap())?;
                    }
                    let hk=HotKey::new(modifier,key);
                    self.manager.register(hk)?;
                    self.new_screenshot=Some(hk);
                    Ok(hk.id())
                },
                KeyType::Save=>{
                    if self.save.is_some() {
                        self.manager.unregister(self.save.unwrap())?;
                    }
                    let hk=HotKey::new(modifier,key);
                    self.manager.register(hk)?;
                    self.save=Some(hk);
                    Ok(hk.id())
                },
                KeyType::Pen=>{
                    if self.pen.is_some() {
                        self.manager.unregister(self.pen.unwrap())?;
                    }
                    let hk=HotKey::new(modifier,key);
                    self.manager.register(hk)?;
                    self.pen=Some(hk);
                    Ok(hk.id())
                },
                KeyType::Rubber=>{
                    if self.rubber.is_some() {
                        self.manager.unregister(self.rubber.unwrap())?;
                    }
                    let hk=HotKey::new(modifier,key);
                    self.manager.register(hk)?;
                    self.rubber=Some(hk);
                    Ok(hk.id())
                },
            }
        }

        pub fn disable_shortcut(&mut self, key_type:KeyType) -> Result<(),Box<dyn Error>> {
            return match key_type {
                KeyType::Quick=>{
                    if self.quick_screenshot.is_some(){
                        self.manager.unregister(self.quick_screenshot.unwrap())?;
                    }
                    Ok(())
                },
                KeyType::Pen=>{
                    if self.pen.is_some(){
                        self.manager.unregister(self.pen.unwrap())?;
                    }
                    Ok(())
                },
                KeyType::NewScreenshot=>{
                    if self.new_screenshot.is_some(){
                        self.manager.unregister(self.new_screenshot.unwrap())?;
                    }
                    Ok(())
                },
                KeyType::Rubber=>{
                    if self.rubber.is_some(){
                        self.manager.unregister(self.rubber.unwrap())?;
                    }
                    Ok(())
                },
                KeyType::Save=>{
                    if self.save.is_some(){
                        self.manager.unregister(self.save.unwrap())?;
                    }
                    Ok(())
                }
            }
        }
        pub fn enable_shortcut(&mut self, key_type:KeyType) -> Result<(),Box<dyn Error>> {
            return match key_type {
                KeyType::Quick=>{
                    if self.quick_screenshot.is_some(){
                        self.manager.register(self.quick_screenshot.unwrap())?;
                    }
                    Ok(())
                },
                KeyType::Pen=>{
                    if self.pen.is_some(){
                        self.manager.register(self.pen.unwrap())?;
                    }
                    Ok(())
                },
                KeyType::NewScreenshot=>{
                    if self.new_screenshot.is_some(){
                        self.manager.register(self.new_screenshot.unwrap())?;
                    }
                    Ok(())
                },
                KeyType::Rubber=>{
                    if self.rubber.is_some(){
                        self.manager.register(self.rubber.unwrap())?;
                    }
                    Ok(())
                },
                KeyType::Save=>{
                    if self.save.is_some(){
                        self.manager.register(self.save.unwrap())?;
                    }
                    Ok(())
                }
            }
        }
        pub fn set_drawing_shortcuts(&mut self,drawing_mode:Option<DrawingMode>)->Result<(),Box<dyn Error>>{
            match drawing_mode {
                Some(DrawingMode::Pause)=>{
                    self.disable_shortcut(KeyType::Save)?;
                    self.disable_shortcut(KeyType::Pen)?;
                    self.disable_shortcut(KeyType::Rubber)?;
                    self.disable_shortcut(KeyType::NewScreenshot)?;
                    self.disable_shortcut(KeyType::Quick)?;
                },
                _=>{
                    self.enable_shortcut(KeyType::Rubber)?;
                    self.enable_shortcut(KeyType::Pen)?;
                    self.enable_shortcut(KeyType::Quick)?;
                    self.enable_shortcut(KeyType::NewScreenshot)?;
                    self.enable_shortcut(KeyType::Save)?;
                }
            }
            return Ok(());
        }

        pub fn get_key(&self,key_type:KeyType)->Option<u32>{
            return match key_type {
                KeyType::Quick=>{
                    if self.quick_screenshot.is_some(){
                        return Some(self.quick_screenshot.unwrap().id());
                    }
                    None
                },
                KeyType::Save=>{
                    if self.save.is_some(){
                        return Some(self.save.unwrap().id());
                    }
                    None
                },
                KeyType::Rubber=>{
                    if self.rubber.is_some(){
                        return Some(self.rubber.unwrap().id());
                    }
                    None
                },
                KeyType::Pen=>{
                    if self.pen.is_some(){
                        return Some(self.pen.unwrap().id());
                    }
                    None
                },
                KeyType::NewScreenshot=>{
                    if self.new_screenshot.is_some(){
                        return Some(self.new_screenshot.unwrap().id());
                    }
                    None
                }
            }
        }
    }
}