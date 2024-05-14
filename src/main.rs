mod archive;
mod elastic_client;
mod log_processor;

use lazy_static::lazy_static;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use serde_json::{json, Value};
use std::{
    fs::OpenOptions, sync::{Arc, Mutex}
};

use crate::{
    archive::{z_archive::Zarchive, Archive, ArchiveFilter, ArchiveUtils, SupportedExtension},
    elastic_client::{ElasticIndexMapping, ElasticsearchClient},
    log_processor::{
        cook_log_processor::CookieLogProcessor, info_log_processor::InfoLogProcessor,
        log_filter::LogFilter, pass_log_processor::PassLogProcessor, LogProcessor,
    },
};

lazy_static! {
    static ref SYSTEM_INFO_REGEX: Regex = Regex::new(r"(?i)((system)|(info))").unwrap();
    static ref PASSWORD_REGEX: Regex = Regex::new(r"(?i)(pass)").unwrap();
    static ref COOKIES_REGEX: Regex = Regex::new(r"(?i)(cookies)").unwrap();
}


#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let elastic_cookies_mapping = ElasticIndexMapping::new(
        "cookies",
        json!({
          "mappings": {
            "properties": {
              "country": {
                "type": "keyword"
              },
              "domain": {
                "type": "keyword"
              },
              "cookies": {
                "type": "nested",
                "properties": {
                  "name": {
                    "type": "keyword"
                  },
                  "value": {
                    "type": "text"
                  }
                }
              }
            }
          }
        }),
    );
    let elastic_credentials_mapping = ElasticIndexMapping::new(
        "credentials",
        json!({
            "mappings": {
                "properties": {
                    "url": {"type": "keyword"},
                    "username": {"type": "keyword"},
                    "password": {"type": "keyword"}
                }
            }
        }),
    );

    let elastic = Arc::new(ElasticsearchClient::new().await?);


    let (tx_cookies, mut rx_cookies) = tokio::sync::mpsc::channel::<Vec<Value>>(4096);
    let (tx_passwd, mut rx_passwd) = tokio::sync::mpsc::channel::<Vec<Value>>(4096);
    let time = std::time::Instant::now();

    let select_file = rfd::AsyncFileDialog::new()
        .set_directory("/")
        .pick_file()
        .await;

    let file = select_file.expect("cannot open file");

    let filename = file.path();

    ArchiveUtils::verify_existence(&filename)?;

    let extension = ArchiveUtils::verify_extension(&filename)?;

    let mut archive = match extension {
        SupportedExtension::Zip => Ok(Zarchive::new(&filename).await),
        SupportedExtension::Unsupported => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Unsupported File",
        )),
    }??;

    // let filehash = ArchiveUtils::generate_hash(&filename)?;

    // let _output = Arc::new(Mutex::new(
    //     OpenOptions::new()
    //         .create(true)
    //         .append(true)
    //         .write(true)
    //         .open(format!("{}.txt", &filehash))?,
    // ));

    let mut filter = LogFilter::new(
        Some(vec![
            Regex::new(r"(?i)((pass)|(system)|(info)|(cookies))").unwrap()
        ]),
        Some(vec![String::from("txt")]),
    );

    let content = archive.enumerate(filter.clone());
    let logs = filter.relation_mapper(content).to_owned();
    let mut tasks = Vec::new();

    let c_elastic = elastic.clone();

    let _= c_elastic.create_indice(elastic_cookies_mapping).await;
    let _= c_elastic.create_indice(elastic_credentials_mapping).await;  
     
    for (_, filenames) in logs {
        let mut filenames_into_iter = filenames.iter();

        let get_infos_filename =
            filenames_into_iter.find(|filename| SYSTEM_INFO_REGEX.is_match(&filename));
        let get_passw_filename =
            filenames_into_iter.find(|filename| PASSWORD_REGEX.is_match(&filename));
        let get_cooks_filename: Vec<_> = filenames_into_iter
            .filter(|filename| COOKIES_REGEX.is_match(&filename))
            .collect();

        if let (Some(infos_filename), Some(passw_filename)) =
            (get_infos_filename, get_passw_filename)
        {
            let Ok(content) = archive.reader(infos_filename) else {
                eprintln!(
                    "{}",
                    tokio::io::Error::new(
                        tokio::io::ErrorKind::InvalidData,
                        "Cannot Read Info File"
                    )
                );
                continue;
            };

            let info_processor = InfoLogProcessor::new();
            let info = Arc::new(info_processor.parse(content));

            if let Ok(content) = archive.reader(passw_filename) {
                let info = info.clone();
                
                let sender = tx_passwd.clone();
                let passw_task = tokio::spawn(async move {
                    let passw_processor = PassLogProcessor::new(&info);
                    let passw_parser = passw_processor.parse(&content)
                    .par_iter()
                    .map(|item| item.to_owned())
                    .map(|item| serde_json::to_value(item))
                    .filter_map(|item| item.ok())
                    .collect::<Vec<_>>();

                    let _=sender.send(passw_parser).await;
                });

                tasks.push(passw_task)
            }

            if !get_cooks_filename.is_empty() {
                for item in get_cooks_filename {
                    let sender = tx_cookies.clone();
                    let info = info.clone();
                    if let Ok(content) = archive.reader(&item) {

                        let cookie_task = tokio::spawn(async move {
                            let cookie_processor = CookieLogProcessor::new(&info);

                            match cookie_processor.parse(content) {
                                Ok(parsed) => {
                                    let document = parsed.values()
                                    .map(|item| item.to_owned())
                                    .map(|item| serde_json::to_value(item))
                                    .filter_map(|result| result.ok())
                                    .collect::<Vec<_>>();

                                    let _=sender.send(document).await;
                                },
                                Err(err) => {
                                    eprintln!("[-] Cookie parse error {}", err)
                                },
                            }
                        });

                        tasks.push(cookie_task)
                    }
                }
            }
        }
    }

    let c2_elastic = elastic.clone();

    let receiver_cookies = tokio::task::spawn(async move {
       while let Some(data) = rx_cookies.recv().await {
        println!("{:?}", data);

           c2_elastic.insert_many("cookies", data).await;
       }
    });

    tasks.push(receiver_cookies);

    let c3_elastic = elastic.clone();

    let receiver_passwd = tokio::task::spawn(async move {

       while let Some(data) = rx_passwd.recv().await {
            println!("{:?}", data);
           c3_elastic.insert_many("credentials", data).await;
       }
    });

    tasks.push(receiver_passwd);

    for task in tasks {
        task.await?
    }

    println!("Elapsed at: {}", time.elapsed().as_millis());
 
    Ok(())
}
