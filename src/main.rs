mod archive;
mod log_processor;

use std::{fs::{File, OpenOptions}, io::Write, sync::{Arc, Mutex}};
use lazy_static::lazy_static;
use regex::Regex;


use crate::{
    archive::SupportedExtension,
    archive::Archive, 
    archive::z_archive::Zarchive,
    archive::ArchiveFilter,
    archive::ArchiveUtils,
    log_processor::{
        log_filter::LogFilter,
        cook_log_processor::CookieLogProcessor, 
        info_log_processor::InfoLogProcessor, 
        pass_log_processor::PassLogProcessor, 
        LogProcessor
    }
};

lazy_static! {
    static ref SYSTEM_INFO_REGEX: Regex = Regex::new(r"(?i)((system)|(info))").unwrap();
    static ref PASSWORD_REGEX: Regex = Regex::new(r"(?i)(pass)").unwrap();
    static ref COOKIES_REGEX: Regex = Regex::new(r"(?i)(cookies)").unwrap();
}

#[tokio::main]
async fn main() -> tokio::io::Result<()>{
    let time = std::time::Instant::now();
    
    let select_file = rfd::AsyncFileDialog::new()
    .set_directory("/")
    .pick_file().await;

    let file = select_file.expect("cannot open file");
    
    let filename = file.path();

    ArchiveUtils::verify_existence(&filename)?;

    let extension = ArchiveUtils::verify_extension(&filename)?;

    let mut archive = match extension {
        SupportedExtension::Zip => Ok(Zarchive::new(&filename).await),
        SupportedExtension::Unsupported => Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Unsupported File"))
    }??;

    let filehash = ArchiveUtils::generate_hash(&filename)?;

    let output = Arc::new(Mutex::new(OpenOptions::new()
    .create(true)
    .append(true)
    .write(true)
    .open(&filehash)?));

    let mut filter = LogFilter::new(
        Some(vec![
            Regex::new(r"(?i)((pass)|(system)|(info)|(cookies))").unwrap()
        ]
    ), Some(vec![
        String::from("txt")
    ]));

    let content = archive.enumerate(filter.clone());
    let logs = filter.relation_mapper(content).to_owned();
    let mut tasks = Vec::new();
    
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
            let info = Arc::new(info_processor.parse(content));

            if let Ok(content) = archive.reader(passw_filename) {
                let info = info.clone();

                let passw_task = tokio::spawn(async move {
                    let passw_processor = PassLogProcessor::new(&info);
                    let passw_parser = passw_processor.parse(&content);
                    
                    if !passw_parser.is_empty() {
                        
                    }
                });

                tasks.push(passw_task)       
            }

            if !get_cooks_filename.is_empty() {
                for item in get_cooks_filename {
                    let c_output = output.clone();

                    let info = info.clone();
                    if let Ok(content) = archive.reader(&item) {
                        let cookie_task = tokio::spawn(async move {
                            let cookie_processor = CookieLogProcessor::new(&info);
                            let mut lock_output = c_output.lock().unwrap();

                            match cookie_processor.parse(content) {
                                Ok(result) => {
                                    match result.get("mercadopago.com.br") {
                                        Some(value) => {
                                            let cookie = value.cookies();
                                            let to_string = serde_json::to_string(&cookie).unwrap();

                                            if let Err(write_err) = writeln!(&mut lock_output, "{}", to_string) {
                                                eprintln!("{}", write_err)
                                            };
                                        },
                                        None => {},
                                    }
                                },
                                Err(err) => {
                                    
                                }
                            };
                        });

                        tasks.push(cookie_task)
                    }
                }
            }
        }
        
    };

    for task in tasks {
        task.await?
    }

    println!("Elapsed at: {}", time.elapsed().as_millis());
    
    Ok(())
}
