use constants::{CLI_DEFAULT_CONFIG_SUBC_NAME};

use clap::{App,Arg,SubCommand};


pub fn cli<'a, 'b>() -> App<'a, 'b> {
    // FIXME: populate about with this
    // let ref def_conf_desc = format!("Create default config at \"{}\".", get_default_config_path().to_str().unwrap());
    App::new("pretty-zsh-prompt")
        .version("0.0.1")
        .author("Tomas Tomecek <tomas@tomecek.net>")
        .about("Get `git status` inside your shell prompt.")
        .subcommand(SubCommand::with_name(CLI_DEFAULT_CONFIG_SUBC_NAME))
    //                             .about(def_conf_desc))
        .arg(Arg::with_name("color_mode")
             .long("color-mode")
             .short("m")
             .default_value("no")
             .possible_values(&["zsh", "no"]))
}
