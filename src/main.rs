
use git2::Repository;

fn main() {


    let url = "https://github.com/alexcrichton/git2-rs";
    let repo = match Repository::clone(url, "/Users/johannes/Repos/git2") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to init: {}", e),
    };



}
