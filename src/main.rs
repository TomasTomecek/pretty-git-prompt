/* This module is suppose to be a glue between all other modules.
 *
 */

extern crate clap;
extern crate git2;
#[cfg(test)]
extern crate tempfile;
extern crate yaml_rust;

use std::io::{self, Write};
use backend::Backend;
use colors::{Shell,colors_wanted,list_colors};
use conf::{Conf,get_configuration,get_configuration_content,create_default_config};
use constants::*;
use models::{DisplayMaster};
use preview::{preview_demo,preview_repo};

use git2::Repository;
use clap::{Arg, ArgAction, ArgMatches, Command};
use yaml_rust::{Yaml,YamlLoader};

// util mod def needs to be first b/c of macro definitions and usage in other modules
#[macro_use]
mod util;
mod backend;
mod colors;
mod conf;
mod constants;
mod models;
mod preview;

// the shell to format for: what was asked for, what the config was written for,
// what the user runs -- in this order
fn resolve_shell(requested: Option<&String>, config_content: Option<&str>) -> Shell {
    if let Some(name) = requested {
        match Shell::from_name(name) {
            Some(s) => return s,
            None => {
                writeln!(io::stderr(), "Unknown shell \"{}\", expected 'bash' or 'zsh'.", name)
                    .ok();
                ::std::process::exit(2);
            }
        }
    }
    if let Some(content) = config_content {
        if let Some(s) = Shell::from_config(content) {
            return s;
        }
    }
    Shell::detect().unwrap_or(Shell::Bash)
}

fn shell_arg() -> Arg {
    Arg::new("shell")
        .short('s')
        .long("shell")
        .value_name("SHELL")
        .help("Shell to print the codes for: 'bash' or 'zsh'.")
}

fn no_color_arg() -> Arg {
    Arg::new("no-color")
        .long("no-color")
        .help("Don't color the output.")
        .action(ArgAction::SetTrue)
}

// piping the output into a pager or head is not an error worth reporting
fn exit_on_write_error(result: io::Result<()>) {
    if let Err(e) = result {
        if e.kind() == io::ErrorKind::BrokenPipe {
            ::std::process::exit(0);
        }
        writeln!(io::stderr(), "Can't write the output: {}", e).ok();
        ::std::process::exit(3);
    }
}

fn config_path(matches: &ArgMatches, sub_matches: &ArgMatches) -> Option<String> {
    sub_matches.get_one::<String>("config")
        .or_else(|| matches.get_one::<String>("config"))
        .cloned()
}

fn run_list_colors(sub_matches: &ArgMatches) {
    let colors = colors_wanted(sub_matches.get_flag("no-color"));
    let shells: Vec<Shell> = match sub_matches.get_one::<String>("shell") {
        Some(_) => vec!(resolve_shell(sub_matches.get_one::<String>("shell"), None)),
        // when we don't know which shell the user runs, print both
        None => match Shell::detect() {
            Some(s) => vec!(s),
            None => vec!(Shell::Zsh, Shell::Bash),
        },
    };
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    exit_on_write_error(list_colors(&mut handle, &shells, colors));
}

fn run_preview(sub_matches: &ArgMatches, conf_path: Option<String>, debug: bool) {
    let colors = colors_wanted(sub_matches.get_flag("no-color"));
    let content = get_configuration_content(conf_path);
    let shell = resolve_shell(sub_matches.get_one::<String>("shell"), Some(&content));
    let yaml: Yaml = YamlLoader::load_from_str(&content).unwrap()[0].clone();

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let result = if sub_matches.get_flag("demo") {
        preview_demo(&mut handle, &yaml, shell, colors, debug)
    } else {
        let repo = match Repository::discover(".") {
            Ok(repo) => repo,
            Err(e) => {
                if debug { println!("This is not a git repository: {:?}", e); }
                writeln!(io::stderr(), "This is not a git repository, \
                                        try 'pretty-git-prompt preview --demo'.").ok();
                ::std::process::exit(1);
            }
        };
        preview_repo(&mut handle, &yaml, Backend::new(repo, debug), shell, colors, debug)
    };
    exit_on_write_error(result);
}

fn main() {
    let def_conf_desc: String = format!("Create default config at \"{}\".", get_default_config_path().to_str().unwrap());
    let matches = Command::new("pretty-git-prompt")
        .version(option_env!("CARGO_PKG_VERSION"))
        .author("Tomas Tomecek <tomas@tomecek.net>")
        .about("Get `git status` inside your shell prompt.")
        .subcommand(Command::new("create-default-config")
            .about(def_conf_desc))
        .subcommand(Command::new("list-colors")
            .about("List colors and text styles with the codes to put in a config file.")
            .arg(shell_arg())
            .arg(no_color_arg()))
        .subcommand(Command::new("preview")
            .about("Render the prompt in this terminal, the way your shell would.")
            .arg(Arg::new("demo")
                .long("demo")
                .help("Use made up repository states instead of the current repository.")
                .action(ArgAction::SetTrue))
            .arg(Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Use the given config file."))
            .arg(shell_arg())
            .arg(no_color_arg()))
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
        Some(("list-colors", sub_matches)) => run_list_colors(sub_matches),
        Some(("preview", sub_matches)) => {
            run_preview(sub_matches, config_path(&matches, sub_matches), debug_enabled)
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
