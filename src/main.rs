extern crate git2;

use git2::Repository;
use git2::BranchType;


// fn get_current_branch_name(repo: Repository) -> String {
//     let head = match repo.head() {
//         Ok(head) => head,
//         Err(e) => panic!("failed to get head: {}", e)
//     };
//     return head.name().unwrap().to_string();
// }



fn main() {
    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    // let current_branch_name = get_current_branch_name(repo);
    // println!("{:?}", current_branch_name);
    let head = match repo.head() {
        Ok(head) => head,
        Err(e) => panic!("failed to get head: {}", e)
    };
    let oid = head.target().unwrap();
    let us = match repo.find_branch("origin/master", BranchType::Remote) {
        Ok(us) => us,
        Err(e) => panic!("failed to find origin/master: {}", e)
    };
    repo.graph_ahead_behind(oid, us.get().target().unwrap());
}
