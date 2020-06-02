use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, Result, SeekFrom};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Data {
    uid: Option<String>,
    token: Option<String>,
}

#[derive(Debug)]
pub struct Settings {
    data: Data,
    file: Option<File>,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            data: Data {
                uid: None,
                token: None,
            },
            file: None,
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let mut data = toml::from_str::<Data>(&contents)?;

        self.data.uid = data.uid.take();
        self.data.token = data.token.take();

        Ok(())
    }

    pub fn save_to_file(&mut self) -> Result<()> {
        let buffer = toml::to_string_pretty(&self.data).unwrap();

        let write_to_file = |file: &mut File| {
            file.seek(SeekFrom::Start(0))?;
            file.set_len(0)?;
            file.write_all(buffer.as_bytes())?;
            file.sync_all()?;
            Ok(())
        };

        match self.file {
            Some(ref mut file) => write_to_file(file),
            None => {
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open("mdl.toml")?;
                write_to_file(&mut file)?;
                self.file = Some(file);
                Ok(())
            }
        }
    }

    pub fn set_token(&mut self, token: &str) {
        self.data.token.replace(String::from(token));
    }
    pub fn get_token(&self) -> Option<&str> {
        self.data.token.as_ref().map(|t| t.as_str())
    }
    pub fn set_uid(&mut self, uid: &str) {
        self.data.uid.replace(String::from(uid));
    }
    pub fn get_uid(&self) -> Option<&str> {
        self.data.uid.as_ref().map(|t| t.as_str())
    }
}
