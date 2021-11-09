use git2::{Object, ObjectType, Oid, Repository};
use regex::Regex;
use std::collections::HashMap;

const INFO: &str = "\x1b[32m[INFO]\x1b[0m";
const CRITICAL: &str = "\x1b[31m[CRITICAL]\x1b[0m";

fn main() {
    // Get path to git repo via command line args or assume current directory
    let repo_root: String = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());

    // Open git repo
    let repo = Repository::open(&repo_root).expect("Couldn't open repository");

    println!(
        "{} {} state={:?}",
        INFO,
        repo.path().display(),
        repo.state()
    );
    println!("--------------------------------------------------------------------------");

    // Get object database from the repo
    let odb = repo.odb().unwrap();

    // Loop through objects in db
    odb.foreach(|oid| {
        let obj = repo.revparse_single(&oid.to_string()).unwrap();

        // Look for secrets in the object
        scan_object(&obj, oid);

        // Return true because the closure has to return a boolean
        true
    })
    .unwrap();
}

fn scan_object(obj: &Object, oid: &Oid) {
    if let Some(ObjectType::Blob) = obj.kind() {
        let blob_str = match std::str::from_utf8(obj.as_blob().unwrap().content()) {
            Ok(x) => x,
            Err(_) => return,
        };
        // println!("{}",blob_str);

        // Check if the blob contains secrets
        if let Some(secrets_found) = find_secrets(blob_str) {
            for bad in secrets_found {
                println!("{} object {} has a secret of type `{}`", CRITICAL, oid, bad);
            }
        }
    }
}

// find_secrets : if secrets are found in blob then they are returned as a vector, otherwise return None
fn find_secrets(blob: &str) -> Option<Vec<String>> {
    let rules = HashMap::from([
        ("Slack Token", "(xox[p|b|o|a]-[0-9]{12}-[0-9]{12}-[0-9]{12}-[a-z0-9]{32})"),
        ("RSA private key", "-----BEGIN RSA PRIVATE KEY-----"),
        ("SSH (OPENSSH) private key", "-----BEGIN OPENSSH PRIVATE KEY-----"),
        ("SSH (DSA) private key", "-----BEGIN DSA PRIVATE KEY-----"),
        ("SSH (EC) private key", "-----BEGIN EC PRIVATE KEY-----"),
        ("PGP private key block", "-----BEGIN PGP PRIVATE KEY BLOCK-----"),
        ("Facebook Oauth", "[f|F][a|A][c|C][e|E][b|B][o|O][o|O][k|K].{0,30}['\"\\s][0-9a-f]{32}['\"\\s]"),
        ("Twitter Oauth", "[t|T][w|W][i|I][t|T][t|T][e|E][r|R].{0,30}['\"\\s][0-9a-zA-Z]{35,44}['\"\\s]"),
        ("GitHub", "[g|G][i|I][t|T][h|H][u|U][b|B].{0,30}['\"\\s][0-9a-zA-Z]{35,40}['\"\\s]"),
        ("Google Oauth", "(\"client_secret\":\"[a-zA-Z0-9-_]{24}\")"),
        ("AWS API Key", "AKIA[0-9A-Z]{16}"),
        ("Heroku API Key", "[h|H][e|E][r|R][o|O][k|K][u|U].{0,30}[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}"),
        ("Generic Secret", "[s|S][e|E][c|C][r|R][e|E][t|T].{0,30}['\"\\s][0-9a-zA-Z]{32,45}['\"\\s]"),
        ("Generic API Key", "[a|A][p|P][i|I][_]?[k|K][e|E][y|Y].{0,30}['\"\\s][0-9a-zA-Z]{32,45}['\"\\s]"),
        ("Slack Webhook", "https://hooks.slack.com/services/T[a-zA-Z0-9_]{8}/B[a-zA-Z0-9_]{8}/[a-zA-Z0-9_]{24}"),
        ("Google (GCP) Service-account", "\"type\": \"service_account\""),
        ("Twilio API Key", "SK[a-z0-9]{32}"),
        ("Password in URL", "[a-zA-Z]{3,10}://[^/\\s:@]{3,20}:[^/\\s:@]{3,20}@.{1,100}[\"'\\s]"),
    ]);

    let mut secrets_found = vec![];
    for (key, val) in rules {
        // Use regex from rules file to match against blob
        let re = Regex::new(val).unwrap();
        if re.is_match(blob) {
            secrets_found.push(key.to_string());
        }
    }

    if !secrets_found.is_empty() {
        // Return bad commits if there are any
        return Some(secrets_found);
    }
    None
}
