mod archive;
mod log_processor;

use std::io::Result;

use archive::{z_archive::Zarchive, ArchiveFilter};
use lazy_static::lazy_static;
use log_processor::log_filter::LogFilter;

use regex::Regex;

use crate::{archive::Archive, log_processor::{info_log_processor::InfoLogProcessor, LogProcessor}};

lazy_static! {
    static ref SYSTEM_INFO_REGEX: Regex = Regex::new(r"(?i)((system)|(info))").unwrap();
    static ref PASSWORD_REGEX: Regex = Regex::new(r"(?i)(pass)").unwrap();
    static ref COOKIES_REGEX: Regex = Regex::new(r"(?i)(cookies)").unwrap();
}

#[tokio::main]
async fn main() -> Result<()>{
    let time = std::time::Instant::now();

    let mut zarchive = Zarchive::new("C:\\Users\\conta\\Downloads\\Telegram Desktop\\2024-04-23_b_@logsinspector.zip").await?;

    let mut filter = LogFilter::new(
        Some(vec![
            Regex::new(r"(?i)((pass)|(system)|(info)|(cookies))").unwrap()
        ]
    ), Some(vec![
        String::from("txt")
    ]));
    
    let content = zarchive.enumerate(filter.clone());
    let relationed = filter.relation_mapper(content).to_owned();

    for (folder, filenames) in relationed {

        let get_info_filename = filenames.iter().find(|e| SYSTEM_INFO_REGEX.is_match(e));
        let get_pass_filename = filenames.iter().find(|e| PASSWORD_REGEX.is_match(e));
        let get_cook_filename = filenames.iter().find(|e| COOKIES_REGEX.is_match(e));

        if let (Some(info_filename), Some(pass), Some(cook)) = (get_info_filename, get_pass_filename, get_cook_filename) {
            let content = zarchive.reader(info_filename).await?;
            let infos = InfoLogProcessor::parse(content);
            

            println!("Folder: {}\nInfo: {:?}",  folder, infos);
        };

    }

    println!("Elapsed at: {}", time.elapsed().as_millis());

    Ok(())
}
