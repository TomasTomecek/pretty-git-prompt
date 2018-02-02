/* This module handles configuration. Ideally it should depend only on constants.
 *
 */

use std::fs::{File,OpenOptions};
use std::io;
use std::io::{Write,Read};
use std::path::{Path,PathBuf};

use constants::{get_default_config_path, CURRENT_CONFIG_VERSION};
use models::{DisplayMaster,SimpleValue,format_value};

use yaml_rust::{YamlLoader, Yaml};


static DEFAULT_CONF: &'static str = "---
# default configuration for pretty-git-prompt
# configuration parameters are descrbed on first occurence only
#
# version of configuration file (required), type string
# right now it needs to be set to '1'
version: '1'
# configuration of various values (required), type dict
# if you omit a value, it won't be displayed
values:
      # usually repository is in state 'clean' (which is not displayed)
      # but it can also be in state like merge, rebase, cherry-pick -- this is displayed then
    - type: repository_state
      # formatting (required), both (pre_format, post_format) are required
      # you can include coloring in pre_format and reset colors in post_format
      # you can also include arbitrary string
      # for more information about setting colors for zsh:
      # https://wiki.archlinux.org/index.php/zsh#Colors
      # http://zsh.sourceforge.net/Doc/Release/Prompt-Expansion.html#Visual-effects
      # and bash:
      # https://www.ibm.com/developerworks/linux/library/l-tip-prompt/
      #
      # this is how the value is formatted in the end:
      #   [pre_format][value][post_format]
      pre_format: ''
      post_format: ''
      # this is used to separate values between each other
    - type: separator
      # condition when to display the separator (required), type string
      # possible values:
      #  * always -- display no matter what
      #  * surrounded -- show only when there is a value displayed before of after separator
      display: surrounded
      pre_format: '│'
      post_format: ''
      # monitor status against different remotes - track history divergence
    - type: remote_difference
      # remote branch name (optional), type string
      # example: 'upstream/master'
      # if omitted look for remotely tracked branch usually set up with:
      #   git branch --set-upstream-to
      # remote_branch: ''
      # display the remote even if there is no difference with current branch (required), type bool
      display_if_uptodate: true
      pre_format: ''
      post_format: ''
      # values which can be displayed as part of 'remote_difference'
      values:
          # formatting for remote name and branch name
        - type: name
          # there are some special values which are substituted:
          #  * <REMOTE> will be replaced with name of a remote
          #  * <LOCAL_BRANCH> will be replaced with current branch name
          #  * <REMOTE_BRANCH> will be replaced with name of remote branch
          pre_format: '<LOCAL_BRANCH>'
          post_format: ''
          # the number of files present locally which are missing in remote repo
        - type: ahead
          pre_format: '↑'
          post_format: ''
          # the number of commits present in remote repo which are missing locally
        - type: behind
          pre_format: '↓'
          post_format: ''
    - type: separator
      display: surrounded
      pre_format: '│'
      post_format: ''
    - type: remote_difference
      remote_branch: 'upstream/master'
      display_if_uptodate: false
      pre_format: ''
      post_format: ''
      values:
        - type: name
          pre_format: '<REMOTE>'
          post_format: ''
        - type: ahead
          pre_format: '↑'
          post_format: ''
        - type: behind
          pre_format: '↓'
          post_format: ''
    - type: separator
      display: surrounded
      pre_format: '│'
      post_format: ''
      # the number of untracked files
    - type: new
      pre_format: '✚'
      post_format: ''
      # the number of tracked files which were changed in working tree
    - type: changed
      pre_format: 'Δ'
      post_format: ''
      # the number of files added to index
    - type: staged
      pre_format: '▶'
      post_format: ''
      # during merge, rebase, or others, the numbers files which conflict
    - type: conflicts
      pre_format: '✖'
      post_format: ''
    - type: stash
      pre_format: '☐'
      post_format: ''
";


#[derive(Debug, Clone)]
struct Separator {
    // debug: bool,  TODO
    value: SimpleValue,
    display: String,
}

