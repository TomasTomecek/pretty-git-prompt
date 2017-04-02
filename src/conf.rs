use std::collections::btree_map::BTreeMap;
use std::fs::{File,OpenOptions};
use std::io;
use std::io::{Write,Read};
use std::path::{Path,PathBuf};

use constants::{get_default_config_path};

use yaml_rust::{YamlLoader, Yaml};

// TODO: remove label key, merge with pre, post
// TODO: add version
static DEFAULT_CONF: &'static str = "---
# configuration of various values (required), type dict
# if you omit a value, it won't be displayed
values:
    # count of untracked files
    new:
        # prefix label (required), type string
        label: '✚'
        # formatting specification of the label and value
        # https://wiki.archlinux.org/index.php/zsh#Colors
        # http://zsh.sourceforge.net/Doc/Release/Prompt-Expansion.html#Visual-effects
        # TODO: bash
        # TODO: fish?
        pre_format: ''
        post_format: ''
    changed:
        label: 'Δ'
        pre_format: ''
        post_format: ''
    staged:
        label: '▶'
        pre_format: ''
        post_format: ''
    conflicts:
        label: '✖'
        pre_format: ''
        post_format: ''
    difference_ahead:
        label: '↑'
        pre_format: ''
        post_format: ''
    difference_behind:
        label: '↓'
        pre_format: ''
        post_format: ''

# monitor status against different remotes (optional), type dict
# track history divergence
monitor_remotes:
      # formatting (required), both are required
      # this is displayed as: '[pre_format][value][post_format]'
      # include coloring in pre_format and reset colors in post_format
      # you can also include arbitrary string
      # there are some special values which are substituted:
      #  * <REMOTE> will be replaced with name of a remote
      #  * <LOCAL_BRANCH> will be replaced with current branch name
      #  * <REMOTE_BRANCH> will be replaced with name of remote branch
    - pre_format: '<LOCAL_BRANCH>'
      post_format: ''
      # remote branch name (optional), type string
      # example: 'upstream/mater'
      # if omitted look for remotely tracked branch usualy set up with:
      #   git branch --set-upstream-to
      # remote_branch: ''
      # display the remote even if there is no difference with current branch (required), type bool
      display_if_uptodate: true
    - remote_branch: 'upstream/master'
      display_if_uptodate: false
      pre_format: '<REMOTE>'
      post_format: ''
";

#[derive(Debug, Clone)]
pub struct RemoteBranch {
    // upstream/master
    // this is the name git is using
    pub remote_branch: String,
    // master
    pub remote_branch_name: String,
    // upstream
    pub remote_name: String,
}

#[derive(Debug, Clone)]
pub struct MonitoredRemote {
    pub remote_branch: Option<RemoteBranch>,
    pub display_if_uptodate: bool,
    pub pre_format: String,
    pub post_format: String,
}

impl MonitoredRemote {
    fn new(remote_config: &Yaml) -> MonitoredRemote {
        let uptodate: bool;
        let display_if_uptodate = &remote_config["display_if_uptodate"];
        if display_if_uptodate.is_badvalue() || display_if_uptodate.is_null() {
            panic!("'display_if_uptodate' key is required and has to be bool");
        }
        uptodate = display_if_uptodate.as_bool().unwrap();

        let pre_format: String;
        let pre_format_yaml = &remote_config["pre_format"];
        if pre_format_yaml.is_badvalue() || pre_format_yaml.is_null() {
            panic!("'pre_format' key is required and has to be string");
        }
        pre_format = String::from(pre_format_yaml.as_str().unwrap());

        let post_format: String;
        let post_format_yaml = &remote_config["post_format"];
        if post_format_yaml.is_badvalue() || post_format_yaml.is_null() {
            panic!("'post_format' key is required and has to be string");
        }
        post_format = String::from(post_format_yaml.as_str().unwrap());

        let mut rb: Option<RemoteBranch> = None;
        let remote_branch = &remote_config["remote_branch"];
        if !(remote_branch.is_badvalue() || remote_branch.is_null()) {
            let remote_branch_string = remote_branch.as_str().unwrap();
            let v: Vec<&str> = remote_branch_string.splitn(2, "/").collect();
            if v.len() != 2 {
                panic!("`remote_branch` must be in form of `<REMOTE>/<BRANCH>`");
            }
            rb = Some(RemoteBranch{ remote_branch: remote_branch_string.to_string(),
                                    remote_name: v[0].to_string(),
                                    remote_branch_name: v[1].to_string() });
        }

        let mut mr = MonitoredRemote{
            remote_branch: rb,
            display_if_uptodate: uptodate,
            pre_format: pre_format,
            post_format: post_format
        };

        mr
    }
}

pub struct Value {
    pub label: String,
    pub pre_format: String,
    pub post_format: String,
}

