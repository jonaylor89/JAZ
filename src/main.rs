use git2::{ObjectType, Repository};
use regex::Regex;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::str::from_utf8;
use termion::color::{self, Fg};

const CONFIG_FILE: &str = "rules.json";

macro_rules! info { 
    () => { format!("{}[INFO]{}", Fg(color::Green), Fg(color::Reset)) }; 
} 

macro_rules! critical { 
    () => { format!("{}[CRITICAL]{}", Fg(color::Red), Fg(color::Reset)) }; 
} 

fn main() {
    // Get config string
    let conf_str = fs::read_to_string(CONFIG_FILE).unwrap();

    // Make a hashmap of uncompiled regex expressions
    let conf: HashMap<String, String> = serde_json::from_str(&conf_str).unwrap();

    // Get path to git repo via command line args or assume current directory
    let repo_root: String = std::env::args().nth(1).unwrap_or(".".to_string());

    // Open git repo
    let repo: git2::Repository = Repository::open(repo_root.as_str()).expect("Couldn't open repository");

    println!(
        "{} {} state={:?}",
        info!(),
        repo.path().display(),
        repo.state()
    );
    println!("{} checking {} key templates", info!(), conf.len());
    println!("--------------------------------------------------------------------------");

    // Get object database from the repo
    let odb = repo.odb().unwrap();
    let mut children = vec![];

    // Loop through objects in db 
    odb.foreach(|oid| {
        let object_id = oid.clone();
        let config = conf.clone();
        let repository = Repository::open(repo_root.as_str()).expect("Couldn't open repository");

        // Spawn a thread to look for secrets in the object
        children.push(std::thread::spawn( move || scan_object(repository, &object_id, config)));
        true
    })
    .unwrap();

    let num_children = &children.len();

    for child in children {
        let _ = child.join();
    }

    println!("{} Spawned {} threads", info!(), num_children);
}

fn scan_object(repo:git2::Repository, oid:&git2::Oid, conf: HashMap<String, String>){

    // Get the object from the oid
    let obj = repo.revparse_single(&oid.to_string()).unwrap();
        // println!("{} {}\n--", obj.kind().unwrap().str(), obj.id());
        match obj.kind() {

            // Only grab objects associated with blobs
            Some(ObjectType::Blob) => {
                let blob_str = match from_utf8(obj.as_blob().unwrap().content()) {
                    Ok(x)=>x,
                    Err(_)=>return,
                };
                // println!("{}",blob_str);

                // Check if the blob contains secrets
                match is_bad(blob_str, &conf) {
                    Some(bad_commits) => {
                            for bad in bad_commits {
                                println!("{} commit {} has a secret of type `{}`", critical!(), oid, bad);
                            }
                        },
                    // None => println!("{} oid {} is {}", INFO, oid, "safe".to_string()),
                    None => (),
                }
            }
            _ => (), // only care about the blobs so ignore anything else.
        }
}
// is_bad : if secrets are found in blob then they are returned as a vector, otherwise return None
fn is_bad(maybe: &str, bads: &HashMap<String, String>) -> Option<Vec<String>> {
    let mut bad_commits = vec![];
    for (key, val) in bads {

        // Use regex from rules file to match against blob
        let re = Regex::new(val).unwrap();
        if re.is_match(maybe) {
            bad_commits.push(key.to_string());
        }
    }
    if bad_commits.len() > 0 {
        // Return bad commits if there are any
        return Some(bad_commits);
    }
    None
}
