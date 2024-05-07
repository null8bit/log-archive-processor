mod archive;
mod log_processor;

use std::{io::Result, ops::Deref, sync::{Arc, Mutex}};

use archive::{z_archive::Zarchive, ArchiveFilter, ArchiveUtils};
use lazy_static::lazy_static;
use log_processor::log_filter::LogFilter;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;

use crate::{archive::Archive, log_processor::{info_log_processor::InfoLogProcessor, pass_log_processor::PassLogProcessor, LogProcessor}};

lazy_static! {
    static ref SYSTEM_INFO_REGEX: Regex = Regex::new(r"(?i)((system)|(info))").unwrap();
    static ref PASSWORD_REGEX: Regex = Regex::new(r"(?i)(pass)").unwrap();
    static ref COOKIES_REGEX: Regex = Regex::new(r"(?i)(cookies)").unwrap();
}

#[tokio::main]
async fn main() -> tokio::io::Result<()>{
    let time = std::time::Instant::now();
    let filename = "C:\\Users\\conta\\Downloads\\Telegram Desktop\\2024-04-23_b_@logsinspector.zip";

    let extension = ArchiveUtils::verify_extension(&filename)?;

    let mut archive = match extension {
        archive::SupportedExtension::Zip => Ok(Zarchive::new(&filename).await),
        archive::SupportedExtension::Unsupported => Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Unsupported File"))
    }??;

    let mut filter = LogFilter::new(
        Some(vec![
            Regex::new(r"(?i)((pass)|(system)|(info)|(cookies))").unwrap()
        ]
    ), Some(vec![
        String::from("txt")
    ]));

    let content = archive.enumerate(filter.clone());
    let logs = filter.relation_mapper(content).to_owned();

    for (_, filenames) in logs {
        let mut filenames_into_iter = filenames.iter();

        let get_infos_filename = filenames_into_iter.find(|filename| SYSTEM_INFO_REGEX.is_match(&filename));
        let get_passw_filename = filenames_into_iter.find(|filename| PASSWORD_REGEX.is_match(&filename));
        let get_cooks_filename: Vec<_> = filenames_into_iter.filter(|filename| COOKIES_REGEX.is_match(&filename)).collect();

        if let (Some(infos_filename), Some(passw_filename)) = (get_infos_filename, get_passw_filename) {
            let Ok(content)  = archive.reader(infos_filename) else {
                eprintln!("{}", tokio::io::Error::new(tokio::io::ErrorKind::InvalidData, "Cannot Read Info File"));
                continue;
            };

            let info_processor = InfoLogProcessor::new();
            let info = info_processor.parse(content);

            if let Ok(content) = archive.reader(passw_filename) {
                let passw_processor = PassLogProcessor::new(&info);
                let passw_parser = passw_processor.parse(&content);

                
            }
        }
    };
    /* 
    let z_archive = Zarchive::new().await?;


    

    

    
    
     {
        let filenames = Arc::new(filenames.clone());

        tokio::task::spawn(async move {
            let filenames = filenames.deref().clone();
            
            let mut z_archive = z_archive_clone.deref();


                



            // };
        }).await?;
    }


    println!("Elapsed at: {}", time.elapsed().as_millis());
    */
    Ok(())
}
