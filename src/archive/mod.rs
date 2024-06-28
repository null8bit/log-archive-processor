use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Result, Write},
    path::Path,
};

use regex::Regex;

pub mod z_archive;

#[derive(Clone, Debug)]
pub struct FilterOptions {
    name: Option<Vec<Regex>>,
    extension: Option<Vec<String>>,
}

impl FilterOptions {
    pub fn new(name: Option<Vec<Regex>>, ext: Option<Vec<String>>) -> Self {
        Self {
            name: name,
            extension: ext,
        }
    }

    pub fn get_regex(&self) -> &Option<Vec<Regex>> {
        &self.name
    }
    pub fn get_extension(&self) -> &Option<Vec<String>> {
        &self.extension
    }
}
pub trait ArchiveFilter {
    type Options;

    fn new<F: IntoIterator<Item = Regex>, E: IntoIterator<Item = String>>(
        name: Option<F>,
        ext: Option<E>,
    ) -> Self;
    fn archive_filter(&self, item: &str) -> bool;
}

pub trait Archive {
    type This;
    async fn new<P: AsRef<Path>>(file: P) -> std::io::Result<Self::This>;
    fn enumerate(&self, filter: impl ArchiveFilter) -> Vec<&str>;
    fn reader(&mut self, filename: &str) -> tokio::io::Result<String>;
}
pub enum SupportedExtension {
    Zip,
    Unsupported,
}

pub struct ArchiveUtils;

impl ArchiveUtils {
    pub fn generate_hash<P: AsRef<Path>>(path: P) -> Result<String> {
        let filepath = path.as_ref();
        let file = File::open(&filepath)?;

        let mut hash = md5::Context::new();

        let mut reader = BufReader::new(file);

        let mut buff = [0; 1024];

        let read_limit = 100 * 1024 * 1024;
        let mut read_count = 0;

        while let Ok(bytes) = reader.read(&mut buff) {
            if bytes == 0 {
                break;
            }

            if read_count + bytes > read_limit {
                let remaining_bytes = bytes - (read_count + bytes - read_limit);
                hash.consume(&buff[..remaining_bytes]);
                break;
            } else {
                hash.consume(&buff[..bytes]);
                read_count += bytes;
            }
        }

        let result = hash.compute();

        Ok(format!("{:x}", result))
    }

    pub fn verify_existence<P: AsRef<Path>>(path: P) -> Result<bool> {
        let filepath = path.as_ref();

        if !filepath.exists() {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file_not_exists",
            ))
        } else {
            Ok(true)
        }
    }

    pub fn verify_extension<P: AsRef<Path>>(path: P) -> Result<SupportedExtension> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path have be a file",
            ));
        }
        let extension = path.extension().unwrap();

        match extension.to_str().unwrap() {
            "zip" => Ok(SupportedExtension::Zip),
            _ => Ok(SupportedExtension::Unsupported),
        }
    }

    pub fn register_hash<H: AsRef<str>>(hash: H) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open("hashes.txt")?;

        writeln!(file, "{}", hash.as_ref())?;

        Ok(())
    }

    pub fn is_registered<H: AsRef<str>>(hash: H) -> Result<bool> {
        let filename = Path::new("hashes.txt");

        if !filename.exists() {
            return Ok(false);
        };

        let file = File::open(filename)?;

        let hash = hash.as_ref();

        let mut reader = BufReader::new(file);

        let mut buff = String::new();

        while let Ok(bytes) = reader.read_line(&mut buff) {
            if bytes == 0 {
                break;
            }

            if buff.trim() == hash {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
