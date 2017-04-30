/*
 * This module cannot import from conf
 *
 */
use backend::Backend;

use yaml_rust::{YamlLoader, Yaml};


// struct for specific values should implement this trait
pub trait Display {
    fn display(&self) -> Option<String>;
}


fn format_value(pre_format: &str, post_format: &str, data: &str) -> String {
    format!("{}{}{}", pre_format, data, post_format)
}


#[derive(Debug)]
pub struct RepoStatus<'a> {
    debug: bool,
    backend: &'a Backend,
    value_type: String,
    pre_format: String,
    post_format: String,
}

impl<'a> Display for RepoStatus<'a> {
    fn display(&self) -> Option<String> {
        log!(self, "display repository state, value: {:?}", self);
        let repo_state = self.backend.get_repository_state();
        // TODO: implement configuration for is_empty
        if !repo_state.is_empty() {
            return Some(format_value(&self.pre_format, &self.post_format, &repo_state));
        }
        None
    }
}

impl<'a> RepoStatus<'a> {
    fn new(value_yaml: &Yaml, backend: &'a Backend, debug: bool) -> RepoStatus<'a> {
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
        RepoStatus{
            value_type: value_type, pre_format: pre_format, post_format: post_format,
            backend: backend, debug: debug
        }
    }
}

// #[derive(Debug, Clone)]
// pub struct SimpleValue {
//     pub value_type: String,
//     pub pre_format: String,
//     pub post_format: String,
// }
// 
// impl SimpleValue {
//     fn new(value_yaml: &Yaml) -> SimpleValue {
//         let value_type = match value_yaml["type"].as_str() {
//             Some(s) => s.to_string(),
//             None => panic!("value_type in {:?} is not specified", value_yaml),
//         };
//         let pre_format = match value_yaml["pre_format"].as_str() {
//             Some(s) => s.to_string(),
//             None => panic!("pre_format in {:?} is not specified", value_yaml),
//         };
//         let post_format = match value_yaml["post_format"].as_str() {
//             Some(s) => s.to_string(),
//             None => panic!("post_format in {:?} is not specified", value_yaml),
//         };
//         Value{
//             value_type: value_type, pre_format: pre_format, post_format: post_format,
//         }
//     }
// }


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
            _ => None,  // panic!("Unknown value type: {:?}", value_yaml)
        };
        o
    }
}


// impl DisplayMaster {
//     // get # of files for specific type
//     fn get_file_status_for_type(&self, t: &str) -> Option<u32> {
//         if let Some(s) = self.backend.get_file_status() {
//             if !s.is_empty() {
//                 return s.get(t).cloned()
//             }
//         }
//         None
//     }
// 
// 
//     fn display_new(&self, value: &Value) -> Option<String> {
//         if let Some(x) = self.get_file_status_for_type(NEW_KEY) {
//             return Some(format_value(value, &format!("{}", x)));
//         }
//         None
//     }
// 
//     fn display_changed(&self, value: &Value) -> Option<String> {
//         if let Some(x) = self.get_file_status_for_type(CHANGED_KEY) {
//             return Some(format_value(value, &format!("{}", x)));
//         }
//         None
//     }
// 
//     fn display_staged(&self, value: &Value) -> Option<String> {
//         if let Some(x) = self.get_file_status_for_type(STAGED_KEY) {
//             return Some(format_value(value, &format!("{}", x)));
//         }
//         None
//     }
// 
//     fn display_conflicts(&self, value: &Value) -> Option<String> {
//         if let Some(x) = self.get_file_status_for_type(CONFLICTS_KEY) {
//             return Some(format_value(value, &format!("{}", x)));
//         }
//         None
//     }
// 
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


