# JAZ
Find secrets hidden in commits

# Building
```
~$ cargo build --release
```

# Usage
```
~$ ./jaz /path/to/repo
```

# TODO
- Multithreading
    - Right now it's too slow
- Give better output
    - Tell exactly what commit/file the problems
- Scan remote repos
    - Either use github api orclone onto local machine