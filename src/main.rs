extern crate clap;
extern crate git2;
extern crate yaml_rust;

use backend::Backend;
use cli::cli;
use conf::{Conf,get_configuration,create_default_config,Value,MonitoredRemote};
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


fn format_value(value: Value, data: String) -> String {
    format!("{}{}{}{}", value.pre_format, value.label, data, value.post_format)
}

fn substiute_special_values(s: String, values: &HashMap<String, String>) -> String {
    let mut r:String = s.clone();
    for (k, v) in values {
        r = r.replace(k, &v);
    }
    r
}


// struct for displaying all the data -- the actual program
struct Program {
    out: Vec<String>,
    backend: Backend,
    conf: Conf,
    debug: bool
}

impl Program {
    pub fn new(backend: Backend, conf: Conf, debug: bool) -> Program {
        let out: Vec<String> = Vec::new();
        Program { out: out, backend: backend, conf: conf, debug: debug }
    }

    // add repository state to output buffer
    fn add_repository_state(&mut self) {
        let repo_state = self.backend.get_repository_state();
        if !repo_state.is_empty() {
            self.out.push(repo_state);
        }
    }

    fn add_monitored_branches_state(&mut self) {
        let mut local_branch: String = self.backend.get_current_branch_name();
        let mr = match self.conf.get_remotes_monitoring() {
            Some(x) => x,
            None => return (),
        };
        for monitored_remote in mr {
            let b = match monitored_remote.branch {
                Some(x) => x,
                None => local_branch.clone()
            };
            let (ahead, behind) = match self.backend.get_remote_branch_ahead_behind(&monitored_remote.remote_name, &b) {
                Some(x) => x,
                None => (0, 0),
            };
            if monitored_remote.display_if_uptodate || ahead > 0 || behind > 0 {
                if let (Some(a_v), Some(b_v)) = (
                    self.conf.get_difference_ahead_value(),
                    self.conf.get_difference_behind_value()
                ) {
                    let mut special_values: HashMap<String, String> = HashMap::new();
                    special_values.insert("<BRANCH>".to_string(), b);
                    special_values.insert("<REMOTE>".to_string(), monitored_remote.remote_name);
                    let mut local: String = format!(
                        "{}{}",
                        substiute_special_values(monitored_remote.pre_format, &special_values),
                        substiute_special_values(monitored_remote.post_format, &special_values),
                    );
                    if ahead > 0 {
                        local += &format_value(a_v, ahead.to_string());
                    }
                    if behind > 0 {
                        local += &format_value(b_v, behind.to_string());
                    }
                    self.out.push(local);
                }
            }
        }
    }

    fn add_file_status(&mut self) {
        if let Some(s) = self.backend.get_file_status() {
            if !s.is_empty() {
                let mut o = String::from("");

                if let Some(x) = s.get(NEW_KEY) {
                    match self.conf.get_new_value() {
                        Some(y) => o += &format!("{}{}{}{}", y.pre_format, y.label, x, y.post_format),
                        None => (),
                    };
                }
                if let Some(x) = s.get(CHANGED_KEY) {
                    match self.conf.get_changed_value() {
                        Some(y) => o += &format!("{}{}{}{}", y.pre_format, y.label, x, y.post_format),
                        None => (),
                    };
                }
                if let Some(x) = s.get(STAGED_KEY) {
                    match self.conf.get_staged_value() {
                        Some(y) => o += &format!("{}{}{}{}", y.pre_format, y.label, x, y.post_format),
                        None => (),
                    };
                }
                if let Some(x) = s.get(CONFLICTS_KEY) {
                    match self.conf.get_conflicts_value() {
                        Some(y) => o += &format!("{}{}{}{}", y.pre_format, y.label, x, y.post_format),
                        None => (),
                    };
                }
                self.out.push(o);
            }
        }
    }

    fn populate(&mut self) {
        self.add_repository_state();
        self.add_monitored_branches_state();
        self.add_file_status();
    }

    // print output buffer
    fn output(&self) {
        log!(self, "# of blocks = {}", self.out.len());
        println!("{}", self.out.join("|"));
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

    let mut conf_path: Option<String> = None;
    if matches.is_present("config") {
        conf_path = Some(String::from(matches.value_of("config").unwrap()));
    }

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

    let backend = Backend{ repo: repo, debug: debug_enabled };
    let mut output = Program::new(backend, conf, debug_enabled);

    output.populate();
    output.output();
}
