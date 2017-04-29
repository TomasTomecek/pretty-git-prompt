extern crate clap;
extern crate git2;
extern crate yaml_rust;

use backend::Backend;
use cli::cli;
use conf::{RemoteBranch};
use conf::{Conf,get_configuration,create_default_config,Value};
use constants::*;

use std::collections::HashMap;

use git2::Repository;

// util mod def needs to be first b/c of macro definitions and usage in other modules
#[macro_use]
mod util;
mod backend;
mod cli;
mod conf;
mod constants;


fn format_value(value: &Value, data: &str) -> String {
    format!("{}{}{}", value.pre_format, data, value.post_format)
}

fn substiute_special_values(s: String, values: &HashMap<String, String>) -> String {
    let mut r:String = s;
    for (k, v) in values {
        r = r.replace(k, &v);
    }
    r
}


// is able to display values, which makes him a master
struct DisplayMaster {
    backend: Backend,
    debug: bool,
}

impl DisplayMaster {
    pub fn new(backend: Backend, debug: bool) -> DisplayMaster {
        DisplayMaster { backend: backend, debug: debug }
    }

    fn display_repository_state(&self, value: &Value) -> Option<String> {
        log!(self, "display repository state, value: {:?}", value);
        let repo_state = self.backend.get_repository_state();
        // TODO: implement configuration for is_empty
        if !repo_state.is_empty() {
            return Some(format_value(value, &repo_state));
        }
        None
    }

    // get # of files for specific type
    fn get_file_status_for_type(&self, t: &str) -> Option<u32> {
        if let Some(s) = self.backend.get_file_status() {
            if !s.is_empty() {
                return s.get(t).cloned()
            }
        }
        None
    }


    fn display_new(&self, value: &Value) -> Option<String> {
        if let Some(x) = self.get_file_status_for_type(NEW_KEY) {
            return Some(format_value(value, &format!("{}", x)));
        }
        None
    }

    fn display_changed(&self, value: &Value) -> Option<String> {
        if let Some(x) = self.get_file_status_for_type(CHANGED_KEY) {
            return Some(format_value(value, &format!("{}", x)));
        }
        None
    }

    fn display_staged(&self, value: &Value) -> Option<String> {
        if let Some(x) = self.get_file_status_for_type(STAGED_KEY) {
            return Some(format_value(value, &format!("{}", x)));
        }
        None
    }

    fn display_conflicts(&self, value: &Value) -> Option<String> {
        if let Some(x) = self.get_file_status_for_type(CONFLICTS_KEY) {
            return Some(format_value(value, &format!("{}", x)));
        }
        None
    }

    // display selected value
    fn display(&self, value: &Value) -> Option<String> {
        let value_type: &str = &value.value_type;
        match value_type {
            "repository_state" => self.display_repository_state(value),
            "new" => self.display_new(value),
            "changed" => self.display_changed(value),
            "staged" => self.display_staged(value),
            "conflicts" => self.display_conflicts(value),
            _ => None,  // panic!("Unknown value type: {:?}", value)
        }
    }
}


// logic of the whole program -- the glue
struct Program {
    conf: Conf,
    display_master: DisplayMaster,
    debug: bool
}

impl Program {
    pub fn new(conf: Conf, dm: DisplayMaster, debug: bool) -> Program {
        Program { conf: conf, display_master: dm, debug: debug }
    }

    // fn add_monitored_branches_state(&mut self) {
    //     let mr = match self.conf.get_remotes_monitoring() {
    //         Some(x) => x,
    //         None => return (),
    //     };
    //     for monitored_remote in mr {
    //         let rb: Option<RemoteBranch> = monitored_remote.remote_branch;
    //         let a_b = match self.backend.get_branch_ahead_behind(rb.clone()) {
    //             Some(x) => x,
    //             None => {
    //                 log!(self, "no ahead behind stats found for = {:?}", rb);
    //                 continue
    //             },
    //         };
    //         let local_branch_name = match a_b.local_branch_name {
    //             Some(l) => l,
    //             None => {
    //                 log!(self, "No local branch name.");
    //                 "".to_string()
    //             }
    //         };
    //         if monitored_remote.display_if_uptodate || a_b.ahead > 0 || a_b.behind > 0 {
    //             if let (Some(a_v), Some(b_v)) = (
    //                 self.conf.get_difference_ahead_value(),
    //                 self.conf.get_difference_behind_value()
    //             ) {
    //                 let mut special_values: HashMap<String, String> = HashMap::new();
    //                 special_values.insert("<LOCAL_BRANCH>".to_string(), local_branch_name);
    //                 match a_b.remote_branch_name {
    //                     Some(v) => special_values.insert("<REMOTE_BRANCH>".to_string(), v),
    //                     None => special_values.insert("<REMOTE_BRANCH>".to_string(), "".to_string()),
    //                 };
    //                 match a_b.remote_name {
    //                     Some(v) => special_values.insert("<REMOTE>".to_string(), v),
    //                     None => special_values.insert("<REMOTE>".to_string(), "".to_string()),
    //                 };
    //                 let mut local: String = format!(
    //                     "{}{}",
    //                     substiute_special_values(monitored_remote.pre_format, &special_values),
    //                     substiute_special_values(monitored_remote.post_format, &special_values),
    //                 );
    //                 if a_b.ahead > 0 {
    //                     local += &format_value(a_v, &a_b.ahead.to_string());
    //                 }
    //                 if a_b.behind > 0 {
    //                     local += &format_value(b_v, &a_b.behind.to_string());
    //                 }
    //                 self.out.push(local);
    //             }
    //         }
    //     }
    // }

    fn display_values(&mut self) {
        let mut out: Vec<String> = Vec::new();
        let values = match self.conf.get_values() {
            Some(v) => v,
            None => panic!("No values present in configuration, nothing to display."),
        };
        for value in values {
            match self.display_master.display(&value) {
                Some(v) => out.push(v),
                None => ()
            };
        }
        self.output(out);
    }

    // print output buffer
    fn output(&self, out: Vec<String>) {
        log!(self, "# of blocks = {}", out.len());
        let output = out.join("|");
        if self.debug {
            println!("'{}'", output);
        } else {
            println!("{}", output);
        }
    }
}

fn main() {
    let app = cli();
    let matches = app.get_matches();

    let debug_enabled = matches.is_present("debug");
    if debug_enabled { println!("Debug messages are enabled."); }

    let repo = match Repository::discover(".") {
        Ok(repo) => repo,
        // not a git repository, ignore
        Err(e) => {
            if debug_enabled { println!("This is not a git repository: {:?}", e); }
            return ();
        }
    };

    let conf_path: Option<String> = if matches.is_present("config") {
        Some(String::from(matches.value_of("config").unwrap()))
    } else {
        None
    };

    let conf = get_configuration(conf_path);

    if matches.is_present(CLI_DEFAULT_CONFIG_SUBC_NAME) {
        let p = get_default_config_path();
        match create_default_config(p.clone()) {
            Ok(path) => {
                println!("Configuration file created at \"{}\"", path);
                return ();
            }
            Err(e) => {
                println!("Failed to create configuration file \"{}\": {}", p.to_str().unwrap(), e);
                return ();
            }
        };
    }

    let backend = Backend::new(repo, debug_enabled);
    let dm = DisplayMaster::new(backend, debug_enabled);
    let mut output = Program::new(conf, dm, debug_enabled);

    output.display_values();
}
