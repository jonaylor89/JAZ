use git2::{ObjectType, Object, Oid, Repository};
use regex::Regex;
use std::collections::HashMap;

// Macros for logging
macro_rules! info {
    () => {
        format!("{}[INFO]{}", "\x1B[32m", "\x1B[0m")
    };
}

macro_rules! critical {
    () => {
        format!("{}[CRITICAL]{}", "\x1B[31m", "\x1B[0m")
    };
}

fn main() {
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

    // Get path to git repo via command line args or assume current directory
    let repo_root: String = std::env::args().nth(1).unwrap_or(".".to_string());

    // Open git repo
    let repo = Repository::open(repo_root.as_str()).expect("Couldn't open repository");

    println!(
        "{} {} state={:?}",
        info!(),
        repo.path().display(),
        repo.state()
    );
    println!("{} checking {} key templates", info!(), rules.len());
    println!("--------------------------------------------------------------------------");

    // Get object database from the repo
    let odb = repo.odb().unwrap();

    // Loop through objects in db
    odb.foreach(|oid| {

        let config = rules.clone();
        let obj = repo.revparse_single(&oid.to_string()).unwrap();

        // Look for secrets in the object
        scan_object(&obj, oid, config);

        // Return true because the closure has to return a boolean
        true
    })
    .unwrap();
}

fn scan_object(obj: &Object, oid: &Oid, conf: HashMap<&str, &str>) {
    
    match obj.kind() {
        // Only grab objects associated with blobs
        Some(ObjectType::Blob) => {
            let blob_str = match std::str::from_utf8(obj.as_blob().unwrap().content()) {
                Ok(x) => x,
                Err(_) => return,
            };
            // println!("{}",blob_str);

            // Check if the blob contains secrets
            match is_bad(blob_str, &conf) {
                Some(bad_commits) => {
                    for bad in bad_commits {
                        println!(
                            "{} object {} has a secret of type `{}`",
                            critical!(),
                            oid,
                            bad
                        );
                    }
                }
                // None => println!("{} oid {} is {}", INFO, oid, "safe".to_string()),
                None => (),
            }
        }
        _ => (), // only care about the blobs so ignore anything else.
    }
}
// is_bad : if secrets are found in blob then they are returned as a vector, otherwise return None
fn is_bad(maybe: &str, bads: &HashMap<&str, &str>) -> Option<Vec<String>> {
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
