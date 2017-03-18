use std::env;
use std::fs::create_dir_all;
use std::path::{PathBuf};

// used to index values in map
pub static CHANGED_KEY: &'static str = "changed";
pub static NEW_KEY: &'static str = "new";
pub static STAGED_KEY: &'static str = "staged";
pub static CONFLICTS_KEY: &'static str = "conflicts";

pub static DEFAULT_CONFIG_NAME: &'static str = "pretty-git-prompt.yml";

pub static CLI_DEFAULT_CONFIG_SUBC_NAME: &'static str = "create-default-config";

pub fn get_default_config_path() -> PathBuf {
    let mut p = match env::var("XDG_CONFIG_HOME") {
        Ok(val) => PathBuf::from(&val),
        Err(_) => {
            match env::var("HOME") {
                Ok(home) => {
                    let mut p2 = PathBuf::from(&home);
                    p2.push(".config");
                    p2
                },
                // we tried hard
                Err(_) => PathBuf::from("/tmp"),
            }
        }
    };
    create_dir_all(p.to_str().unwrap());
    p.push(DEFAULT_CONFIG_NAME);
    p
}

#[test]
fn test_default_config_path() {
    let p = get_default_config_path();
    // TODO: test if the dir exists
    // TODO: test if the str ends with DEFAULT_CONFIG_NAME
}
