// TODO:
//  * add option to debug: print all errors

extern crate clap;
extern crate git2;
extern crate yaml_rust;

use backend::Backend;
use cli::cli;
use conf::{Conf,get_configuration,create_default_config,Value,MonitoredRemote};
use constants::*;

use git2::Repository;

mod backend;
mod cli;
mod conf;
mod constants;


fn format_value(value: Value, data: String) -> String {
    format!("{}{}{}{}", value.pre_format, value.label, data, value.post_format)
}


// struct for displaying all the data -- the actual program
struct Program {
    out: Vec<String>,
    backend: Backend,
    conf: Conf,
}

impl Program {
    pub fn new(backend: Backend, conf: Conf) -> Program {
        let out: Vec<String> = Vec::new();
        Program { out: out, backend: backend, conf: conf }
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
                    let mut local: String = format!("{}{}/{}{}",
                                                    monitored_remote.pre_format, monitored_remote.remote_name,
                                                    b, monitored_remote.post_format);
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
        // println!("{}", out.len());
        println!("{}", self.out.join("|"));
    }
}

fn main() {
    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(_) => return (),
    };
    let backend = Backend{ repo: repo };
    let conf = get_configuration(false);
    let app = cli();
    let matches = app.get_matches();

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

    let mut output = Program::new(backend, conf);

    output.populate();
    output.output();
}
