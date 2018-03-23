/* This module is suppose to be a glue between all other modules.
 *
 */

extern crate clap;
extern crate git2;
// for tests
extern crate tempdir;
extern crate yaml_rust;

use std::io::{self, Write};
use backend::Backend;
use conf::{Conf,get_configuration,create_default_config};
use constants::*;
use models::{DisplayMaster};

use git2::Repository;
use clap::{App,Arg,SubCommand};

// util mod def needs to be first b/c of macro definitions and usage in other modules
#[macro_use]
mod util;
mod backend;
mod conf;
mod constants;
mod models;


fn get_version_str() -> String {
    let version: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
    let commit: Option<&'static str> = option_env!("TRAVIS_COMMIT");
    let is_dirty: Option<&'static str> = option_env!("GIT_REPO_IS_DIRTY");
    format!(
        "{} ({}{})",
        match version {
            Some(v) => v,
            None => "<version undefined>",
        },
        match commit {
            Some(v) => v,
            None => "<commit unknown>"
        },
        match is_dirty {
            Some(_) => ", dirty",
            None => ""
        }
    )
}

fn run_app() -> Result<(), String> {
    let version = get_version_str();
    let version_ref: &str = version.as_str();
    let def_conf_desc: &str = &format!("Create default config at \"{}\".", get_default_config_path().to_str().unwrap());
    let app = App::new("pretty-git-prompt")
        .version(version_ref)
        .author("Tomas Tomecek <tomas@tomecek.net>")
        .about("Get `git status` inside your shell prompt.")
        .subcommand(SubCommand::with_name(CLI_DEFAULT_CONFIG_SUBC_NAME)
                    .about(def_conf_desc))
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Use the given config file.")
             .takes_value(true))
        .arg(Arg::with_name("debug")
             .short("d")
             .long("debug")
             .help("Print debug messages, useful for identifying issues."));
    let matches = app.get_matches();

    let debug_enabled = matches.is_present("debug");
    if debug_enabled { println!("Debug messages are enabled."); }

    let conf_path: Option<String> = if matches.is_present("config") {
        Some(String::from(matches.value_of("config").unwrap()))
    } else {
        None
    };

    // create default config command
    if matches.is_present(CLI_DEFAULT_CONFIG_SUBC_NAME) {
        let p = get_default_config_path();
        match create_default_config(&p) {
            Ok(path) => {
                println!("Configuration file created at \"{}\"", path);
                return Ok(());
            }
            Err(e) => {
                return Err(format!("Failed to create configuration file \"{}\": {}", p.to_str().unwrap(), e));
            }
        };
    } else {
        // no command, run primary functionality
        let repo = match Repository::discover(".") {
            Ok(repo) => repo,
            // not a git repository, ignore
            Err(e) => {
                if debug_enabled { println!("This is not a git repository: {:?}", e); }
                return Ok(());
            }
        };

        let backend = Backend::new(repo, debug_enabled);
        let dm: DisplayMaster = DisplayMaster::new(backend, debug_enabled);
        let mut conf: Conf = get_configuration(conf_path, dm);
        let out: String = conf.populate_values();
        println!("{}", out);
    }
    Ok(())
}

fn main() {
    ::std::process::exit(match run_app() {
        Ok(_) => 0,
        Err(err) => {
            writeln!(io::stderr(), "{}", err).unwrap();
            2
        }
    });
}
