// TODO:
//  * add option to debug: print all errors
//  * show repo state
//  * colors

extern crate git2;

use std::io::{self, Write};

use std::collections::HashMap;

use git2::Error;
use git2::{Repository,Branch,BranchType,Oid,Reference,StatusShow};
use git2::{StatusOptions,Statuses,Status,RepositoryState};
use git2::{STATUS_WT_MODIFIED,STATUS_WT_DELETED,STATUS_WT_NEW,STATUS_WT_TYPECHANGE,STATUS_WT_RENAMED};
use git2::{STATUS_INDEX_MODIFIED,STATUS_INDEX_DELETED,STATUS_INDEX_NEW,STATUS_INDEX_TYPECHANGE,STATUS_INDEX_RENAMED};


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

impl Program {
    fn get_head(&self) -> Option<Reference> {
        match self.repo.head() {
            Ok(head) => Some(head),
            Err(e) => None,
        }
    }

    fn get_current_branch_oid(&self) -> Option<Oid> {
        let head = self.get_head();
        match head {
            Some(v) => v.target(),
            None => None,
        }
    }

    fn get_current_branch_name(&self) -> String {
        let blank = String::from("");
        let h = match self.get_head() {
            Some(v) => v,
            None => return blank,
        };
        if h.is_branch() {
            match h.shorthand() {
                Some(v) => v.to_string(),
                None => blank,
            }
        } else {
            let hash = match h.target() {
                Some(v) => v.to_string(),
                None => blank,
            };
            let (s, _) = hash.split_at(7);
            s.to_string()
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
            Err(e) => None,
        }
    }

    fn get_repository_state(&self) -> RepositoryState {
        self.repo.state()
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
                let counter = d.entry("U").or_insert(0);
                *counter += 1;
            };
            if file_status.contains(STATUS_WT_NEW) {
                let counter = d.entry("N").or_insert(0);
                *counter += 1;
            };
            if file_status.intersects(staged) {
                let counter = d.entry("A").or_insert(0);
                *counter += 1;
            };
        }
        Some(d)
    }

    fn run(&self) {
        print!("{}|", self.get_current_branch_name());

        match self.get_current_branch_ahead_behind() {
            // FIXME: cover case behind && ahead
            Some((_, behind)) if behind > 0 => print!("-{}", behind),
            Some((ahead, _)) if ahead > 0 => print!("+{}", ahead),
            Some(_) => {}
            None => {}
        }

        match self.get_upstream_branch_ahead_behind() {
            Some((ahead, behind)) => print!("U+{}-{}|", ahead, behind),
            None => {}
        }

        let statuses = match self.get_file_status() {
            Some(s) => {
                for (k, v) in s {
                    if v > 0 {
                        print!("{}{}", k, v);
                    }
                }
            },
            None => {}
        };

        // println!("{:?}", program.get_repository_state());

        print!("\n");
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
