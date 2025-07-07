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
    use git2::{ObjectType, Repository};
    use std::fs;

    #[test]
    fn find_nothing() {
        let secret: &[u8] = "Nothing to see here".as_bytes();
        let result: Option<Vec<&'static str>> = find_secrets(secret);

        assert!(result.is_none());
    }

    #[test]
    fn find_ssh_openssh_key() {
        let secret: &[u8] = "-----BEGIN OPENSSH PRIVATE KEY-----".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "SSH (OPENSSH) private key");
    }

    #[test]
    fn find_rsa_private_key() {
        let secret: &[u8] = "-----BEGIN RSA PRIVATE KEY-----".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "RSA private key");
    }

    #[test]
    fn find_ssh_dsa_key() {
        let secret: &[u8] = "-----BEGIN DSA PRIVATE KEY-----".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "SSH (DSA) private key");
    }

    #[test]
    fn find_ssh_ec_key() {
        let secret: &[u8] = "-----BEGIN EC PRIVATE KEY-----".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "SSH (EC) private key");
    }

    #[test]
    fn find_pgp_private_key() {
        let secret: &[u8] = "-----BEGIN PGP PRIVATE KEY BLOCK-----".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "PGP private key block");
    }

    #[test]
    fn find_aws_api_key() {
        let secret: &[u8] = "AKIAIOSFODNN7EXAMPLE".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "AWS API Key");
    }

    #[test]
    fn find_github_token() {
        let secret: &[u8] = "github \"1234567890abcdef1234567890abcdef12345\" ".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "GitHub");
    }

    #[test]
    fn find_google_oauth() {
        let secret: &[u8] = "\"client_secret\":\"abcdef1234567890abcdef12\"".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Google Oauth");
    }

    #[test]
    fn find_heroku_api_key() {
        let secret: &[u8] = "heroku 12345678-1234-1234-1234-123456789012".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Heroku API Key");
    }

    #[test]
    fn find_generic_secret() {
        let secret: &[u8] = "secret \"abcdef1234567890abcdef1234567890abcdef12\"".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Generic Secret");
    }

    #[test]
    fn find_generic_api_key() {
        let secret: &[u8] = "api_key \"abcdef1234567890abcdef1234567890abcdef12\"".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Generic API Key");
    }

    #[test]
    fn find_gcp_service_account() {
        let secret: &[u8] = "\"type\": \"service_account\"".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Google (GCP) Service-account");
    }

    #[test]
    fn find_password_in_url() {
        let secret: &[u8] = "https://user:password@example.com/path \"".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Password in URL");
    }

    #[test]
    fn find_facebook_oauth() {
        let secret: &[u8] = "facebook \"abcdef1234567890abcdef1234567890\"".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Facebook Oauth");
    }

    #[test]
    fn find_twitter_oauth() {
        let secret: &[u8] = "twitter \"abcdef1234567890abcdef1234567890abcdef12\"".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Twitter Oauth");
    }

    #[test]
    fn find_multiple_secrets() {
        let secret: &[u8] = "-----BEGIN RSA PRIVATE KEY-----\nAKIAIOSFODNN7EXAMPLE\n".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find secrets");

        assert_eq!(result.len(), 2);
        assert!(result.contains(&"RSA private key"));
        assert!(result.contains(&"AWS API Key"));
    }

    #[test]
    fn find_secrets_case_insensitive() {
        let secret: &[u8] = "SECRET \"abcdef1234567890abcdef1234567890abcdef12\"".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Generic Secret");
    }

    #[test]
    fn find_secrets_with_whitespace() {
        let secret: &[u8] = "api_key  \"abcdef1234567890abcdef1234567890abcdef12\"".as_bytes();
        let result: Vec<&'static str> = find_secrets(secret).expect("Should find a secret");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Generic API Key");
    }

    #[test]
    fn empty_blob() {
        let secret: &[u8] = "".as_bytes();
        let result: Option<Vec<&'static str>> = find_secrets(secret);

        assert!(result.is_none());
    }

    #[test]
    fn binary_data() {
        let secret: &[u8] = &[0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD];
        let result: Option<Vec<&'static str>> = find_secrets(secret);

        assert!(result.is_none());
    }

    #[test]
    fn very_long_input() {
        let long_string = "x".repeat(10000);
        let secret: &[u8] = long_string.as_bytes();
        let result: Option<Vec<&'static str>> = find_secrets(secret);

        assert!(result.is_none());
    }

    #[test]
    fn scan_object_non_blob() {
        let repo = Repository::init_bare("target/test_repo").unwrap();
        let odb = repo.odb().unwrap();
        
        let tree_id = {
            let tree_builder = repo.treebuilder(None).unwrap();
            tree_builder.write().unwrap()
        };
        
        if let Ok(obj) = odb.read(tree_id) {
            assert_eq!(obj.kind(), ObjectType::Tree);
        }
        
        fs::remove_dir_all("target/test_repo").ok();
    }

    #[test]
    fn scan_object_with_secret() {
        let repo = Repository::init_bare("target/test_repo2").unwrap();
        let odb = repo.odb().unwrap();
        
        let secret_content = "AKIAIOSFODNN7EXAMPLE";
        let blob_id = odb.write(ObjectType::Blob, secret_content.as_bytes()).unwrap();
        let obj = odb.read(blob_id).unwrap();
        
        assert_eq!(obj.kind(), ObjectType::Blob);
        assert!(find_secrets(obj.data()).is_some());
        
        fs::remove_dir_all("target/test_repo2").ok();
    }

    #[test]
    fn scan_object_without_secret() {
        let repo = Repository::init_bare("target/test_repo3").unwrap();
        let odb = repo.odb().unwrap();
        
        let normal_content = "This is just normal text content";
        let blob_id = odb.write(ObjectType::Blob, normal_content.as_bytes()).unwrap();
        let obj = odb.read(blob_id).unwrap();
        
        assert_eq!(obj.kind(), ObjectType::Blob);
        assert!(find_secrets(obj.data()).is_none());
        
        fs::remove_dir_all("target/test_repo3").ok();
    }
}
