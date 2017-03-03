// TODO:
//  * add option to debug: print all errors
//  * show file status
//  * show repo state
//  * colors

extern crate git2;

use std::io::{self, Write};

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

    // fn get_current_branch_remote(&self) -> String {
    //     let oid = self.get_current_branch_oid();
    //     let b = Branch::wrap(oid);
    //     let upstream = b.upstream().ok().unwrap();
    //     String::from(upstream.name().ok().unwrap().unwrap())
    // }

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
        let statuses = match self.repo.statuses(Some(&mut opts)) {
            Ok(s) => Some(s),
            Err(e) => None,
        };
        statuses
    }

    fn get_repository_state(&self) -> RepositoryState {
        self.repo.state()
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

        // let statuses = self.get_status();
        // for s in statuses.iter() {
        //     let file_status = s.status();
        //     println!("{}", s.path().unwrap());
        //     if file_status.intersects(
        //         STATUS_WT_MODIFIED | STATUS_WT_DELETED | STATUS_WT_TYPECHANGE | STATUS_WT_RENAMED
        //     ) {
        //         println!("changes");
        //     };
        //     if file_status.contains(STATUS_WT_NEW) {
        //         println!("new files");
        //     };
        //     if file_status.intersects(
        //         STATUS_INDEX_MODIFIED | STATUS_INDEX_DELETED | STATUS_INDEX_TYPECHANGE | STATUS_INDEX_RENAMED | STATUS_INDEX_NEW
        //     ) {
        //         println!("index update");
        //     };
        // }
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
