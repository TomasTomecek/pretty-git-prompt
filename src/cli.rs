use constants::{CLI_DEFAULT_CONFIG_SUBC_NAME};

use clap::{App,Arg,SubCommand};


pub fn cli<'a, 'b>() -> App<'a, 'b> {
    // FIXME: populate about with this
    // let ref def_conf_desc = format!("Create default config at \"{}\".", get_default_config_path().to_str().unwrap());
    App::new("pretty-zsh-prompt")
        .version("0.1.2")
        .author("Tomas Tomecek <tomas@tomecek.net>")
        .about("Get `git status` inside your shell prompt.")
        .subcommand(SubCommand::with_name(CLI_DEFAULT_CONFIG_SUBC_NAME))
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Use the given config file.")
             .takes_value(true))
        .arg(Arg::with_name("debug")
             .short("d")
             .long("debug")
             .help("Print debug messages, useful for identifying issues."))
}
