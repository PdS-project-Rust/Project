pub mod settings_module {
    use std::error::Error;
    use serde::{Serialize, Deserialize};
    use serde_json;
    use global_hotkey::hotkey::Code;
    use std::str::FromStr;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Settings {
        pub open: String,
        pub quick: String,
        pub path: String,
    }
    impl Default for Settings {
        fn default() -> Settings {
            Settings {
                open: String::from("D"),
                quick: String::from("F"),
                path: String::from("./screenshot/")
            }
        }
    }
    impl Settings {

        pub fn get_open_hotkey(&self) -> Result<Code,Box<dyn Error>> {
            let code_str = format!("Key{}", self.open);
            Ok(Code::from_str(&code_str)?)
        }

        pub fn get_screenshot_hotkey(&self) -> Result<Code,Box<dyn Error>> {
            let code_str = format!("Key{}", self.quick);
            Ok(Code::from_str(&code_str)?)
        }
    }

    pub fn read_settings_from_file(filename: String) -> Result<Settings, Box<dyn Error>> {
        let file = std::fs::File::open(filename);
        if file.is_ok(){
            let file=file.unwrap();
            let reader = std::io::BufReader::new(file);
            let u = serde_json::from_reader(reader)?;
            Ok(u)
        }else{
            let sett=Settings::default();
            write_settings_to_file("settings.json".to_string(),&sett)?;
            Ok(sett)
        }
    }

    pub fn write_settings_to_file(filename: String, settings: &Settings) -> Result<(), Box<dyn Error>> {
        let file = std::fs::File::create(filename)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer(writer, settings)?;
        Ok(())
    }
        

}