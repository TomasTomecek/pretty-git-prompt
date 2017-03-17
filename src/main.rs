// TODO:
//  * add option to debug: print all errors

mod format;
mod cli;
mod conf;
extern crate git2;

use cli::cli;
use format::{Format,FormatType,FormatEntry};
use conf::{Conf,get_default_configuration};

use std::collections::HashMap;

use git2::*;

// used to index values in map
static CHANGED_KEY: &'static str = "changed";
static NEW_KEY: &'static str = "new";
static STAGED_KEY: &'static str = "staged";
static CONFLICTS_KEY: &'static str = "conflicts";


fn get_branch_remote(reference: Reference) -> Option<Oid> {
    let b = Branch::wrap(reference);
    let upstream = match b.upstream() {
        Ok(u) => u,
        Err(_) => return None,
    };
    upstream.get().target()
}


struct Backend {
    repo: Repository,
}

impl Backend {
    fn get_head(&self) -> Option<Reference> {
        match self.repo.head() {
            Ok(head) => Some(head),
            Err(_) => {
                match self.repo.find_reference("HEAD") {
                    Ok(x) => Some(x),
                    Err(_) => None,
                }
            },
        }
    }

    fn get_current_branch_oid(&self) -> Option<Oid> {
        let head = self.get_head();
        match head {
            Some(v) => v.target(),
            None => None,
        }
    }

    // TODO: return Option
    fn get_current_branch_name(&self) -> String {
        let blank = String::from("");
        let h = match self.get_head() {
            Some(v) => {
                match v.resolve() {
                    Ok(y) => y,
                    Err(_) => v
                }
            }
            None => return blank,
        };

        let s = h.shorthand();
        if s.is_some() {
            let ref_name = s.unwrap();
            if ref_name != "HEAD" {
                return ref_name.to_string();
            } else {
                let ref_name = h.symbolic_target();
                if ref_name.is_some() {
                    let ref_name_string = ref_name.unwrap().to_string();
                    let mut path: Vec<&str> = ref_name_string.split('/').collect();
                    let branch_short = path.pop();
                    if branch_short.is_some() {
                        return branch_short.unwrap().to_string();
                    }
                }
            }
        }

        let hash = match h.target() {
            Some(v) => v.to_string(),
            None => blank,
        };
        if hash.len() >= 8 {
            let (s, _) = hash.split_at(7);
            s.to_string()
        } else {
            hash
        }
    }

    fn get_current_branch_remote_oid(&self) -> Option<Oid> {
        match self.get_head() {
            Some(r) => get_branch_remote(r),
            None => None
        }
    }

    fn get_current_branch_ahead_behind(&self) -> Option<(usize, usize)> {
        let rm_oid = match self.get_current_branch_remote_oid() {
            Some(r) => r,
            None => return None
        };
        let oid = match self.get_current_branch_oid() {
            Some(r) => r,
            None => return None
        };
        let res = self.repo.graph_ahead_behind(oid, rm_oid);
        match res {
            Ok(r) => Some(r),
            Err(_) => None
        }
    }

    fn get_remote_branch_ahead_behind(&self, remote_name: &str, branch_name: &str) -> Option<(usize, usize)>  {
        let remote_reference = match self.find_remote_branch(remote_name, branch_name) {
            Ok(u) => u.into_reference(),
            Err(_) => return None,
        };
        let remote_reference_oid = match remote_reference.target() {
            Some(u) => u,
            None => return None,
        };
        let oid = match self.get_current_branch_oid() {
            Some(r) => r,
            None => return None
        };
        let res = self.repo.graph_ahead_behind(oid, remote_reference_oid);
        match res {
            Ok(r) => Some(r),
            Err(_) => None
        }
    }

    fn find_remote_branch(&self, remote_name: &str, branch_name: &str) -> Result<Branch, Error> {
        let cur_branch_name = self.get_current_branch_name();
        let b = match branch_name {
            x if x.is_empty() => &cur_branch_name,
            y => y
        };
        let remote_branch_name = format!("{}/{}", remote_name, b);
        self.repo.find_branch(&remote_branch_name, BranchType::Remote)
    }

    fn get_status(&self) -> Option<Statuses> {
        let mut so = StatusOptions::new();
        let mut opts = so.show(StatusShow::IndexAndWorkdir);
        opts.include_untracked(true);
        match self.repo.statuses(Some(&mut opts)) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    fn get_repository_state(&self) -> String {
        let state = self.repo.state();
        match state {
            RepositoryState::Clean => String::from(""),
            RepositoryState::Merge => String::from("merge"),
            RepositoryState::Revert | RepositoryState::RevertSequence => String::from("revert"),
            RepositoryState::CherryPick | RepositoryState::CherryPickSequence => String::from("cherry-pick"),
            RepositoryState::Bisect => String::from("bisect"),
            RepositoryState::Rebase |
                RepositoryState::RebaseInteractive |
                RepositoryState::RebaseMerge => String::from("rebase"),
            RepositoryState::ApplyMailbox | RepositoryState::ApplyMailboxOrRebase => String::from("apply"),
        }
    }

    fn get_file_status(&self) -> Option<HashMap<&str, u32>> {
        let mut d = HashMap::new();

        let changed = STATUS_WT_MODIFIED | STATUS_WT_DELETED | STATUS_WT_TYPECHANGE | STATUS_WT_RENAMED;
        let staged = STATUS_INDEX_MODIFIED | STATUS_INDEX_DELETED | STATUS_INDEX_TYPECHANGE | STATUS_INDEX_RENAMED | STATUS_INDEX_NEW;

        let statuses = match self.get_status() {
            Some(x) => x,
            None => return None,
        };

        for s in statuses.iter() {
            let file_status = s.status();
            // println!("{}", s.path().unwrap());

            if file_status.intersects(changed) {
                let counter = d.entry(CHANGED_KEY).or_insert(0);
                *counter += 1;
            };
            if file_status.contains(STATUS_WT_NEW) {
                let counter = d.entry(NEW_KEY).or_insert(0);
                *counter += 1;
            };
            if file_status.intersects(staged) {
                let counter = d.entry(STAGED_KEY).or_insert(0);
                *counter += 1;
            };
            if file_status.intersects(STATUS_CONFLICTED) {
                let counter = d.entry(CONFLICTS_KEY).or_insert(0);
                *counter += 1;
            };
        }
        Some(d)
    }
}

// struct for displaying all the data -- the actual program
struct Output {
    out: Vec<String>,
    format: Format,
    backend: Backend,
    conf: Conf,
}

impl Output {
    pub fn new(format_type: FormatType, backend: Backend, conf: Conf) -> Output {
        let out: Vec<String> = Vec::new();
        Output { out: out, backend: backend, format: Format{ format_type: format_type }, conf: conf }
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

    let x = matches.value_of("color_mode").unwrap();
    let ft: FormatType = match x {
        "zsh" => FormatType::Zsh,
        "no" | _ => FormatType::NoColor,
    };
    let mut output = Output::new(ft, backend, conf);

    output.populate();
    output.output();
}
