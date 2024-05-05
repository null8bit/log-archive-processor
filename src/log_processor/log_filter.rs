use std::{collections::HashMap, path::Path};

use crate::archive::{ArchiveFilter, FilterOptions};

type RelationMap = HashMap<String, Vec<String>>;

#[derive(Clone)]
pub struct LogFilter {
    options: FilterOptions,
    relation_map: RelationMap
}

impl ArchiveFilter for LogFilter {
    type Options = FilterOptions;
    
    fn new<F: IntoIterator<Item = regex::Regex> , E: IntoIterator<Item = String>>(name: Option<F>, ext: Option<E>) -> Self {
        let options = match (name, ext) {
            (Some(regex), Some(extension)) => FilterOptions::new(Some(regex.into_iter().collect()), Some(extension.into_iter().collect())),
            (None, None) => FilterOptions::new(None, None),
            (None, Some(_)) => FilterOptions::new(None, None),
            (Some(regex), None) => FilterOptions::new(Some(regex.into_iter().collect()), None),
        };

        Self {options, relation_map: HashMap::new()}
    }

    fn archive_filter(&self, item: &str) -> bool {
        let options = &self.options;
        let path = Path::new(item);
        let is_dir = path.is_dir();

        if is_dir {
            return false
        };

        return match (options.get_extension(), options.get_regex()) {
            (None, None) => true,
            (None, Some(regexs)) => {
                regexs.iter().all(|re| re.is_match(item))
            },
            (Some(exts), None) => exts.iter().any(|e| item.ends_with(e)),
            (Some(exts), Some(regexs)) => {
                regexs.iter().all(|re| re.is_match(item)) && exts.iter().any(|e| item.ends_with(e))
            },
        }
    }
 
}

impl LogFilter {
    pub fn extract_log_folder(path: &str) -> String {
        path.split("/").collect::<Vec<_>>().first().unwrap().to_string()
    }
    pub fn relation_mapper<T: AsRef<str>>(&mut self, list: Vec<T>) -> &RelationMap {

        list.iter().for_each(|item| {
            let item_as_ref = item.as_ref();
            let log_folder = Self::extract_log_folder(item_as_ref);
            self.relation_map.entry(log_folder).or_insert(Vec::new());
        });
        
        list.iter().for_each(|item| {
            let item_as_ref = item.as_ref();
            let log_folder = Self::extract_log_folder(item_as_ref);
            let get_hash_value = self.relation_map.get_mut(&log_folder);
            if let Some(value) = get_hash_value {
                value.push(item_as_ref.to_string())
            }
        });
        
        &self.relation_map
    }
}

