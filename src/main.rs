// TODO:
//  * add option to debug: print all errors

extern crate clap;
extern crate git2;
extern crate yaml_rust;

use backend::Backend;
use cli::cli;
use conf::{Conf,get_default_configuration,create_default_config};
use constants::*;
use format::{Format,FormatType,FormatEntry};

use git2::Repository;

mod backend;
mod cli;
mod conf;
mod constants;
mod format;


// struct for displaying all the data -- the actual program
struct Program {
    out: Vec<String>,
    format: Format,
    backend: Backend,
    conf: Conf,
}

impl Program {
    pub fn new(format_type: FormatType, backend: Backend, conf: Conf) -> Program {
        let out: Vec<String> = Vec::new();
        Program { out: out, backend: backend, format: Format{ format_type: format_type }, conf: conf }
    }

    // add repository state to output buffer
    fn add_repository_state(&mut self) {
        let repo_state = self.backend.get_repository_state();
        if !repo_state.is_empty() {
            let o = self.f(&repo_state, "");
            self.out.push(o);
        }
    }

    // master↑3↓4
    fn add_local_branch_state(&mut self) {
        let mut local: String = self.f(&self.backend.get_current_branch_name(), &self.conf.get_branch_color());
        let (ahead, behind) = match self.backend.get_current_branch_ahead_behind() {
            Some(x) => x,
            None => (0, 0),
        };
        if ahead > 0 {
            local += &self.f(
                &format!("{}{}", self.conf.get_difference_ahead_symbol(), ahead),
                &self.conf.get_remote_difference_color()
            );
        }
        if behind > 0 {
            local += &self.f(
                &format!("{}{}", self.conf.get_difference_behind_symbol(), behind),
                &self.conf.get_remote_difference_color()
            );
        }
        self.out.push(local);
    }

    // upstream↑2↓1
    fn add_remote_branches_state(&mut self) {
        for (remote_name, branch_name) in self.conf.get_remotes_monitoring() {
            let (ahead, behind) = match self.backend.get_remote_branch_ahead_behind(&remote_name, &branch_name) {
                Some(x) => x,
                None => (0, 0),
            };
            if ahead > 0 || behind > 0 {
                let mut local: String = self.f(
                    &format!("{}/{}", remote_name, branch_name),
                    &self.conf.get_branch_color()
                );
                if ahead > 0 {
                    local += &self.f(
                        &format!("{}{}", self.conf.get_difference_ahead_symbol(), ahead),
                        &self.conf.get_remote_difference_color()
                    );
                }
                if behind > 0 {
                    local += &self.f(
                        &format!("{}{}", self.conf.get_difference_behind_symbol(), behind),
                        &self.conf.get_remote_difference_color()
                    );
                }
                self.out.push(local);
            }
        }
    }

    fn f(&self, text: &str, color: &str) -> String {
        self.format.format(FormatEntry{text: String::from(text), color: String::from(color)})
    }

    fn add_file_status(&mut self) {
        if let Some(s) = self.backend.get_file_status() {
            if !s.is_empty() {
                let mut o = String::from("");

                if let Some(x) = s.get(NEW_KEY) {
                     o += &self.f(&format!("{}{}", self.conf.get_new_symbol(), x), &self.conf.get_new_color());
                }
                if let Some(x) = s.get(CHANGED_KEY) {
                     o += &self.f(&format!("{}{}", self.conf.get_changed_symbol(), x), &self.conf.get_changed_color());
                }
                if let Some(x) = s.get(STAGED_KEY) {
                     o += &self.f(&format!("{}{}", self.conf.get_staged_symbol(), x), &self.conf.get_staged_color());
                }
                if let Some(x) = s.get(CONFLICTS_KEY) {
                     o += &self.f(&format!("{}{}", self.conf.get_conflicts_symbol(), x), &self.conf.get_conflicts_color());
                }
                self.out.push(o);
            }
        }
    }

    fn populate(&mut self) {
        self.add_repository_state();
        self.add_local_branch_state();
        self.add_remote_branches_state();
        self.add_file_status();
    }

    // print output buffer
    fn output(&self) {
        // println!("{}", out.len());
        println!("{}", self.out.join("|"));
    }
}

fn main() {
    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(_) => return (),
    };
    let backend = Backend{ repo: repo };
    let conf = get_default_configuration();
    let app = cli();
    let matches = app.get_matches();

    if matches.is_present(CLI_DEFAULT_CONFIG_SUBC_NAME) {
        let p = get_default_config_path();
        return match create_default_config(p.clone()) {
            Ok(path) => {
                println!("Configuration file created at \"{}\"", path);
                return ();
            }
            Err(e) => {
                println!("Failed to create configuration file \"{}\": {}", p.to_str().unwrap(), e);
                return ();
            }
        };
    }

    let x = matches.value_of("color_mode").unwrap();
    let ft: FormatType = match x {
        "zsh" => FormatType::Zsh,
        "no" | _ => FormatType::NoColor,
    };
    let mut output = Program::new(ft, backend, conf);

    output.populate();
    output.output();
}
