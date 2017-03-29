use std::collections::HashMap;

use constants::{CHANGED_KEY,NEW_KEY,STAGED_KEY,CONFLICTS_KEY};

use git2::*;


fn get_branch_remote(reference: Reference) -> Option<Oid> {
    let b = Branch::wrap(reference);
    let upstream = match b.upstream() {
        Ok(u) => u,
        Err(_) => return None,
    };
    upstream.get().target()
}


pub struct Backend {
    pub repo: Repository,
    pub debug: bool
}

impl Backend {
    fn get_head(&self) -> Option<Reference> {
        match self.repo.head() {
            Ok(head) => Some(head),
            Err(_) => {
                match self.repo.find_reference("HEAD") {
                    Ok(x) => Some(x),
                    Err(e) => {
                        log!(self, "reference HEAD: {}", e);
                        return None
                    },
                }
            },
        }
    }

    fn get_current_branch_oid(&self) -> Option<Oid> {
        let head = self.get_head();
        match head {
            Some(v) => v.target(),
            None => {
                log!(self, "Failed to find Oid for HEAD.");
                return None
            },
        }
    }

    // TODO: return Option
    pub fn get_current_branch_name(&self) -> String {
        let blank = String::from("");
        let h = match self.get_head() {
            Some(v) => {
                match v.resolve() {
                    Ok(y) => {
                        log!(self, "Branch name is {:?}", y.name());
                        y
                    },
                    Err(e) => {
                        log!(self, "Branch name is {:?}, error: {:?}", v.name(), e);
                        v
                    }
                }
            }
            None => {
                log!(self, "No branch name found");
                return blank;
            }
        };

        if let Some(ref_name_string) = h.shorthand() {
            if ref_name_string != "HEAD" {
                let s = ref_name_string.to_string();
                log!(self, "Ref name is: {}", s);
                return s;
            } else {
                if let Some(ref_name) = h.symbolic_target() {
                    let ref_name_string = ref_name.to_string();
                    log!(self, "Ref name is: {}", ref_name_string);
                    let mut path: Vec<&str> = ref_name_string.split('/').collect();
                    if let Some(branch_short) = path.pop() {
                        let s = branch_short.to_string();
                        log!(self, "Ref name is: {}", s);
                        return s
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

    pub fn get_current_branch_ahead_behind(&self) -> Option<(usize, usize)> {
        let rm_oid = match self.get_current_branch_remote_oid() {
            Some(r) => r,
            None => {
                log!(self, "Can't find remote counterpart of current branch.");
                return None
            },

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

    pub fn get_remote_branch_ahead_behind(&self, remote_name: &str, branch_name: &str) -> Option<(usize, usize)>  {
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

    // TODO: use branch tracking here
    fn find_remote_branch(&self, remote_name: &str, branch_name: &str) -> Result<Branch, Error> {
        let cur_branch_name = self.get_current_branch_name();
        let b = match branch_name {
            x if x.is_empty() => &cur_branch_name,
            y => y
        };
        let remote_branch_name = format!("{}/{}", remote_name, b);
        self.repo.find_branch(&remote_branch_name, BranchType::Remote)
    }

    pub fn get_status(&self) -> Option<Statuses> {
        let mut so = StatusOptions::new();
        let mut opts = so.show(StatusShow::IndexAndWorkdir);
        opts.include_untracked(true);
        match self.repo.statuses(Some(&mut opts)) {
            Ok(s) => Some(s),
            Err(e) => {
                log!(self, "Unable to get status of repository: {:?}", e);
                None
            }
        }
    }

    pub fn get_repository_state(&self) -> String {
        let state = self.repo.state();
        match state {
            RepositoryState::Clean => String::from(""),  // XXX: this seems to be hardcoded
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

    pub fn get_file_status(&self) -> Option<HashMap<&str, u32>> {
        let mut d = HashMap::new();

        let changed = STATUS_WT_MODIFIED | STATUS_WT_DELETED | STATUS_WT_TYPECHANGE | STATUS_WT_RENAMED;
        let staged = STATUS_INDEX_MODIFIED | STATUS_INDEX_DELETED | STATUS_INDEX_TYPECHANGE | STATUS_INDEX_RENAMED | STATUS_INDEX_NEW;

        let statuses = match self.get_status() {
            Some(x) => x,
            None => return None,
        };

        for s in statuses.iter() {
            let file_status = s.status();
            log!(self, "{}", s.path().unwrap());

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
