/*
 * This module cannot import from conf
 *
 */
use std::collections::HashMap;

use backend::{Backend,RemoteBranch,BranchAheadBehind};
use constants::*;

use yaml_rust::{Yaml};


// struct for specific values should implement this trait
pub trait Display {
    fn display(&self) -> Option<String>;
}


fn substiute_special_values(s: String, values: &HashMap<String, String>) -> String {
    let mut r:String = s;
    for (k, v) in values {
        r = r.replace(k, &v);
    }
    r
}

pub fn format_value(pre_format: &str, post_format: &str, data: &str) -> String {
    format!("{}{}{}", pre_format, data, post_format)
}


#[derive(Debug, Clone)]
pub struct SimpleValue {
    pub value_type: String,
    pub pre_format: String,
    pub post_format: String,
}

impl SimpleValue {
    pub fn new(value_yaml: &Yaml) -> SimpleValue {
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
    fn new(simple_value: &SimpleValue, backend: &'a Backend, debug: bool) -> RepoStatus<'a> {
        RepoStatus{
            value: simple_value.clone(), backend: backend, debug: debug
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

    fn new(simple_value: &SimpleValue, backend: &'a Backend, debug: bool) -> FileStatus<'a> {
        FileStatus{
            value: simple_value.clone(), backend: backend, debug: debug
        }
    }
}


#[derive(Debug)]
pub struct RemoteTracking<'a> {
    remote_branch: Option<RemoteBranch>,
    display_if_uptodate: bool,
    debug: bool,
    backend: &'a  Backend,
    value: SimpleValue,
    values: Vec<SimpleValue>,

}

impl<'a> Display for RemoteTracking<'a> {
    fn display(&self) -> Option<String> {
        log!(self, "display remote_difference: {:?}", self);

        let a_b: BranchAheadBehind = match self.backend.get_branch_ahead_behind(
            self.remote_branch.clone()) {
            Some(x) => x,
            None => {
                panic!("no ahead behind stats found for = {:?}", self.remote_branch);
            },
        };
        let local_branch_name: String = match a_b.local_branch_name.clone() {
            Some(l) => l,
            None => {
                log!(self, "No local branch name.");
                "".to_string()
            }
        };
        let mut special_values: HashMap<String, String> = HashMap::new();
        special_values.insert("<LOCAL_BRANCH>".to_string(), local_branch_name.clone());
        match a_b.remote_branch_name.clone() {
            Some(v) => special_values.insert("<REMOTE_BRANCH>".to_string(), v),
            None => special_values.insert("<REMOTE_BRANCH>".to_string(), "".to_string()),
        };
        match a_b.remote_name.clone() {
            Some(v) => special_values.insert("<REMOTE>".to_string(), v),
            None => special_values.insert("<REMOTE>".to_string(), "".to_string()),
        };

        let mut response: String = "".to_string();
        for value in self.values.clone() {
            match self.display_value(value.clone(), a_b.clone(),
                                     special_values.clone()) {
                Some(s) => response += &s,
                None => (),
            }
        }
        if response.len() > 0 {
            Some(response)
        } else {
            None
        }
    }
}


impl<'a> RemoteTracking<'a> {
    fn new(value_yaml: &Yaml, simple_value: &SimpleValue,
           backend: &'a Backend, debug: bool) -> RemoteTracking<'a> {
        let remote_branch: Option<RemoteBranch> = match value_yaml["remote_branch"].as_str() {
            Some(s) => {
                let remote_branch_string = s.to_string();
                let v: Vec<&str> = remote_branch_string.splitn(2, "/").collect();
                if v.len() != 2 {
                    panic!("`remote_branch` must be in form of `<REMOTE>/<BRANCH>`");
                }
                Some(RemoteBranch{
                    remote_branch: remote_branch_string.clone(),
                    remote_name: v[0].to_string(),
                    remote_branch_name: v[1].to_string()
                })
            },
            None => {
                // log!(self, "remote_branch is not specified: {:?}", value_yaml);
                None
            },
        };
        let display_if_uptodate = match value_yaml["display_if_uptodate"].as_bool() {
            Some(b) => b,
            None => panic!("display_if_uptodate in {:?} is not specified", value_yaml),
        };
        let mut values: Vec<SimpleValue> = Vec::new();
        match value_yaml["values"].as_vec() {
            Some(v) => {
                for inner_value_yaml in v {
                    values.push(SimpleValue::new(inner_value_yaml));
                }
            },
            None => panic!("values is empty: {:?}", value_yaml),
        };
        RemoteTracking{
            value: simple_value.clone(), backend: backend, debug: debug,
            display_if_uptodate: display_if_uptodate, values: values,
            remote_branch: remote_branch
        }
    }

    fn display_name(&self, value: &SimpleValue, special_values: HashMap<String, String>) -> Option<String> {
        Some(format_value(
            &substiute_special_values(value.pre_format.clone(), &special_values),
            &substiute_special_values(value.post_format.clone(), &special_values),
            ""
        ))
    }

    fn display_ahead(&self, value: &SimpleValue, ahead: usize) -> Option<String> {
        if ahead > 0 {
            return Some(format_value(&value.pre_format, &value.post_format, &ahead.to_string()));
        }
        None
    }

    fn display_behind(&self, value: &SimpleValue, behind: usize) -> Option<String> {
        if behind > 0 {
            return Some(format_value(&value.pre_format, &value.post_format, &behind.to_string()));
        }
        None
    }

    fn display_value(&self, simple_value: SimpleValue, a_b: BranchAheadBehind,
                     special_values: HashMap<String, String>) -> Option<String> {
        if !(self.display_if_uptodate || a_b.ahead > 0 || a_b.behind > 0) {
            return None;
        }
        match simple_value.value_type.as_str() {
            "name" => self.display_name(&simple_value, special_values),
            "ahead" => self.display_ahead(&simple_value, a_b.ahead),
            "behind" => self.display_behind(&simple_value, a_b.behind),
            _ => panic!("Unknown value for remote_difference: {:?}", simple_value),
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

    pub fn display_value(&self, value_yaml: &Yaml, simple_value: &SimpleValue) -> Option<String> {
        let o: Option<String> = match simple_value.value_type.as_str() {
            "repository_state" => RepoStatus::new(&simple_value, &self.backend, self.debug).display(),
            "new" |
            "changed" |
            "staged" |
            "conflicts" => FileStatus::new(&simple_value, &self.backend, self.debug).display(),
            // separator is displayed in conf, pretty hacky
            // "separator" => Separator::new(&simple_value, self.debug).display(),
            "remote_difference" => RemoteTracking::new(value_yaml, &simple_value, &self.backend, self.debug).display(),
            _ => panic!("Unknown value type: {:?}", value_yaml)
        };
        o
    }
}
