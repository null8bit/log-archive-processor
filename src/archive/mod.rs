use std::path::Path;

use regex::Regex;

pub mod z_archive;


#[derive(Clone, Debug)]
pub struct FilterOptions {
    name: Option<Vec<Regex>>,
    extension: Option<Vec<String>>
}

impl FilterOptions {
    pub fn new(name: Option<Vec<Regex>>, ext: Option<Vec<String>>) -> Self {
        Self {
            name: name,
            extension: ext
        }
    }

    pub fn get_regex(&self) -> &Option<Vec<Regex>> {&self.name}
    pub fn get_extension(&self) -> &Option<Vec<String>> {&self.extension}
}
pub trait ArchiveFilter {
    type Options;
    
    fn new<F: IntoIterator<Item = Regex> , E: IntoIterator<Item = String>>(name: Option<F>, ext: Option<E>) -> Self;
    fn archive_filter(&self, item: &str) -> bool;
}

pub trait Archive {
    type This;
    async fn new<P: AsRef<Path>>(file: P) -> std::io::Result<Self::This>;
    fn enumerate(&self, filter: impl ArchiveFilter) -> Vec<&str>;
    async fn reader(&mut self, filename: &str) -> tokio::io::Result<String>;
}