// FIXME: this should be defined in models
impl Separator {
    fn new(value_yaml: &Yaml, simple_value: &SimpleValue) -> Separator {
        let separator_display_mode = match value_yaml["display"].as_str() {
            Some(s) => {
                if !vec!("always", "surrounded").contains(&s) {
                    panic!("'display' needs to be one of 'always' or 'surrounded'");
                }
                s
            },
            None => panic!("separator needs to have specified 'display' attribute"),
        };
        Separator{
            value: simple_value.clone(), display: separator_display_mode.to_string()
        }
    }

    fn is_display_always(&self) -> bool {
        self.display == "always"
    }

    // FIXME: https://github.com/TomasTomecek/pretty-git-prompt/issues/33
    // fn is_display_surrounded(&self) -> bool {
    //     self.display == "surrounded"
    // }

    fn display(&self) -> Option<String> {
        // log!(self, "display separator, value: {:?}", self);
        Some(format_value(&self.value.pre_format, &self.value.post_format, ""))
    }
}



pub struct Conf {
    c: Yaml,
    display_master: DisplayMaster,
}

impl Conf {
    pub fn new(yaml: Yaml, display_master: DisplayMaster) -> Conf {
        let y_ref = &yaml;
        let version = &y_ref["version"];
        if version.is_badvalue() || version.is_null() {
            panic!("'version' is missing in config file.");
        }
        // there could be a better place to validate this
        match version.as_str() {
            Some(s) => {
                if s != CURRENT_CONFIG_VERSION {
                    panic!("Config should be using version '{}', instead is using '{}'",
                           CURRENT_CONFIG_VERSION, s);
                }
            },
            None => panic!("'version' should be string: {:?}", version),
        }
        Conf { c: yaml.clone(), display_master: display_master }
    }

    // FIXME: this is super-hacky and because of separators, since they need to know
    //        if there is a value surrounding them; ideally this would return an array of
    //        struct, which would hold common attributes and a reference to yaml, each value
    //        would be then validated
    pub fn populate_values(&mut self) -> String {
        let values_yaml = &self.c["values"];
        if values_yaml.is_badvalue() || values_yaml.is_null() {
            panic!("No values to display.");
        }
        let values = values_yaml.as_vec().unwrap();
        let mut out: String = String::new();
        // was there a previous value already displayed?
        let mut prev_was_set = false;
        // are we suppose to display a separator?
        let mut separator_pending: Option<String> = None;

        // FIXME: all of this logic should live outside of this module
        for v in values {
            let simple_value = SimpleValue::new(v);
            let value_type = simple_value.value_type.as_str();
            if value_type == "separator" {
                let separator = Separator::new(v, &simple_value);
                let separator_display = separator.display();
                if separator.is_display_always() {
                    out += &separator_display.unwrap();
                } else {
                    separator_pending = separator_display;
                }
            } else if let Some(s) = self.display_master.display_value(v, &simple_value) {
                // add separator if it is needed
                if let Some(separator) = separator_pending.clone() {
                    if prev_was_set {
                        // println!("add separator {:?}", simple_value);
                        out += &separator;
                        separator_pending = None;
                    }
                }
                out += &s;
                prev_was_set = true;
            }
        }
        out.clone()
    }
}

pub fn load_configuration_from_file<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(e)
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => Ok(contents),
        Err(e) => Err(e)
    }
}

// main function to obtain Conf struct
pub fn get_configuration(supplied_conf_path: Option<String>, display_master: DisplayMaster) -> Conf {
    let content: String = if supplied_conf_path.is_some() {
        match load_configuration_from_file(supplied_conf_path.unwrap()) {
            Ok(c) => c,
            Err(e) => {
                println!("ERROR");
                panic!("Couldn't open configuration file: {:?}", e);
            }
        }
    } else {
        match load_configuration_from_file(get_default_config_path()) {
            Ok(c) => c,
            Err(e) => {
                let kind = e.kind();
                if kind == io::ErrorKind::NotFound {
                    String::from(DEFAULT_CONF)
                } else {
                    println!("ERROR");
                    panic!("Couldn't open configuration file: {:?}", kind);
                }
            }
        }
    };
    let docs = YamlLoader::load_from_str(&content).unwrap();
    Conf::new(docs[0].clone(), display_master)
}

// take default config and write it to path of default config location
// error out if the config already exists
pub fn create_default_config(path: &PathBuf) -> Result<String, io::Error> {
    match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.clone()) {
        Ok(mut file) => {
            match file.write_all(&String::from(DEFAULT_CONF).into_bytes()) {
                Ok(_) => (),
                Err(e) => return Err(e)
            };
            match file.flush() {
                Ok(_) => (),
                Err(e) => return Err(e)
            };
            Ok(String::from(path.to_str().unwrap()))
        },
        Err(e) => Err(e)
    }
}


