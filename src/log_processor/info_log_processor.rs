use std::sync::{Arc, Mutex};

use rayon::prelude::*;

use super::LogProcessor;


type InfoType = Option<String>;

pub enum LogInfoFields {
    Country(String),
    Hwid(String)
}

#[derive(Clone, Debug)]
pub struct LogInfo {
    country: InfoType,
    hwid: InfoType
}

impl LogInfo {
    
    pub fn new() -> Self {
        Self {country: None, hwid: None}
    }
    
    pub(crate) fn country(&self) -> InfoType {
        self.country.clone()
    }
    pub(crate) fn hwid(&self) -> InfoType {
        self.hwid.clone()
    }

    fn set(&mut self, field: LogInfoFields) {
        match field {
            LogInfoFields::Country(value) => {
                if !value.is_empty() {
                    self.country = Some(value)
                }
            },
            LogInfoFields::Hwid(value) => {
                if !value.is_empty() {
                    self.hwid = Some(value)
                }
            }
        }
    }
}
pub struct InfoLogProcessor;

impl LogProcessor for InfoLogProcessor {
    type Out = LogInfo;

    fn parse<C: AsRef<str>>(&self, content: C) -> LogInfo {
        let info = Arc::new(Mutex::new(LogInfo::new()));
        let data = content.as_ref();
        
        data.par_lines().for_each(|line| {
            
            let Some((key, value)) = line.split_once(":").map(|(k, v)| (k.trim(), v.trim() )) else {
                return;
            };

            let mut info = info.lock().unwrap();

            match key.to_lowercase().as_str() {
                "hwid" => {
                    info.set(LogInfoFields::Hwid(value.to_string()))
                }

                "country" => {
                    info.set(LogInfoFields::Country(value.to_string()))
                }
                _ => {}
            }
        });

        Arc::try_unwrap(info).ok().expect("Falha ao recuperar o objeto").into_inner().unwrap()
    }
    
}

impl InfoLogProcessor {
    pub(crate) fn new() -> Self {Self}
}