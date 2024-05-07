use std::sync::Mutex;

use rayon::{iter::ParallelIterator, str::ParallelString};

use super::{info_log_processor::LogInfo, LogProcessor};

pub type CredentialType = Option<String>;


enum CredentialFields {
    Url(CredentialType),
    Username(CredentialType),
    Password(CredentialType),
    Info(LogInfo)
}

#[derive(Debug, Clone)]
pub struct Credential {
    url: CredentialType,
    username: CredentialType,
    password: CredentialType,
    infos: LogInfo
}

impl Credential {
    pub(crate) fn new() -> Self {
        Self { url: Some(String::new()), username: Some(String::new()), password: Some(String::new()), infos: LogInfo::new() }
    }
    fn set(&mut self, field: CredentialFields) {
        match field {
            CredentialFields::Url(value) => self.url = value,
            CredentialFields::Username(value) => self.username = value,
            CredentialFields::Password(value) => self.password = value,
            CredentialFields::Info(value) => self.infos = value,
        }
    }

    pub(crate) fn url(&self) -> CredentialType {
        self.url.clone()
    }
    pub(crate) fn username(&self) -> CredentialType {
        self.username.clone()
    }
    pub(crate) fn password(&self) -> CredentialType {
        self.password.clone()
    }
}
pub struct PassLogProcessor {info: LogInfo}

impl LogProcessor for PassLogProcessor {
    type Out = Vec<Credential>;

    fn parse<C: AsRef<str>>(&self, content: C) -> Self::Out {
        let data = content.as_ref();
        let credentials = Mutex::new(Vec::new());
        
        let url = Mutex::new(String::new());
        let username = Mutex::new(String::new());
        let password = Mutex::new(String::new());

        data
        .par_lines()
        .filter(|line| !line.trim().is_empty() && !line.contains("="))
        .for_each(|line| {
            let Some((key, value)) = line.split_once(":").map(|(k, v)| (k.to_lowercase(), v.trim())) else {
                return;
            };

            let mut credential = Credential::new();
            let mut creds = credentials.lock().unwrap();
            
            let mut url_ref = url.lock().unwrap();
            let mut username_ref = username.lock().unwrap();
            let mut password_ref = password.lock().unwrap();

            credential.set(CredentialFields::Info(self.info.clone()));

            match key.as_ref() {
                "url" => {
                    
                    if !url_ref.is_empty() && !username_ref.is_empty() && !password_ref.is_empty() {
                        credential.set(CredentialFields::Url(Some(url_ref.to_string())));
                        credential.set(CredentialFields::Username(Some(username_ref.to_string())));
                        credential.set(CredentialFields::Password(Some(password_ref.to_string())));
        
                        creds.push(credential)
                    }

                    url_ref.replace_range(.., value)
                },
                "username" => {
                    username_ref.replace_range(.., value)
                },
                "password" => {
                    password_ref.replace_range(.., value)
                },
                _ => {}
            }
        });

        credentials.into_inner().unwrap()
    }
}

impl PassLogProcessor {
    pub(crate) fn new(info: &LogInfo) -> Self {Self {info: info.clone()}}
}