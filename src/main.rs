/* This module is suppose to be a glue between all other modules.
 *
 */

extern crate clap;
extern crate git2;
extern crate yaml_rust;

use backend::Backend;
use cli::cli;
use conf::{Conf,get_configuration,create_default_config};
use constants::*;
use models::{DisplayMaster};

use git2::Repository;

// util mod def needs to be first b/c of macro definitions and usage in other modules
#[macro_use]
mod util;
mod backend;
mod cli;
mod conf;
mod constants;
mod models;


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
    let out: String = conf.populate_values();
    println!("{}", out);
}
