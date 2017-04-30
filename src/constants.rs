use std::env;
use std::fs::create_dir_all;
use std::path::{PathBuf};

// used to index values in map
pub static CHANGED_KEY: &'static str = "changed";
pub static NEW_KEY: &'static str = "new";
pub static STAGED_KEY: &'static str = "staged";
pub static CONFLICTS_KEY: &'static str = "conflicts";

pub static CURRENT_CONFIG_VERSION: &'static str = "1";

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
    let p_clone = p.clone();  // fighting borrow checker hard
    let d = p_clone.to_str().unwrap();
    match create_dir_all(d) {
        Ok(_) => (),
        Err(e) => panic!("Unable to create directory for confi: {}: {:?}", d, e),
    };
    p.push(DEFAULT_CONFIG_NAME);
    p
}

mod tests {
    use std::ffi::OsStr;

    #[test]
    fn test_default_config_path() {
        // FIXME: create test dir and remove it aftet testing
        let p = get_default_config_path();
        assert!(p.parent().unwrap().exists());
        assert_eq!(p.file_name().unwrap(), OsStr::new(DEFAULT_CONFIG_NAME));
    }
}
