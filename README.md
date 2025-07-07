
# JAZ - Git Repo Secret Scanning  

blog post:
[https://blog.jonaylor.xyz/discover-hidden-secrets-in-git-repos-with-rust](https://blog.jonaylor.xyz/discover-hidden-secrets-in-git-repos-with-rust)

### Intro

With the growing importance of open source software and increasing usage of public code repositories like GitHub, it has become increasingly important to protect against accidental secret commiting. While it may seem that doing this is as easy as deleting a key file or removing an oauth token from a configuration file, the bittersweet fact about git is that it'll keep a history of that secret. By searching through the git commit logs, an attacker could find and exploit application secrets. This project set out to create an automated way to find secrets hidden in the git commit history.

### Secrets Scanning

As of now we are scanning for the following secrets based off common regex patterns identifying them.  More can easily be added once a regex pattern is developed to identify them.

1. Slack Token  
2. RSA private key  
3. (OPENSSH) private key  
4. SSH (DSA) private key  
5. SSH (EC) private key  
6. PGP private key block  
7. Facebook Oauth  
8. Twitter Oauth  
9. GitHub  
10. Google Oauth  
11. AWS API Key  
12. Heroku API Key  
13. Generic Secret  
14. Generic API Key  
15. Slack Webhook  
16. Google (GCP) Service-account  
17. Twilio API Key  
18. Password in URL
 
### Design

In each object contained in the object database, we scan through and look for regex patterns of common keys provided by an array. If any secrets are found, the script simply prints the secret type to the console and provides the object id.

### Improvements  

The following is a list of improvements that would be good to add for the future.  In general they make JAZ better or easier to use.

- Threadpool  
- Config file based scanning  
- remote scanning  
- better CI/CD  
- automated GitHub repo scanning  

### Installation

**From Source**
```bash
~$ git clone https://github.com/jonaylor89/JAZ.git
~$ cd JAZ
~$ cargo build --release
```
This will build into the `target/release` directory. For debug builds, use `cargo build`.

**Arch Linux**
```bash
~$ yay -S jaz
```

**MacOS**
```bash
~$ brew install jaz
```

**Cargo**
```bash
~$ cargo install jaz
```

### Usage

**Basic Usage**
```bash
# Scan current directory (must be a git repository)
~$ jaz

# Scan a specific repository
~$ jaz /path/to/repo

# Scan with compiled binary
~$ ./target/release/jaz /path/to/repo
```

**Example Output**
```
[INFO] /path/to/repo/.git/ state=Clean
--------------------------------------------------------------------------
[CRITICAL] object a1b2c3d4e5f6 has a secret of type `AWS API Key`
[CRITICAL] object f6e5d4c3b2a1 has a secret of type `SSH (OPENSSH) private key`
[CRITICAL] object 1234567890ab has a secret of type `Slack Token`
```

**Common Use Cases**

1. **Pre-commit Hook**: Add JAZ to your git hooks to scan before commits
   ```bash
   # .git/hooks/pre-commit
   #!/bin/bash
   jaz . && echo "No secrets found" || (echo "Secrets detected!" && exit 1)
   ```

2. **CI/CD Pipeline**: Integrate into your build process
   ```yaml
   # GitHub Actions example
   - name: Scan for secrets
     run: |
       cargo install jaz
       jaz .
   ```

3. **Audit Existing Repositories**: Scan repositories you've inherited
   ```bash
   # Scan multiple repos
   for repo in ~/projects/*/; do
     echo "Scanning $repo"
     jaz "$repo"
   done
   ```

**Exit Codes**
- `0`: No secrets found
- `1`: Secrets detected or error occurred

### Understanding the Output

JAZ scans all git objects in the repository's object database, not just the current working directory. This means it will find secrets in:
- All commits in the repository history
- All branches (local and remote-tracking)
- Staged and unstaged changes
- Deleted files that were previously committed

**Object ID Information**
When JAZ finds a secret, it reports the git object ID (SHA-1 hash). You can investigate further with:
```bash
# View the object content
git show <object-id>

# Find which commits contain this object
git log --all --full-history -- $(git rev-list --all | xargs git ls-tree -r | grep <object-id> | cut -f2)
```

**Common Remediation Steps**
1. **For recent commits**: Use `git reset` or `git rebase` to remove the secret
2. **For historical commits**: Use `git filter-branch` or `git filter-repo` to rewrite history
3. **For shared repositories**: Coordinate with your team before rewriting history
4. **Always**: Rotate/invalidate the exposed secret immediately

### Troubleshooting

**Repository Not Found**
```
Error: Couldn't open repository
```
- Ensure the path points to a valid git repository
- Check that `.git` directory exists
- Verify you have read permissions

**Permission Denied**
```
Error: Permission denied
```
- Run with appropriate permissions
- Check repository ownership and permissions

**Large Repository Performance**
For very large repositories, JAZ may take some time to scan all objects. Consider:
- Using `--depth=1` when cloning if you only need recent history
- Running JAZ on a dedicated machine for large-scale scanning

### Testing

Run the comprehensive test suite:
```bash
cargo test
```

This includes tests for all 18 secret types and various edge cases.

### Results

We scanned common testing repositories for this sort of thing like [Plazmaz/leaky-repo](https://github.com/Plazmaz/leaky-repo) and [dijininja/leakyrepo](https://github.com/digininja/leakyrepo).  In general JAZ found all or most of the secrets.  In the case of dijininja/leakyrepo we found a lot of RSA private keys which is acceptable but technically is a misidentification.  For Plazmaz/leaky-repo we find the majority of the keys although once again misidentify some.

