use std::fmt;
use std::collections::HashMap;
use std::cell::RefCell;

use constants::{CHANGED_KEY,NEW_KEY,STAGED_KEY,CONFLICTS_KEY};

use git2::*;


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
struct Cache {
    current_branch_name: RefCell<Option<String>>,
    // TODO: Reference can't be cached (can't be cloned)
    //       implement via OID
    // head: RefCell<Option<Oid>>,
    file_statuses: RefCell<Option<HashMap<String, u32>>>
}

pub struct Backend {
    cache: Cache,
    pub repo: Repository,
    pub debug: bool,
}

impl fmt::Debug for Backend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Backend {{ cache: {:?}, repo: ?, debug: {:?} }}", self.cache, self.debug)
    }
}


#[derive(Clone)]
pub struct BranchAheadBehind {
    pub local_branch_name: Option<String>,
    pub remote_branch_name: Option<String>,
    pub remote_name: Option<String>,
    pub ahead: usize,
    pub behind: usize
}

impl BranchAheadBehind {
    fn new(l: Option<String>) -> BranchAheadBehind {
        BranchAheadBehind{ local_branch_name: l, remote_branch_name: None, remote_name: None,
                           ahead: 0, behind: 0 }
    }
}


#[derive(Clone)]
struct RefPair {
    remote_name: String,
    branch_name: String,
    oid: Oid,
}


impl Cache {
    pub fn new() -> Cache {
        Cache{
            current_branch_name: RefCell::new(None),
            file_statuses: RefCell::new(None),
        }
    }

    fn set_current_branch_name(&self, n: Option<String>) {
        let mut c = self.current_branch_name.borrow_mut();
        *c = n;
    }

    fn is_current_branch_set(&self) -> bool {
        self.current_branch_name.borrow().is_some()
    }

    fn get_current_branch(&self) -> Option<String> {
        self.current_branch_name.borrow().clone()
    }

    fn set_file_statuses(&self, n: Option<HashMap<String, u32>>) {
        let mut c = self.file_statuses.borrow_mut();
        *c = n;
    }

    fn is_file_statuses_set(&self) -> bool {
        self.file_statuses.borrow().is_some()
    }

    fn get_file_statuses(&mut self) -> Option<HashMap<String, u32>> {
        self.file_statuses.borrow().clone()
    }
}


impl Backend {
    pub fn new(repo: Repository, debug: bool) -> Backend {
        Backend{ repo: repo, debug: debug, cache: Cache::new() }
    }

