/* This module is suppose to be a glue between all other modules.
 *
 */

extern crate clap;
extern crate git2;
extern crate yaml_rust;

use std::io::{self, Write};
use backend::Backend;
use conf::{Conf,get_configuration,create_default_config};
use constants::*;
use models::{DisplayMaster};

use git2::Repository;
use clap::{Arg, ArgAction, Command};

// util mod def needs to be first b/c of macro definitions and usage in other modules
#[macro_use]
mod util;
mod backend;
mod conf;
mod constants;
mod models;

fn main() {
    let def_conf_desc: String = format!("Create default config at \"{}\".", get_default_config_path().to_str().unwrap());
    let matches = Command::new("pretty-git-prompt")
        .version(option_env!("CARGO_PKG_VERSION"))
        .author("Tomas Tomecek <tomas@tomecek.net>")
        .about("Get `git status` inside your shell prompt.")
        .subcommand(Command::new("create-default-config")
            .about(def_conf_desc))
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .value_name("FILE")
            .help("Use the given config file."))
        .arg(Arg::new("debug")
            .short('d')
            .long("debug")
            .help("Print debug messages, useful for identifying issues.")
            .action(ArgAction::SetTrue)
        ).get_matches();

    let debug_enabled = matches.get_flag("debug");
    if debug_enabled { println!("Debug messages are enabled."); }

    match matches.subcommand() {
        Some(("create-default-config", _sub_matches)) => {
            let p = get_default_config_path();
            match create_default_config(&p) {
                Ok(path) => {
                    println!("Configuration file created at \"{}\"", path);
                    ::std::process::exit(0);
                }
                Err(e) => {
                    if let Err(e2) = writeln!(
                        io::stderr(),
                        "Failed to create configuration file \"{}\": {}",
                        p.to_str().unwrap(),
                        e
                    ) {
                        println!("Writing error: {}", e2.to_string());
                    }
                    ::std::process::exit(2);
                }
            };
        },
        _ => {
            // no command, run primary functionality
            let repo = match Repository::discover(".") {
                Ok(repo) => repo,
                // not a git repository, ignore
                Err(e) => {
                    if debug_enabled { println!("This is not a git repository: {:?}", e); }
                    ::std::process::exit(0);
                }
            };

            let backend = Backend::new(repo, debug_enabled);
            let dm: DisplayMaster = DisplayMaster::new(backend, debug_enabled);
            let conf_path = matches.get_one::<String>("config");
            let mut conf: Conf = get_configuration(conf_path.cloned(), dm);
            let out: String = conf.populate_values();
            println!("{}", out);
        }
    }
}
