extern crate git2;

use git2::{Repository,Branch,BranchType,Oid,Reference};


struct Program {
    repo: Repository,
}

fn get_branch_remote(reference: Reference) -> Oid {
    let b = Branch::wrap(reference);
    let upstream = match b.upstream() {
        Ok(u) => u,
        Err(e) => panic!("Failed to get upstream: {}.", e)
    };
    match upstream.get().target() {
        Some(o) => o,
        None => panic!("Failed to get oid of branch.")
    }
}

impl Program {
    fn get_head(&self) -> Reference {
        match self.repo.head() {
            Ok(head) => head,
            Err(e) => panic!("Failed to get head: {}.", e)
        }
    }

    fn get_current_branch_oid(&self) -> Oid {
        let head = self.get_head();
        match head.target() {
            Some(v) => v,
            None => panic!("Failed to get reference to head."),
        }
    }

    fn get_current_branch_name(&self) -> String {
        match self.get_head().name() {
            Some(v) => v.to_string(),
            None => panic!("Failed to get name of head."),
        }
    }

    // fn get_current_branch_remote(&self) -> String {
    //     let oid = self.get_current_branch_oid();
    //     let b = Branch::wrap(oid);
    //     let upstream = b.upstream().ok().unwrap();
    //     String::from(upstream.name().ok().unwrap().unwrap())
    // }

    fn get_current_branch_remote_oid(&self) -> Oid {
        get_branch_remote(self.get_head())
    }

    fn get_current_branch_ahead_behind(&self) -> (usize, usize)  {
        let res = self.repo.graph_ahead_behind(
            self.get_current_branch_oid(),
            self.get_current_branch_remote_oid(),
        );
        match res {
            Ok(r) => r,
            Err(e) => panic!("failed to compute ahead/behind: {}", e)
        }
    }
}


fn main() {
    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    let program = Program{ repo: repo };
    println!("{}", program.get_current_branch_name());
    let (ahead, behind) = program.get_current_branch_ahead_behind();
    println!("A: {}, B: {}", ahead, behind);
    // let current_branch_name = get_current_branch_name(repo);
    // println!("{:?}", current_branch_name);
    // let us = match repo.find_branch("origin/master", BranchType::Remote) {
    //     Ok(us) => us,
    //     Err(e) => panic!("failed to find origin/master: {}", e)
    // };
}
