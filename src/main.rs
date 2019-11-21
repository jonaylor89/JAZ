
use git2::{Repository, BranchType};

fn main() {

    // Get path to git repo via command line args or assume current directory
    let repo_root = std::env::args().nth(1).unwrap_or(".".to_string());

    // Open git repo
    let repo = Repository::open(repo_root.as_str()).expect("Couldn't open repository");

    for branch in repo.branches(Some(BranchType::Local)).unwrap() {

        // This is not what rust code should look like
        println!("[INFO] Scanning branch {}", branch.unwrap().0.name().unwrap().unwrap());
    }

    // Print the current start of the git repo
    println!("[INFO] {} state={:?}", repo.path().display(), repo.state());

}
