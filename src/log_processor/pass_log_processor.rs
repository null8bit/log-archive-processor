use std::collections::HashSet;

use super::{info_log_processor::LogInfo, LogProcessor};

pub type CredentialType = Option<String>;

enum CredentialFields {
    Url(String),
    Username(String),
    Password(String),
    Info(LogInfo)
}
pub struct Credential {
    url: CredentialType,
    username: CredentialType,
    passowrd: CredentialType,
    infos: LogInfo
}

impl Credential {
    pub(crate) fn new() -> Self {
        Self { url: Some(String::new()), username: Some(String::new()), passowrd: Some(String::new()), infos: LogInfo::new() }
    }
    fn set(&mut self, field: CredentialFields) {}

    pub(crate) fn url(&self) -> CredentialType {
        self.url.clone()
    }
    pub(crate) fn username(&self) -> CredentialType {
        self.username.clone()
    }
    pub(crate) fn password(&self) -> CredentialType {
        self.passowrd.clone()
    }
}
pub struct PassLogProcessor;

impl LogProcessor for PassLogProcessor {
    type Out = HashSet<Credential>;

    fn parse<C: AsRef<str>>(content: C) -> Self::Out {
        let data = content.as_ref();
        let credentials = HashSet::new();

        credentials
    }
}