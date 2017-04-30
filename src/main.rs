extern crate clap;
extern crate git2;
extern crate yaml_rust;

use backend::Backend;
use cli::cli;
use conf::{RemoteBranch};
use conf::{Conf,get_configuration,create_default_config};
use constants::*;
use models::{DisplayMaster,Display};

use std::collections::HashMap;

use git2::Repository;

// util mod def needs to be first b/c of macro definitions and usage in other modules
#[macro_use]
mod util;
mod backend;
mod cli;
mod conf;
mod constants;
mod models;


fn substiute_special_values(s: String, values: &HashMap<String, String>) -> String {
    let mut r:String = s;
    for (k, v) in values {
        r = r.replace(k, &v);
    }
    r
}
// logic of the whole program -- the glue
struct Program {
    conf: Conf,
    out: Vec<String>,
    debug: bool
}

impl Program {
    pub fn new(conf: Conf, out: Vec<String>, debug: bool) -> Program {
        Program { conf: conf, out: out, debug: debug }
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

    // fn display_values(&mut self) {
    //     let mut out: Vec<String> = Vec::new();
    //     let values = match self.conf.get_values() {
    //         Some(v) => v,
    //         None => panic!("No values present in configuration, nothing to display."),
    //     };
    //     for value in values {
    //         match self.display_master.display(&value) {
    //             Some(v) => out.push(v),
    //             None => ()
    //         };
    //     }
    //     self.output(out);
    // }

    // print output buffer
    fn output(&self) {
        log!(self, "# of blocks = {}", self.out.len());
        let output = self.out.join("|");
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
    let dm: DisplayMaster = DisplayMaster::new(backend, debug_enabled);
    let mut conf: Conf = get_configuration(conf_path, dm);
    let out: Vec<String> = conf.populate_values();
    let mut output = Program::new(conf, out, debug_enabled);
    output.output();
}
