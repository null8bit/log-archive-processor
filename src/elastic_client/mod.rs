use std::{
    borrow::Borrow,
    io::{Error, Result},
};

use elasticsearch::{
    http::transport::Transport,
    indices::{IndicesCreateParts, IndicesExistsParts},
    BulkIndexOperation, BulkOperation, BulkParts, Elasticsearch, IndexParts,
};
use serde::{Serialize, Serializer};
use serde_json::{json, Value};

pub struct ElasticIndexMapping {
    name: String,
    map: Value,
}

impl ElasticIndexMapping {
    pub fn new<I: AsRef<str>>(index: I, map: Value) -> Self {
        Self {
            name: index.as_ref().to_string(),
            map,
        }
    }

    pub fn mapping(&self) -> Value {
        self.map.clone()
    }

    pub fn index(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct ElasticsearchClient {
    client: Elasticsearch,
}

impl ElasticsearchClient {
    pub async fn new() -> Result<Self> {
        let transport = Transport::single_node("http://127.0.0.1:9200").unwrap();
        let client = Elasticsearch::new(transport);
        let result = client.cat().health().send().await;

        match result {
            Ok(response) => {
                if !response.status_code().is_success() {
                    return Err(Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        response.status_code().as_str(),
                    ));
                }
            }
            Err(err) => {
                return Err(Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    err.to_string(),
                ))
            }
        }
        Ok(Self { client })
    }

    async fn indice_exists(&self, indice: &str) -> bool {
        let client = self.client.clone();

        return match client
            .indices()
            .exists(IndicesExistsParts::Index(&[indice]))
            .send()
            .await
        {
            Ok(result) => {
                if result.status_code().is_success() {
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        };
    }

    pub async fn create_indice(&self, mapping: ElasticIndexMapping) -> Result<()> {
        let client = self.client.clone();

        if !self.indice_exists(&mapping.name).await {
            client
                .indices()
                .create(IndicesCreateParts::Index(&mapping.name))
                .send()
                .await
                .unwrap();
        }

        Ok(())
    }

    pub async fn insert_many<R: Serialize, D: IntoIterator<Item = R>>(
        &self,
        index_name: &str,
        documents: D,
    ) {
        let client = self.client.clone();

        let operations = documents
            .into_iter()
            .map(|item| {
                let value = serde_json::to_value(item).unwrap();

                BulkOperation::index(value).into()
            })
            .collect::<Vec<BulkOperation<Value>>>();

        let _result = client
            .bulk(BulkParts::Index(index_name))
            .body(operations)
            .send()
            .await;

        // match result {
        //     Ok(response) => {
        //         let content = response.status_code();

        //         println!("[+] Insertion result -> {:?}", content.canonical_reason())
        //     },
        //     Err(err) => {
        //         eprintln!("[-] Insertion error -> {}", err.to_string())
        //     },
        // }
    }
}
