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

    pub enum ActiveShortcuts{
        ScreenshotDone,
        ScreenshotWaiting,
        Pause
    }
    pub struct HotkeyManager{
        manager:GlobalHotKeyManager,
        quick_screenshot:(Option<HotKey>,bool),
        new_screenshot:(Option<HotKey>,bool),
        save:(Option<HotKey>,bool),
        pen:(Option<HotKey>,bool),
        rubber:(Option<HotKey>,bool),
    }
    impl HotkeyManager {
        pub fn new()->Result<Self,Box<dyn Error>>{
            Ok(HotkeyManager{
                manager:GlobalHotKeyManager::new()?,
                quick_screenshot:(None,false),
                new_screenshot:(None,false),
                save:(None,false),
                pen:(None,false),
                rubber:(None,false),
            })
        }
        pub fn register_new_hotkey(&mut self, modifier:Option<Modifiers>,key:Code,key_type:KeyType)->Result<u32,Box<dyn Error>>{
            match key_type {
                KeyType::Quick=>{
                    if self.quick_screenshot.0.is_some() && self.quick_screenshot.1 == true{
                        self.manager.unregister(self.quick_screenshot.0.unwrap())?;
                    }
                    let hk=HotKey::new(modifier,key);
                    self.manager.register(hk)?;
                    self.quick_screenshot=(Some(hk),true);
                    Ok(hk.id())
                },
                KeyType::NewScreenshot=>{
                    if self.new_screenshot.0.is_some() && self.new_screenshot.1 == true {
                        self.manager.unregister(self.new_screenshot.0.unwrap())?;
                    }
                    let hk=HotKey::new(modifier,key);
                    self.manager.register(hk)?;
                    self.new_screenshot=(Some(hk),true);
                    Ok(hk.id())
                },
                KeyType::Save=>{
                    if self.save.0.is_some() && self.save.1 == true {
                        self.manager.unregister(self.save.0.unwrap())?;
                    }
                    let hk=HotKey::new(modifier,key);
                    self.manager.register(hk)?;
                    self.save=(Some(hk),true);
                    Ok(hk.id())
                },
                KeyType::Pen=>{
                    if self.pen.0.is_some() && self.pen.1 == true {
                        self.manager.unregister(self.pen.0.unwrap())?;
                    }
                    let hk=HotKey::new(modifier,key);
                    self.manager.register(hk)?;
                    self.pen=(Some(hk),true);
                    Ok(hk.id())
                },
                KeyType::Rubber=>{
                    if self.rubber.0.is_some() && self.rubber.1 == true {
                        self.manager.unregister(self.rubber.0.unwrap())?;
                    }
                    let hk=HotKey::new(modifier,key);
                    self.manager.register(hk)?;
                    self.rubber=(Some(hk),true);
                    Ok(hk.id())
                },
            }
        }

        pub fn disable_shortcut(&mut self, key_type:KeyType) -> Result<(),Box<dyn Error>> {
            return match key_type {
                KeyType::Quick=>{
                    if self.quick_screenshot.0.is_some() && self.quick_screenshot.1 == true{
                        self.manager.unregister(self.quick_screenshot.0.unwrap())?;
                        self.quick_screenshot.1==false;
                    }
                    Ok(())
                },
                KeyType::Pen=>{
                    if self.pen.0.is_some() && self.pen.1 == true{
                        self.manager.unregister(self.pen.0.unwrap())?;
                        self.pen.1==false;
                    }
                    Ok(())
                },
                KeyType::NewScreenshot=>{
                    if self.new_screenshot.0.is_some() && self.new_screenshot.1 == true{
                        self.manager.unregister(self.new_screenshot.0.unwrap())?;
                        self.new_screenshot.1==false;
                    }
                    Ok(())
                },
                KeyType::Rubber=>{
                    if self.rubber.0.is_some() && self.rubber.1 == true{
                        self.manager.unregister(self.rubber.0.unwrap())?;
                        self.rubber.1==false;
                    }
                    Ok(())
                },
                KeyType::Save=>{
                    if self.save.0.is_some() && self.save.1 == true{
                        self.manager.unregister(self.save.0.unwrap())?;
                        self.save.1==false;
                    }
                    Ok(())
                }
            }
        }
        pub fn enable_shortcut(&mut self, key_type:KeyType) -> Result<(),Box<dyn Error>> {
            return match key_type {
                KeyType::Quick=>{
                    if self.quick_screenshot.0.is_some() && self.quick_screenshot.1 == false{
                        self.manager.register(self.quick_screenshot.0.unwrap())?;
                        self.quick_screenshot.1=true;
                    }
                    Ok(())
                },
                KeyType::Pen=>{
                    if self.pen.0.is_some() && self.pen.1 == false{
                        self.manager.register(self.pen.0.unwrap())?;
                        self.pen.1=true;
                    }
                    Ok(())
                },
                KeyType::NewScreenshot=>{
                    if self.new_screenshot.0.is_some() && self.new_screenshot.1 == false{
                        self.manager.register(self.new_screenshot.0.unwrap())?;
                        self.new_screenshot.1=true;
                    }
                    Ok(())
                },
                KeyType::Rubber=>{
                    if self.rubber.0.is_some() && self.rubber.1 == false{
                        self.manager.register(self.rubber.0.unwrap())?;
                        self.rubber.1=true;
                    }
                    Ok(())
                },
                KeyType::Save=>{
                    if self.save.0.is_some() && self.save.1 == false{
                        self.manager.register(self.save.0.unwrap())?;
                        self.save.1=true;
                    }
                    Ok(())
                }
            }
        }
        pub fn set_active_shortcuts(&mut self,active_shortcuts:ActiveShortcuts)->Result<(),Box<dyn Error>>{
            match active_shortcuts {
                ActiveShortcuts::Pause=>{
                    self.disable_shortcut(KeyType::Save)?;
                    self.disable_shortcut(KeyType::Pen)?;
                    self.disable_shortcut(KeyType::Rubber)?;
                    self.disable_shortcut(KeyType::NewScreenshot)?;
                    self.disable_shortcut(KeyType::Quick)?;
                },
                ActiveShortcuts::ScreenshotWaiting=>{
                    self.disable_shortcut(KeyType::Rubber)?;
                    self.disable_shortcut(KeyType::Pen)?;
                    self.enable_shortcut(KeyType::Quick)?;
                    self.enable_shortcut(KeyType::NewScreenshot)?;
                    self.disable_shortcut(KeyType::Save)?;
                },
               ActiveShortcuts::ScreenshotDone=>{
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
                    if self.quick_screenshot.0.is_some(){
                        return Some(self.quick_screenshot.0.unwrap().id());
                    }
                    None
                },
                KeyType::Save=>{
                    if self.save.0.is_some(){
                        return Some(self.save.0.unwrap().id());
                    }
                    None
                },
                KeyType::Rubber=>{
                    if self.rubber.0.is_some(){
                        return Some(self.rubber.0.unwrap().id());
                    }
                    None
                },
                KeyType::Pen=>{
                    if self.pen.0.is_some(){
                        return Some(self.pen.0.unwrap().id());
                    }
                    None
                },
                KeyType::NewScreenshot=>{
                    if self.new_screenshot.0.is_some(){
                        return Some(self.new_screenshot.0.unwrap().id());
                    }
                    None
                }
            }
        }
    }
}