mod tests {
    // We'll use this git repo for testing
    use std::env;
    use std::fs::{File,OpenOptions,remove_file};
    use std::io::{Read};
    use std::path::{Path,PathBuf};
    use std::process::{Command,Stdio};
    use conf::{get_configuration,create_default_config,DEFAULT_CONF,Conf};
    use yaml_rust::{YamlLoader};
    use backend::Backend;
    use models::DisplayMaster;
    use git2::{Repository};
    use tempdir::TempDir;

    macro_rules! init_git {
        () => {
            let dir = TempDir::new("p-g-p").unwrap();
            env::set_current_dir(&dir).unwrap();
            // we could use git2 to create a repo with a commit, but it's soooo complicated
            let mut c = Command::new("git").args(
                &["-c", "user.name=Git \"Pretty\" Prompter", "-c", "user.email=pretty-git-prompt@example.com",
                  "init", "."])
                .stdout(Stdio::null())
                .spawn()
                .unwrap();
            let rc = c.wait().unwrap();

            let mut c = Command::new("git").args(
                &["-c", "user.name=Git \"Pretty\" Prompter", "-c", "user.email=pretty-git-prompt@example.com",
                  "commit", "--allow-empty", "-m", "init"])
                .stdout(Stdio::null())
                .spawn()
                .unwrap();
            let rc = c.wait().unwrap();
        }
    }

    #[test]
    #[should_panic(expected = "'version' is missing in config file.")]
    fn test_empty_config() {
        let config_text = "{}";
        let docs = YamlLoader::load_from_str(config_text).unwrap();

        init_git!();

        let repo = Repository::discover(".").unwrap();
        let backend = Backend::new(repo, true);
        let dm: DisplayMaster = DisplayMaster::new(backend, true);
        Conf::new(docs[0].clone(), dm);
    }

    #[test]
    fn test_values_is_present() {
        let config_text = "version: '1'
values: []";
        let docs = YamlLoader::load_from_str(config_text).unwrap();

        init_git!();

        let repo = Repository::discover(".").unwrap();
        let backend = Backend::new(repo, true);
        let dm: DisplayMaster = DisplayMaster::new(backend, true);
        Conf::new(docs[0].clone(), dm);
    }

    #[test]
    fn test_create_default_config() {
        let p = PathBuf::from("/tmp/test_pretty_git_prompt_config1");
        if Path::new(&p).exists() {
            remove_file(p.clone()).unwrap();
        }

        let result = create_default_config(&p);
        assert!(result.is_ok());

        let mut file = File::open(p.clone()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, DEFAULT_CONF);

        remove_file(p.clone()).unwrap();
    }
    #[test]
    fn test_create_default_config_when_exists() {
        let p = PathBuf::from("/tmp/test_pretty_git_prompt_config2");
        OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(p.clone()).unwrap();
        assert!(Path::new(&p).exists());

        let result = create_default_config(&p);
        assert!(result.is_err());

        remove_file(p.clone()).unwrap();
    }
    #[test]
    fn test_load_default_config() {
        let p = PathBuf::from("/tmp/test_pretty_git_prompt_config3");
        if Path::new(&p).exists() {
            remove_file(p.clone()).unwrap();
        }

        let result = create_default_config(&p);
        assert!(result.is_ok());

        init_git!();

        let repo = Repository::discover(".").unwrap();
        let backend = Backend::new(repo, true);
        let dm: DisplayMaster = DisplayMaster::new(backend, true);
        get_configuration(None, dm);

        remove_file(p.clone()).unwrap();
    }

    #[test]
    #[should_panic(expected = "Config should be using version '1', instead is using '0'")]
    fn test_lower_version() {
        let config_text = "version: '0'";
        let docs = YamlLoader::load_from_str(config_text).unwrap();

        init_git!();

        let repo = Repository::discover(".").unwrap();
        let backend = Backend::new(repo, true);
        let dm: DisplayMaster = DisplayMaster::new(backend, true);
        Conf::new(docs[0].clone(), dm);
    }
}
