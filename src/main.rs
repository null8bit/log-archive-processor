mod archive;
mod log_processor;

use std::{io::Result, ops::Deref, sync::{Arc, Mutex}};

use archive::{z_archive::Zarchive, ArchiveFilter};
use lazy_static::lazy_static;
use log_processor::log_filter::LogFilter;

use regex::Regex;

use crate::{archive::Archive, log_processor::{info_log_processor::InfoLogProcessor, pass_log_processor::PassLogProcessor, LogProcessor}};

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
    
    for (_, filenames) in relationed {
        let mut filenames = filenames.iter();
        let get_infos_filename = filenames.find(|filename| SYSTEM_INFO_REGEX.is_match(&filename));
        let get_passw_filename = filenames.find(|filename| PASSWORD_REGEX.is_match(&filename));
        let get_cooks_filename: Vec<_> = filenames
        .filter(|filename| COOKIES_REGEX.is_match(&filename))
        .collect();

        if let (Some(infos_filename), Some(passw_filename)) = (get_infos_filename, get_passw_filename) {
            let Ok(content) = zarchive.reader(&infos_filename).await else {
                println!("[-] Cannot read file");
                continue;
            };
            let info_processor = InfoLogProcessor::new();
            let info = info_processor.parse(content);

            if let Ok(content) = zarchive.reader(&passw_filename).await {
                let passw_processor = PassLogProcessor::new(&info);
                let passw_parser = passw_processor.parse(content);

                println!("[+] total passwords => {}", passw_parser.len())
            }
        };

    }
    //  {

    //     
        
    //      {
    //         
    //         
    //             let mut z_archive_instance2 = Arc::into_inner(z_archive_clone2).expect("cannot recover").into_inner().unwrap();


    //             

    //             Ok(infos)
    //         }).await?;

    //         let Ok(infos) = infos_parser else {
    //             eprintln!("[-] Infos parsing error");
    //             continue;
    //         };

    //         println!("{:?}", infos);
    //     }

    // }

    println!("Elapsed at: {}", time.elapsed().as_millis());

    Ok(())
}
