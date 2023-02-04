use git2::{ObjectType, OdbObject, Repository};
use once_cell::sync::Lazy;
use regex::bytes::RegexSet;
use std::ffi::OsString;

const INFO: &str = "\x1b[32m[INFO]\x1b[0m";
const CRITICAL: &str = "\x1b[31m[CRITICAL]\x1b[0m";

fn main() {
    // Get path to git repo via command line args or assume current directory
    let repo_root: OsString = std::env::args_os()
        .nth(1)
        .unwrap_or_else(|| OsString::from("."));

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
    odb.foreach(|&oid| {
        let obj = odb.read(oid).unwrap();

        // Look for secrets in the object
        scan_object(&obj);

        // Return true because the closure has to return a boolean
        true
    })
    .unwrap();
}

/// scan_object : Scan contents of `obj` and print to the console if it contains a secret
fn scan_object(obj: &OdbObject) {
    if obj.kind() != ObjectType::Blob {
        return;
    }
    // Check if the blob contains secrets
    if let Some(secrets_found) = find_secrets(obj.data()) {
        for bad in secrets_found {
            println!(
                "{} object {} has a secret of type `{}`",
                CRITICAL,
                obj.id(),
                bad
            );
        }
    }
}

/// find_secrets : if secrets are found in `blob` then they are returned as a vector, otherwise return None
fn find_secrets(blob: &[u8]) -> Option<Vec<&'static str>> {
    const RULES: &[(&str, &str)] = &[
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
    ];
    static REGEX_SET: Lazy<RegexSet> = Lazy::new(|| {
        RegexSet::new(RULES.iter().map(|&(_, regex)| regex)).expect("All regexes should be valid")
    });

    let matches = REGEX_SET.matches(blob);
    if !matches.matched_any() {
        return None;
    }

    Some(matches.iter().map(|i| RULES[i].0).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_nothing() {
        let secret: &[u8] = "Nothing to see here".as_bytes();
        let result: Option<Vec<&'static str>> = find_secrets(secret);

        assert!(result.is_none());
    }

    #[test]
    fn find_ssh_key() {
        let secret: &[u8] = "-----BEGIN OPENSSH PRIVATE KEY-----".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(
            result.get(0).expect("Should contain one secret type"),
            &"SSH (OPENSSH) private key"
        );
    }
}
