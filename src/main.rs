// TODO:
//  * add option to debug: print all errors
//  * colors

extern crate git2;

use std::collections::HashMap;

use git2::*;

static CHANGED_SYMBOL: &'static str = "■";
static NEW_SYMBOL: &'static str = "✚";
static STAGED_SYMBOL: &'static str = "●";
static CONFLICTED_SYMBOL: &'static str = "✖";


struct Program {
    repo: Repository,
}

fn get_branch_remote(reference: Reference) -> Option<Oid> {
    let b = Branch::wrap(reference);
    let upstream = match b.upstream() {
        Ok(u) => u,
        Err(_) => return None,
    };
    upstream.get().target()
}

fn zsh_colorize(s: &str, color: &str) -> String {
    // {% ... %} correct word wrap with zsh
    format!("%{{%F{{{color}}}%}}{s}%{{%f%}}", color=color, s=s)
}

impl Program {
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

    fn get_upstream_branch_ahead_behind(&self) -> Option<(usize, usize)>  {
        let upstream_reference = match self.find_upstream_repo_branch() {
            Ok(u) => u.into_reference(),
            Err(_) => return None,
        };
        let upstream_reference_oid = match upstream_reference.target() {
            Some(u) => u,
            None => return None,
        };
        let oid = match self.get_current_branch_oid() {
            Some(r) => r,
            None => return None
        };
        let res = self.repo.graph_ahead_behind(oid, upstream_reference_oid);
        match res {
            Ok(r) => Some(r),
            Err(_) => None
        }
    }

    fn find_upstream_repo_branch(&self) -> Result<Branch, Error> {
        let us_branch_name = format!("{}{}", "upstream/", self.get_current_branch_name());
        self.repo.find_branch(&us_branch_name, BranchType::Remote)
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
                let counter = d.entry(CHANGED_SYMBOL).or_insert(0);
                *counter += 1;
            };
            if file_status.contains(STATUS_WT_NEW) {
                let counter = d.entry(NEW_SYMBOL).or_insert(0);
                *counter += 1;
            };
            if file_status.intersects(staged) {
                let counter = d.entry(STAGED_SYMBOL).or_insert(0);
                *counter += 1;
            };
            if file_status.intersects(STATUS_CONFLICTED) {
                let counter = d.entry(CONFLICTED_SYMBOL).or_insert(0);
                *counter += 1;
            };
        }
        Some(d)
    }

    fn run(&self) {
        let mut out: Vec<String> = Vec::new();

        let repo_state = self.get_repository_state();
        if !repo_state.is_empty() {
            out.push(repo_state);
        }

        // master↑3↓4
        let mut local = zsh_colorize(&self.get_current_branch_name(), "blue");
        let (ahead, behind) = match self.get_current_branch_ahead_behind() {
            Some(x) => x,
            None => (0, 0),
        };
        if ahead > 0 { local += &zsh_colorize(&format!("↑{}", ahead), "white"); }
        if behind > 0 { local += &zsh_colorize(&format!("↓{}", behind), "white"); }
        out.push(local);

        // upstream↑2↓1
        let (ahead, behind) = match self.get_upstream_branch_ahead_behind() {
            Some(x) => x,
            None => (0, 0),
        };
        if ahead > 0 || behind > 0 {
            let mut local = zsh_colorize("u", "blue");
            if ahead > 0 { local += &zsh_colorize(&format!("↑{}", ahead), "white"); }
            if behind > 0 { local += &zsh_colorize(&format!("↓{}", behind), "white"); }
            out.push(local);
        }

        if let Some(s) = self.get_file_status() {
            if !s.is_empty() {
                let mut o = String::from("");

                if let Some(x) = s.get(CHANGED_SYMBOL) {
                     o += &zsh_colorize(&format!("{}{}", CHANGED_SYMBOL, x), "red");
                }
                if let Some(x) = s.get(CONFLICTED_SYMBOL) {
                     o += &zsh_colorize(&format!("{}{}", CONFLICTED_SYMBOL, x), "yellow");
                };
                if let Some(x) = s.get(NEW_SYMBOL) {
                     o += &zsh_colorize(&format!("{}{}", NEW_SYMBOL, x), "red");
                };
                if let Some(x) = s.get(STAGED_SYMBOL) {
                     o += &zsh_colorize(&format!("{}{}", STAGED_SYMBOL, x), "green");
                };
                out.push(o);
            }
        }

        // println!("{}", out.len());
        println!("{}", out.join("|"));
    }
}


fn main() {
    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(_) => return (),
    };
    let program = Program{ repo: repo };
    program.run();
}
