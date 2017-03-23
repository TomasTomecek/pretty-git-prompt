use std::collections::btree_map::BTreeMap;
use std::fs::{File,OpenOptions};
use std::io;
use std::io::{Write,Read};
use std::path::{Path,PathBuf};

use constants::{get_default_config_path};

use yaml_rust::{YamlLoader, Yaml};

// TODO: supply different configs for different shells, default to no color
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
        pre_format: '%{%F{014}%}'
        post_format: '%{%f%}'
    changed:
        label: 'Δ'
        pre_format: '%{%B%F{red}%}'
        post_format: '%{%b%f%}'
    staged:
        label: '▶'
        pre_format: '%{%F{green}%}'
        post_format: '%{%f%}'
    conflicts:
        label: '✖'
        pre_format: '%{%F{yellow}%}'
        post_format: '%{%f%}'
    difference_ahead:
        label: '↑'
        pre_format: '%{%F{white}%}'
        post_format: '%{%f%}'
    difference_behind:
        label: '↓'
        pre_format: '%{%F{white}%}'
        post_format: '%{%f%}'

# monitor status against different remotes (optional), type dict
# track history divergence
monitor_remotes:
    origin:
        display_if_uptodate: true
    # remote name (optional), type dict
    upstream:
        # remote branch name (optional), type string
        # if omitted look for remotely tracked one
        # git branch --set-upstream-to
        branch: master
        # prefix label (optional), type string
        # if omitted display full text: $remote/$branch
        # label: ''
        # display the remote even if there is no difference with current branch (required), type bool
        display_if_uptodate: false
";

pub struct MonitoredRemote {
    pub remote_name: String,
    pub branch: Option<String>,
    pub label: Option<String>,
    pub display_if_uptodate: bool,
}

impl MonitoredRemote {
    fn new(remote_config: &Yaml, remote_name: &str) -> MonitoredRemote {
        let uptodate: bool;
        let ref display_if_uptodate = remote_config["display_if_uptodate"];
        if display_if_uptodate.is_badvalue() || display_if_uptodate.is_null() {
            panic!("'display_if_uptodate' key is required and has to be bool");
        }
        uptodate = display_if_uptodate.as_bool().unwrap();

        let mut mr = MonitoredRemote{ remote_name: String::from(remote_name), branch: None, label: None, display_if_uptodate: uptodate };

        let ref branch = remote_config["branch"];
        if !(branch.is_badvalue() || branch.is_null()) {
            mr.branch = Some(String::from(branch.as_str().unwrap()));
        }

        let ref label = remote_config["label"];
        if !(label.is_badvalue() || label.is_null()) {
            mr.label = Some(String::from(label.as_str().unwrap()));
        }

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
        let remotes = remotes_yaml.as_hash().unwrap();
        let mut response: Vec <MonitoredRemote> = Vec::new();
        for (k, v) in remotes {
            response.push(MonitoredRemote::new(v, k.as_str().unwrap()));
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

pub fn load_configuration_from_file() -> Result<String, io::Error> {
    let path = get_default_config_path();
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

// TODO: allow overriding config from CLI
pub fn get_configuration(force_default_config: bool) -> Conf {
    let mut content: String;
    if force_default_config {
        content = String::from(DEFAULT_CONF);
    } else {
        content = match load_configuration_from_file() {
            Ok(f) => f,
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
        let c = get_configuration(true);
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
        let c = get_configuration(true);
        let o = c.get_changed_value();
        let v = o.unwrap();
        assert_eq!(v.label, "Δ");
    }
    #[test]
    fn test_default_staged_symbol() {
        let c = get_configuration(true);
        let o = c.get_staged_value();
        let v = o.unwrap();
        assert_eq!(v.label, "▶");
    }
    #[test]
    fn test_default_conflicts_symbol() {
        let c = get_configuration(true);
        let o = c.get_conflicts_value();
        let v = o.unwrap();
        assert_eq!(v.label, "✖");
    }
    #[test]
    fn test_difference_ahead_symbol() {
        let c = get_configuration(true);
        let o = c.get_difference_ahead_value();
        let v = o.unwrap();
        assert_eq!(v.label, "↑");
    }
    #[test]
    fn test_difference_behind_symbol() {
        let c = get_configuration(true);
        let o = c.get_difference_behind_value();
        let v = o.unwrap();
        assert_eq!(v.label, "↓");
    }

    #[test]
    fn test_default_monitored_remotes() {
        let c = get_configuration(true);
        let remotes = c.get_remotes_monitoring().unwrap();
        let ref origin = remotes[0];
        assert_eq!(origin.display_if_uptodate, true);
        let ref upstream = remotes[1];
        assert_eq!(upstream.display_if_uptodate, false);
        let ref b = upstream.branch;
        assert_eq!(b.clone().unwrap(), "master");
        assert_eq!(upstream.label, None);
    }
    #[test]
    fn test_monitored_remotes_ordering() {
        let config_text = "monitor_remotes:
    x:
        display_if_uptodate: true
    y:
        display_if_uptodate: true
    z:
        display_if_uptodate: true
";
        let docs = YamlLoader::load_from_str(config_text).unwrap();
        let c = Conf::new(docs[0].clone());

        let remotes = c.get_remotes_monitoring().unwrap();
        let mut iter = remotes.iter();
        assert_eq!(iter.next().unwrap().remote_name, "x");
        assert_eq!(iter.next().unwrap().remote_name, "y");
        assert_eq!(iter.next().unwrap().remote_name, "z");
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

        let c = get_configuration(true);

        remove_file(p.clone());
    }
}
