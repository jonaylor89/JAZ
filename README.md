
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

To scan we open the object database and for every object in the database we spawn a thread and check its contents. In each object we scan through and look for regex patterns of common keys provided by an array.  In the future we'd like this array to be configurable and for there to be a pool of threads to distribute work across as opposed to spawning a thread for each operation as we do now.

### Improvements  

The following is a list of improvements that would be good to add for the future.  In general they make JAZ better or easier to use.

- Threadpool  
- Config file based scanning  
- remote scanning  
- better CI/CD  
- automated GitHub repo scanning  

### Installation

From Source
```
~$ cargo build
```
This will build into the target directory under debug by default and under release if the project is built with `cargo build --release`

Arch
```
~$ yay -S jaz
```

MacOS
```
~$ brew install jaz
```

Cargo
```
~$ cargo install jaz
```

Execution
```
~$ ./jaz /path/to/repo
```

### Results

We scanned common testing repositories for this sort of thing like [Plazmaz/leaky-repo](https://github.com/Plazmaz/leaky-repo) and [dijininja/leakyrepo](https://github.com/digininja/leakyrepo).  In general JAZ found all or most of the secrets.  In the case of dijininja/leakyrepo we found a lot of RSA private keys which is acceptable but technically is a misidentification.  For Plazmaz/leaky-repo we find the majority of the keys although once again misidentify some.

