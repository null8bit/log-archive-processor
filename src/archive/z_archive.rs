
use std::io::Read;
use std::path::Path;

use std::fs::File;

use zip::ZipArchive;

use super::ArchiveFilter;
use super::Archive;

#[derive(Debug)]
pub struct Zarchive {
   archive: ZipArchive<File>
}

impl Archive for Zarchive {
    type This = Self;

    async fn new<P: AsRef<Path>>(file: P) -> std::io::Result<Self::This>{
        let filepath = file.as_ref();
        let reader = File::open(filepath)?;
        let zip_archive = ZipArchive::new(reader)?;

        Ok(Self {archive: zip_archive})
    }

    fn enumerate(&self, filter: impl ArchiveFilter) -> Vec<&str> {
        (
            &self.archive.file_names()
            .filter(|item| filter.archive_filter(item))
            .collect::<Vec<&str>>()
        ).to_vec()
    }

    async fn reader(&mut self, filename: &str) -> tokio::io::Result<String> {
        let mut z_file = self.archive.by_name(filename)?;
        let mut buffer = Vec::new();

        z_file.read_to_end(&mut buffer)?;

        let content = String::from_utf8_lossy(&buffer);

        Ok(content.to_string())
    }
    

}