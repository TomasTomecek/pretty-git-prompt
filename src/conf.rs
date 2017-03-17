extern crate yaml_rust;
use self::yaml_rust::{YamlLoader, Yaml};

use std::collections::btree_map::BTreeMap;

// TODO: add comments to the yaml
// TODO: track_remotes -> branch -> '' -- track current remote branch
// TODO: load this file from disk
// TODO: add option to create default
static DEFAULT_CONF: &'static str = "
---
symbols:
    new: '✚'
    changed: '■'
    staged: '●'
    conflicts: '✖'
    difference_ahead: '↑'
    difference_behind: '↓'
monitor_remotes:
    upstream: master
colors:
    branch: 'blue'
    remote_difference: 'white'
    new: 'red'
    changed: 'red'
    staged: 'green'
    conflicts: 'yellow'
";


pub struct Conf {
    c: Yaml,
}

impl Conf {
    pub fn new(yaml: Yaml) -> Conf {
        Conf { c: yaml }
    }

    pub fn get_new_symbol(&self) -> String {
        String::from(self.c["symbols"]["new"].as_str().unwrap())
    }
    pub fn get_changed_symbol(&self) -> String {
        String::from(self.c["symbols"]["changed"].as_str().unwrap())
    }
    pub fn get_staged_symbol(&self) -> String {
        String::from(self.c["symbols"]["staged"].as_str().unwrap())
    }
    pub fn get_conflicts_symbol(&self) -> String {
        String::from(self.c["symbols"]["conflicts"].as_str().unwrap())
    }
    pub fn get_difference_ahead_symbol(&self) -> String {
        String::from(self.c["symbols"]["difference_ahead"].as_str().unwrap())
    }
    pub fn get_difference_behind_symbol(&self) -> String {
        String::from(self.c["symbols"]["difference_behind"].as_str().unwrap())
    }

    pub fn get_remotes_monitoring(&self) -> BTreeMap<String, String> {
        let remotes = self.c["monitor_remotes"].as_hash().unwrap();
        let mut response: BTreeMap<String, String> = BTreeMap::new();
        for (k, v) in remotes {
            response.insert(String::from(k.as_str().unwrap()), String::from(v.as_str().unwrap()));
        }
        response
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

pub fn get_default_configuration() -> Conf {
    let docs = YamlLoader::load_from_str(DEFAULT_CONF).unwrap();
    Conf::new(docs[0].clone())
}

#[test]
fn test_default_new_symbol() {
    let c = get_default_configuration();
    assert_eq!(c.get_new_symbol(), "✚");
}
#[test]
fn test_default_changed_symbol() {
    let c = get_default_configuration();
    assert_eq!(c.get_changed_symbol(), "■");
}
#[test]
fn test_default_staged_symbol() {
    let c = get_default_configuration();
    assert_eq!(c.get_staged_symbol(), "●");
}
#[test]
fn test_default_conflicts_symbol() {
    let c = get_default_configuration();
    assert_eq!(c.get_conflicts_symbol(), "✖");
}
#[test]
fn test_difference_ahead_symbol() {
    let c = get_default_configuration();
    assert_eq!(c.get_difference_ahead_symbol(), "↑");
}
#[test]
fn test_difference_behind_symbol() {
    let c = get_default_configuration();
    assert_eq!(c.get_difference_behind_symbol(), "↓");
}
#[test]
fn test_default_monitored_remotes() {
    let c = get_default_configuration();
    let remotes = c.get_remotes_monitoring();
    assert_eq!(remotes["upstream"], String::from("master"));
}

#[test]
fn test_monitored_remotes_ordering() {
    let config = "
    monitor_remotes:
        a: b
        c: d
        e: f
    ";
    let docs = YamlLoader::load_from_str(config).unwrap();
    let c = Conf::new(docs[0].clone());
    let remotes = c.get_remotes_monitoring();
    let mut iter = remotes.iter();
    assert_eq!(iter.next().unwrap(), (&String::from("a"), &String::from("b")));
    assert_eq!(iter.next().unwrap(), (&String::from("c"), &String::from("d")));
    assert_eq!(iter.next().unwrap(), (&String::from("e"), &String::from("f")));
}
#[test]
fn test_default_branch_color() {
    let c = get_default_configuration();
    assert_eq!(c.get_branch_color(), "blue");
}
#[test]
fn test_default_remote_difference_color() {
    let c = get_default_configuration();
    assert_eq!(c.get_remote_difference_color(), "white");
}
#[test]
fn test_default_new_color() {
    let c = get_default_configuration();
    assert_eq!(c.get_new_color(), "red");
}
#[test]
fn test_default_changed_color() {
    let c = get_default_configuration();
    assert_eq!(c.get_changed_color(), "red");
}
#[test]
fn test_default_staged_color() {
    let c = get_default_configuration();
    assert_eq!(c.get_staged_color(), "green");
}
#[test]
fn test_default_conflicts_color() {
    let c = get_default_configuration();
    assert_eq!(c.get_conflicts_color(), "yellow");
}
