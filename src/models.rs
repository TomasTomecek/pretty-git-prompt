/*
 * This module cannot import from conf
 *
 */
use std::collections::HashMap;

use backend::Backend;
use constants::*;

use yaml_rust::{YamlLoader, Yaml};


// struct for specific values should implement this trait
pub trait Display {
    fn display(&self) -> Option<String>;
}


fn format_value(pre_format: &str, post_format: &str, data: &str) -> String {
    format!("{}{}{}", pre_format, data, post_format)
}


#[derive(Debug, Clone)]
pub struct SimpleValue {
    pub value_type: String,
    pub pre_format: String,
    pub post_format: String,
}

impl SimpleValue {
    fn new(value_yaml: &Yaml) -> SimpleValue {
        let value_type = match value_yaml["type"].as_str() {
            Some(s) => s.to_string(),
            None => panic!("value_type in {:?} is not specified", value_yaml),
        };
        let pre_format = match value_yaml["pre_format"].as_str() {
            Some(s) => s.to_string(),
            None => panic!("pre_format in {:?} is not specified", value_yaml),
        };
        let post_format = match value_yaml["post_format"].as_str() {
            Some(s) => s.to_string(),
            None => panic!("post_format in {:?} is not specified", value_yaml),
        };
        SimpleValue{
            value_type: value_type, pre_format: pre_format, post_format: post_format,
        }
    }
}


#[derive(Debug)]
pub struct RepoStatus<'a> {
    debug: bool,
    backend: &'a Backend,
    value: SimpleValue,
}

impl<'a> Display for RepoStatus<'a> {
    fn display(&self) -> Option<String> {
        log!(self, "display repository state, value: {:?}", self);
        let repo_state = self.backend.get_repository_state();
        // TODO: implement configuration for is_empty
        if !repo_state.is_empty() {
            return Some(format_value(&self.value.pre_format,
                                     &self.value.post_format, &repo_state));
        }
        None
    }
}

impl<'a> RepoStatus<'a> {
    fn new(value_yaml: &Yaml, backend: &'a Backend, debug: bool) -> RepoStatus<'a> {
        let simple_value = SimpleValue::new(value_yaml);
        RepoStatus{
            value: simple_value, backend: backend, debug: debug
        }
    }
}


#[derive(Debug)]
pub struct FileStatus<'a> {
    debug: bool,
    backend: &'a Backend,
    value: SimpleValue,
}

impl<'a> Display for FileStatus<'a> {
    fn display(&self) -> Option<String> {
        log!(self, "display file state, value: {:?}", self);
        if let Some(x) = self.get_file_status_for_type(&self.value.value_type) {
            return Some(format_value(&self.value.pre_format,
                                     &self.value.post_format,
                                     &format!("{}", x)));
        }
        None
    }
}

impl<'a> FileStatus<'a> {
    // get # of files for specific type
    fn get_file_status_for_type(&self, file_type: &str) -> Option<u32> {
        let mut h: HashMap<String, &str> = HashMap::new();
        h.insert("new".to_string(), NEW_KEY);
        h.insert("changed".to_string(), CHANGED_KEY);
        h.insert("staged".to_string(), STAGED_KEY);
        h.insert("conflicts".to_string(), CONFLICTS_KEY);
        if let Some(s) = self.backend.get_file_status() {
            match h.get(file_type.clone()) {
                Some(v) => return s.get(v.clone()).cloned(),
                None => panic!("Invalid name for file status: {}", file_type)
            };
        }
        None
    }

    fn new(value_yaml: &Yaml, backend: &'a Backend, debug: bool) -> FileStatus<'a> {
        let simple_value = SimpleValue::new(value_yaml);
        FileStatus{
            value: simple_value, backend: backend, debug: debug
        }
    }
}


// this struct is master of structs which implement Display trait
// -- a true master
pub struct DisplayMaster {
    backend: Backend,
    debug: bool,
}

impl DisplayMaster {
    pub fn new(backend: Backend, debug: bool) -> DisplayMaster {
        DisplayMaster { backend: backend, debug: debug }
    }

    pub fn display_value(&self, value_yaml: &Yaml) -> Option<String> {
        let value_type = match value_yaml["type"].as_str() {
            Some(s) => s,
            None => panic!("value_type in {:?} is not specified", value_yaml),
        };

        let o: Option<String> = match value_type {
            "repository_state" => RepoStatus::new(value_yaml, &self.backend, self.debug).display(),
            "new" |
            "changed" |
            "staged" |
            "conflicts" => FileStatus::new(value_yaml, &self.backend, self.debug).display(),
            _ => None,  // panic!("Unknown value type: {:?}", value_yaml)
        };
        o
    }
}


//     // display selected value
//     fn display(&self, value: &Value) -> Option<String> {
//         let value_type: &str = &value.value_type;
//         match value_type {
//             "repository_state" => self.display_repository_state(value),
//             "new" => self.display_new(value),
//             "changed" => self.display_changed(value),
//             "staged" => self.display_staged(value),
//             "conflicts" => self.display_conflicts(value),
//             _ => None,  // panic!("Unknown value type: {:?}", value)
//         }
//     }
// }