impl Value {
    fn new(y: &Yaml, key: &str) -> Option<Value> {
        let ref values_yaml = y["values"];
        if values_yaml.is_badvalue() || values_yaml.is_null() {
            panic!("'values' key is required and has to be map")
        }
        let ref value_yaml = values_yaml[key];
        if value_yaml.is_badvalue() || value_yaml.is_null() {
            // TODO: debug log here
            return None;
        }
        let label = match value_yaml["label"].as_str() {
            Some(x) => x,
            None => panic!("'label' has to be present and needs to be string"),
        };
        let mut v = Value{ label: String::from(label), pre_format: String::from(""), post_format: String::from("") };
        let ref pre_format_yaml = value_yaml["pre_format"];
        if !(pre_format_yaml.is_badvalue() || pre_format_yaml.is_null()) {
            v.pre_format = String::from(pre_format_yaml.as_str().unwrap());
        }
        let ref post_format_yaml = value_yaml["post_format"];
        if !(post_format_yaml.is_badvalue() || post_format_yaml.is_null()) {
            v.post_format = String::from(post_format_yaml.as_str().unwrap());
        }
        Some(v)
    }
}

pub struct Conf {
    c: Yaml,
}

impl Conf {
    pub fn new(yaml: Yaml) -> Conf {
        Conf { c: yaml }
    }

    pub fn get_new_value(&self) -> Option<Value> {
        Value::new(&self.c, "new")
    }
    pub fn get_changed_value(&self) -> Option<Value> {
        Value::new(&self.c, "changed")
    }
    pub fn get_staged_value(&self) -> Option<Value> {
        Value::new(&self.c, "staged")
    }
    pub fn get_conflicts_value(&self) -> Option<Value> {
        Value::new(&self.c, "conflicts")
    }
    pub fn get_difference_ahead_value(&self) -> Option<Value> {
        Value::new(&self.c, "difference_ahead")
    }
    pub fn get_difference_behind_value(&self) -> Option<Value> {
        Value::new(&self.c, "difference_behind")
    }

    pub fn get_remotes_monitoring(&self) -> Option<Vec<MonitoredRemote>> {
        let ref remotes_yaml = self.c["monitor_remotes"];
        if remotes_yaml.is_badvalue() || remotes_yaml.is_null() {
            return None;
        }
        let remotes = remotes_yaml.as_vec().unwrap();
        let mut response: Vec <MonitoredRemote> = Vec::new();
        for y in remotes {
            response.push(MonitoredRemote::new(y));
        }
        Some(response)
    }

    pub fn get_branch_color(&self) -> String {
        String::from(self.c["colors"]["branch"].as_str().unwrap())
    }
    pub fn get_remote_difference_color(&self) -> String {
        String::from(self.c["colors"]["remote_difference"].as_str().unwrap())
    }
    pub fn get_new_color(&self) -> String {
        String::from(self.c["colors"]["new"].as_str().unwrap())
    }
    pub fn get_changed_color(&self) -> String {
        String::from(self.c["colors"]["changed"].as_str().unwrap())
    }
    pub fn get_staged_color(&self) -> String {
        String::from(self.c["colors"]["staged"].as_str().unwrap())
    }
    pub fn get_conflicts_color(&self) -> String {
        String::from(self.c["colors"]["conflicts"].as_str().unwrap())
    }
}

pub fn load_configuration_from_file<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(e)
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(size) => Ok(contents),
        Err(e) => Err(e)
    }
}

pub fn get_configuration(supplied_conf_path: Option<String>) -> Conf {
    let mut content: String;
    if supplied_conf_path.is_some() {
        content = match load_configuration_from_file(supplied_conf_path.unwrap()) {
            Ok(c) => c,
            Err(e) => {
                println!("ERROR");
                panic!("Couldn't open configuration file: {:?}", e);
            }
        };
    } else {
        content = match load_configuration_from_file(get_default_config_path()) {
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
        };
    }
    let docs = YamlLoader::load_from_str(&content).unwrap();
    Conf::new(docs[0].clone())
}

