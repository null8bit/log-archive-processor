use std::{collections::HashMap, io::Error};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{info_log_processor::LogInfo, LogProcessor};

lazy_static! {
    static ref REGEX_SPLIT_VALUES: Regex = Regex::new(r"[\t]+").unwrap();
}

pub enum CookieFields {
    Domain(String),
    HttpOnly(String),
    Path(String),
    Secure(String),
    ExpiresIn(String),
    Name(String),
    Value(String)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    domain: String,
    http_only: String,
    path: String,
    secure: String,
    expires_in: String,
    name: String,
    value: String
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LogCookie {
    info: LogInfo,
    cookies: Vec<Cookie>
}

impl LogCookie {
    fn new(info: LogInfo, cookies: Vec<Cookie>) -> Self {
        Self { info, cookies }
    }

    pub fn cookies(&self) -> Vec<Cookie> {
        self.cookies.clone()
    }

    pub fn infos(&self) -> LogInfo {
        self.info.clone()
    }
}
impl Cookie {
    pub fn new() -> Self {
        Self { domain: String::new(), http_only: String::new(), path: String::new(), secure: String::new(), name: String::new(), value: String::new(), expires_in: String::new() }
    }

    pub fn set(&mut self, field: CookieFields) {
        
        match field {
            CookieFields::Domain(value) => self.domain.replace_range(.., &value),
            CookieFields::HttpOnly(value) => self.http_only.replace_range(.., &value),
            CookieFields::Path(value) => self.path.replace_range(.., &value),
            CookieFields::Secure(value) => self.secure.replace_range(.., &value),
            CookieFields::Name(value) => self.name.replace_range(.., &value),
            CookieFields::Value(value) => self.value.replace_range(.., &value),
            CookieFields::ExpiresIn(value) => self.expires_in.replace_range(.., &value),
        };
    }
}
pub struct CookieLogProcessor {
    infos: LogInfo
}

impl LogProcessor for CookieLogProcessor {
    type Out = Result<HashMap<String, LogCookie>, std::io::Error>;

    fn parse<C: AsRef<str>>(&self, content: C) -> Self::Out {
        let mut cookies_map = HashMap::new();

        content.as_ref()
        .lines().for_each(|line| {
            let mut cookie = Cookie::new();
            let log_info = self.infos.clone();

            let explode: Vec<_> = REGEX_SPLIT_VALUES.split(line)
            .map(|item| item.trim())
            .map(|item| {
                if item.starts_with(".") {
                    item.replacen(".", "", 1)
                } else {
                    item.to_string()
                }
            }).filter_map(|item| {
                if item.is_empty() {
                    None
                } else {
                    Some(item)
                }
            })
            .collect();
            
   
            if explode.len() == 7 {
                cookie.set(CookieFields::Domain(explode[0].to_string()));
                cookie.set(CookieFields::HttpOnly(explode[1].to_string()));
                cookie.set(CookieFields::Path(explode[2].to_string()));
                cookie.set(CookieFields::Secure(explode[3].to_string()));
                cookie.set(CookieFields::ExpiresIn(explode[4].to_string()));
                cookie.set(CookieFields::Name(explode[5].to_string()));
                cookie.set(CookieFields::Value(explode[6].to_string()));

                let entry = cookies_map.entry(cookie.domain.clone()).or_insert(LogCookie::new( log_info, Vec::new() ));
                entry.cookies.push(cookie)

                // let entry = cookies_map.entry(cookie.domain.clone()).or_insert(Vec::new());
                // entry.push(cookie);
            }
        });

        if cookies_map.is_empty() {
            Err(Error::new(std::io::ErrorKind::UnexpectedEof, "Cookie hashmap is empty"))
        } else {
            Ok(cookies_map)
        }
    }
}

impl CookieLogProcessor {
    pub fn new(infos: &LogInfo) -> Self {
        Self {infos: infos.clone()}
    }
}