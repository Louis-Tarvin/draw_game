# Draw Game Server

## Running

Make sure you have Rust installed, then run `cargo run`

Optional command-line arguments:
```
    -l <log>
        The path to the directory of log files.
    -p <port>
        The port for the server. Defaults to 3007.
    -d <serve_dir>
        Mount this directory at the root level of the server. Serves 'index.html' at '/'.
    -w <word_pack_dir>
        The path to the word pack directory. Defaults to 'wordpacks'
