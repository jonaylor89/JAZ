
use git2::{Repository, BranchType};
// use serde_json::json;
use serde_json;
use std::fs;
use std::collections::HashMap;

const CONFIG_FILE: &str = "rules.json";

fn main() {

    // Get config string
    let conf_str = fs::read_to_string(CONFIG_FILE).unwrap();

    // Make a hashmap of uncompiled regex expressions
    let conf: HashMap<String, String> = serde_json::from_str(&conf_str).unwrap();

    // for (key, val) in &json_map {
    //     println!("{}: \"{}\"", key, val);
    // }


    // Get path to git repo via command line args or assume current directory
    let repo_root = std::env::args().nth(1).unwrap_or(".".to_string());

    // Open git repo
    let repo = Repository::open(repo_root.as_str()).expect("Couldn't open repository");

    println!("[INFO] checking {} key templates", conf.len());

    for branch in repo.branches(Some(BranchType::Local)).unwrap() {

        
        // This is not what rust code should look like
        println!("[INFO] Scanning branch {}", branch.unwrap().0.name().unwrap().unwrap());
    }

    // Print the current start of the git repo
    println!("[INFO] {} state={:?}", repo.path().display(), repo.state());

}
