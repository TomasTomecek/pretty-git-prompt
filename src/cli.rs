extern crate clap;

use self::clap::{App,Arg};


pub fn cli<'a, 'b>() -> App<'a, 'b> {
    App::new("pretty-zsh-prompt")
        .version("0.0.1")
        .author("Tomas Tomecek <tomas@tomecek.net>")
        .about("Get `git status` inside your shell prompt.")
        .arg(Arg::with_name("color_mode")
             .long("color-mode")
             .short("m")
             .default_value("no")
             .possible_values(&["zsh", "no"]))
}