pub fn create_default_config(path: PathBuf) -> Result<String, io::Error> {
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


// TODO: use 'expected' with should_panic
// #[should_panic(expected = "assertion failed")]
mod tests {
    use std::fs::{File,OpenOptions,remove_file};
    use std::io::{Write,Read};
    use std::path::{Path,PathBuf};

    use conf::{get_configuration,create_default_config,DEFAULT_CONF,Conf};

    use yaml_rust::{YamlLoader, Yaml};

    #[test]
    #[should_panic]
    fn test_values_is_present() {
        let config_text = "{}";
        let docs = YamlLoader::load_from_str(config_text).unwrap();
        let c = Conf::new(docs[0].clone());

        let o = c.get_new_value();
    }

    #[test]
    fn test_default_new_value() {
        let c = get_configuration(None);
        let o = c.get_new_value();
        let v = o.unwrap();
        assert_eq!(v.label, "✚");
    }
    #[test]
    fn test_nonexistent_new_value() {
        let config_text = "values: {}";
        let docs = YamlLoader::load_from_str(config_text).unwrap();
        let c = Conf::new(docs[0].clone());

        let o = c.get_new_value();
        assert!(o.is_none());
    }
    #[test]
    #[should_panic]
    fn test_empty_new_value() {
        let config_text = "values:
    new: []";
        let docs = YamlLoader::load_from_str(config_text).unwrap();
        let c = Conf::new(docs[0].clone());

        let o = c.get_new_value();
        assert!(o.is_none());
    }
    #[test]
    fn test_some_new_value() {
        let config_text = "values:
    new:
        label: '+'
        pre_format: '%{%F{014}%}'
        post_format: '%{%f%}'
";
        let docs = YamlLoader::load_from_str(config_text).unwrap();
        let c = Conf::new(docs[0].clone());

        let o = c.get_new_value();
        assert!(o.is_some());
        let v = o.unwrap();
        assert_eq!(v.label, "+");
        assert_eq!(v.pre_format, "%{%F{014}%}");
        assert_eq!(v.post_format, "%{%f%}");
    }

    #[test]
    fn test_default_changed_symbol() {
        let c = get_configuration(None);
        let o = c.get_changed_value();
        let v = o.unwrap();
        assert_eq!(v.label, "Δ");
    }
    #[test]
    fn test_default_staged_symbol() {
        let c = get_configuration(None);
        let o = c.get_staged_value();
        let v = o.unwrap();
        assert_eq!(v.label, "▶");
    }
    #[test]
    fn test_default_conflicts_symbol() {
        let c = get_configuration(None);
        let o = c.get_conflicts_value();
        let v = o.unwrap();
        assert_eq!(v.label, "✖");
    }
    #[test]
    fn test_difference_ahead_symbol() {
        let c = get_configuration(None);
        let o = c.get_difference_ahead_value();
        let v = o.unwrap();
        assert_eq!(v.label, "↑");
    }
    #[test]
    fn test_difference_behind_symbol() {
        let c = get_configuration(None);
        let o = c.get_difference_behind_value();
        let v = o.unwrap();
        assert_eq!(v.label, "↓");
    }

    #[test]
    fn test_default_monitored_remotes() {
        let c = get_configuration(None);
        let remotes = c.get_remotes_monitoring().unwrap();
        let ref origin = remotes[0];
        assert_eq!(origin.display_if_uptodate, true);
        let ref upstream = remotes[1];
        assert_eq!(upstream.display_if_uptodate, false);
        let b_struct_opt = upstream.remote_branch.clone();
        let b_struct = b_struct_opt.unwrap();
        assert_eq!(b_struct.remote_branch, "upstream/master");
    }
    #[test]
    fn test_monitored_remotes_ordering() {
        let config_text = "monitor_remotes:
    - remote_branch: x/x
      display_if_uptodate: true
      pre_format: ''
      post_format: ''
    - remote_branch: y/y
      display_if_uptodate: true
      pre_format: ''
      post_format: ''
    - remote_branch: z/z
      display_if_uptodate: true
      pre_format: ''
      post_format: ''
";
        let docs = YamlLoader::load_from_str(config_text).unwrap();
        let c = Conf::new(docs[0].clone());

        let remotes = c.get_remotes_monitoring().unwrap();
        let mut iter = remotes.iter();
        assert_eq!(iter.next().unwrap().clone().remote_branch.unwrap().remote_branch, "x/x");
        assert_eq!(iter.next().unwrap().clone().remote_branch.unwrap().remote_branch, "y/y");
        assert_eq!(iter.next().unwrap().clone().remote_branch.unwrap().remote_branch, "z/z");
    }

    #[allow(unused_must_use)]
    #[test]
    fn test_create_default_config() {
        let p = PathBuf::from("/tmp/test_pretty_git_prompt_config1");
        if Path::new(&p).exists() {
            remove_file(p.clone());
        }

        let result = create_default_config(p.clone());
        assert!(result.is_ok());

        let mut file = File::open(p.clone()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents);
        assert_eq!(contents, DEFAULT_CONF);

        remove_file(p.clone());
    }
    #[test]
    fn test_create_default_config_when_exists() {
        let p = PathBuf::from("/tmp/test_pretty_git_prompt_config2");
        OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(p.clone());
        assert!(Path::new(&p).exists());

        let result = create_default_config(p.clone());
        assert!(result.is_err());

        remove_file(p.clone());
    }
    #[test]
    fn test_load_default_config() {
        let p = PathBuf::from("/tmp/test_pretty_git_prompt_config3");
        if Path::new(&p).exists() {
            remove_file(p.clone());
        }

        let result = create_default_config(p.clone());
        assert!(result.is_ok());

        let c = get_configuration(None);

        remove_file(p.clone());
    }
}