    fn get_head(&self) -> Option<Reference> {
        match self.repo.head() {
            Ok(head) => Some(head),
            Err(e2) => {
                log!(self, "Can't get HEAD: {}", e2);
                match self.repo.find_reference("HEAD") {
                    Ok(x) => {
                        log!(self, "Found HEAD directly: {:?}", x.name());
                        Some(x)
                    },
                    Err(e) => {
                        log!(self, "reference HEAD: {}", e);
                        None
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
                None
            },
        }
    }

    fn resolve_symbolic_reference<'a>(&self, reference: Option<Reference<'a>>) -> Option<Reference<'a>> {
        let s_reference = match reference {
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
                return None;
            }
        };
        Some(s_reference)
    }

    fn get_branch_name_for_reference(&self, r: Reference) -> Option<String> {
        if let Some(ref_name_string) = r.shorthand() {
            if ref_name_string != "HEAD" {
                let s = ref_name_string.to_string();
                log!(self, "Shorthand for reference is: {}", s);
                return Some(s);
            } else if let Some(ref_name) = r.symbolic_target() {
                let ref_name_string = ref_name.to_string();
                log!(self, "shorthand = HEAD, links to: {}", ref_name_string);
                let mut path: Vec<&str> = ref_name_string.split('/').collect();
                if let Some(branch_short) = path.pop() {
                    let s = branch_short.to_string();
                    log!(self, "Last part of full name is: {}", s);
                    return Some(s);
                }
            }
        }

        // we're getting the commit hash as name
        match r.target() {
            Some(v) => {
                let hash_str = v.to_string();
                if hash_str.len() >= 8 {
                    let (s, _) = hash_str.split_at(7);
                    Some(s.to_string())
                } else {
                    Some(hash_str)
                }
            },
            None => None,
        }
    }

    pub fn get_current_branch_name(&self) -> Option<String> {
        if self.cache.is_current_branch_set() {
            return self.cache.get_current_branch();
        }
        let mut head: Option<Reference> = None;
        if head.is_none() {
            head = self.get_head();
        }
        let h = match self.resolve_symbolic_reference(head) {
            Some(v) => v,
            None => return None,
        };
        let current_branch_name = self.get_branch_name_for_reference(h);
        self.cache.set_current_branch_name(current_branch_name.clone());
        current_branch_name
    }

    fn get_branch_remote(&self, reference: Reference) -> Option<RefPair> {
        let b = Branch::wrap(reference);
        let upstream = match b.upstream() {
            Ok(u) => u,
            Err(e) => {
                log!(self, "Can't get upstream branch for {:?}: {:?}", b.name(), e);
                return None;
            }
        };
        let branch_name = match upstream.name() {
            Ok(o_n) => match o_n {
                Some(n) => n,
                None => {
                    log!(self, "Invalid branch name");
                    return None;
                },
            },
            Err(e) => {
                log!(self, "Error while getting name for upstream branch: {:?}", e);
                return None;
            }
        };
        let upstream_reference = upstream.get();
        let oid = match upstream_reference.target() {
            Some(o) => o,
            None => {
                log!(self, "Can't get oid of upstream branch");
                return None;
            }
        };
        let remote_name = match upstream_reference.name() {
            Some(n) => {
                let v: Vec<&str> = n.splitn(3, '/').collect();
                if v.len() >= 3 {
                    v[1]
                } else {
                    log!(self, "Can't figure out remote name: {:?}", v);
                    return None;
                }
            },
            None => {
                log!(self, "Can't get full name of upstream branch.");
                return None;
            }
        };
        Some(RefPair{ remote_name: remote_name.to_string(),
                      branch_name: branch_name.to_string(), oid: oid })
    }

    fn get_current_branch_remote_oid(&self) -> Option<RefPair> {
        match self.get_head() {
            Some(r) => self.get_branch_remote(r),
            None => None
        }
    }

    pub fn get_branch_ahead_behind(&self, remote_branch: Option<RemoteBranch>) -> Option<BranchAheadBehind> {
        let current_branch_name = self.get_current_branch_name();
        log!(self, "Current branch name = {:?}", current_branch_name);
        let mut ab = BranchAheadBehind::new(current_branch_name);
        let ref_pair_option = self.get_remote_branch(remote_branch);
        let ref_pair = match ref_pair_option {
            Some(u) => {
                u.clone()
            },
            None => return Some(ab),
        };
        ab.remote_branch_name = Some(ref_pair.branch_name.clone());
        ab.remote_name = Some(ref_pair.remote_name);

        let oid = match self.get_current_branch_oid() {
            Some(r) => r,
            None => return None
        };
        let res = self.repo.graph_ahead_behind(oid, ref_pair.oid);
        match res {
            Ok((a, b)) => {
                ab.ahead = a;
                ab.behind = b;
            },
            Err(e) => {
                log!(self, "Can't get ahead & behind stats for branch {}: {:?}", ref_pair.branch_name, e);
            }
        };
        Some(ab)
    }

    // find remote branch if branch_name is specified
    // if not, get remote tracking branch for current branch
    fn get_remote_branch(&self, remote_branch: Option<RemoteBranch>) -> Option<RefPair> {
        match remote_branch {
            Some(b) => {
                match self.repo.find_branch(&b.remote_branch, BranchType::Remote) {
                    Ok(o) => Some(RefPair{ branch_name: b.remote_branch_name,
                                           remote_name: b.remote_name,
                                           oid: o.into_reference().target().unwrap() }),
                    Err(e) => {
                        // don't panic here - it doesn't exist, we don't care
                        log!(self, "No remote branch found for {}: {:?}", b.remote_branch_name, e);
                        None
                    }
                }
            },
            None => {
                self.get_current_branch_remote_oid()
            }
        }
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

    pub fn get_file_status(&mut self) -> Option<HashMap<String, u32>> {
        if self.cache.is_file_statuses_set() {
            return self.cache.get_file_statuses();
        }
        let mut d = HashMap::new();

        let changed = Status::WT_MODIFIED | Status::WT_DELETED | Status::WT_TYPECHANGE | Status::WT_RENAMED;
        let staged = Status::INDEX_MODIFIED | Status::INDEX_DELETED | Status::INDEX_TYPECHANGE | Status::INDEX_RENAMED | Status::INDEX_NEW;

        let statuses = match self.get_status() {
            Some(x) => x,
            None => return None,
        };

        for s in statuses.iter() {
            let file_status = s.status();
            log!(self, "{}", s.path().unwrap());

            if file_status.intersects(changed) {
                let counter = d.entry(CHANGED_KEY.to_string()).or_insert(0);
                *counter += 1;
            };
            if file_status.contains(Status::WT_NEW) {
                let counter = d.entry(NEW_KEY.to_string()).or_insert(0);
                *counter += 1;
            };
            if file_status.intersects(staged) {
                let counter = d.entry(STAGED_KEY.to_string()).or_insert(0);
                *counter += 1;
            };
            if file_status.intersects(Status::CONFLICTED) {
                let counter = d.entry(CONFLICTS_KEY.to_string()).or_insert(0);
                *counter += 1;
            };
        }
        self.cache.set_file_statuses(Some(d.clone()));
        Some(d)
    }

    pub fn get_stash_count(&mut self) -> u16 {
        let mut count: u16 = 0;
        let r = self.repo.stash_foreach(
            |_u: usize, _s: &str, _o: &Oid| {
                count += 1;
                true
            }
        );
        match r {
            Ok(_) => log!(self, "Stash contains {} items", count),
            Err(e) => log!(self, "There was an error while checking stash: {:?}", e),
        };
        count
    }
}
