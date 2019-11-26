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
    let repo_root:String = std::env::args().nth(1).unwrap_or(".".to_string());

    // Open git repo
    let repo:git2::Repository = Repository::open(repo_root.as_str()).expect("Couldn't open repository");

    println!(
        "{} {} state={:?}",
        info!(),
        repo.path().display(),
        repo.state()
    );
    println!("{} checking {} key templates", info!(), conf.len());
    println!("--------------------------------------------------------------------------");

    let odb = repo.odb().unwrap();
    let mut children = vec![];
    odb.foreach(|oid| {
        let object_id:git2::Oid = oid.clone();
        let config :HashMap<String, String>= conf.clone();
        let repository:git2::Repository = Repository::open(repo_root.as_str()).expect("Couldn't open repository");
        children.push(std::thread::spawn( move || scan_object(repository, &object_id, config)));
        true
    })
    .unwrap();


    for child in children {
        let _ = child.join();
    }
}

fn scan_object(repo:git2::Repository, oid:&git2::Oid, conf: HashMap<String, String>){
    let obj = repo.revparse_single(&oid.to_string()).unwrap();
        // println!("{} {}\n--", obj.kind().unwrap().str(), obj.id());
        match obj.kind() {
            Some(ObjectType::Blob) => {
                let blob_str = match from_utf8(obj.as_blob().unwrap().content()) {
                    Ok(x)=>x,
                    Err(_)=>return,
                };
                // println!("{}",blob_str);
                match is_bad(blob_str, &conf) {
                    Some(x) => println!("{} commit {} has a secret of type `{}`", critical!(), oid, x),
                    // None => println!("{} oid {} is {}", INFO, oid, "safe".to_string()),
                    None => (),
                }
            }
            _ => (), // only care about the blobs so ignore anything else.
        }
}
// is_bad : if secret found it's type is returned, otherwise return None
fn is_bad(maybe: &str, bads: &HashMap<String, String>) -> Option<String> {
    for (key, val) in bads {
        let re = Regex::new(val).unwrap();
        if re.is_match(maybe) {
            return Some(key.to_string());
        }
    }
    None
}
