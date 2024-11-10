# Curl

### GET
```shell
cargo run -- "URL"
```
### POST
```shell
// Use string that contains one or multiple key-value pairs separated by &
cargo run -- "URL" -X POST -d "K=Key&V=Value"

// Use Json format
cargo run -- "URL" --json '{"Key" : "hello", "Value" : "world"}'
```

### Arguments
```rust
struct CurlArgs {
    #[structopt(name = "url")]
    url: String,

    /// HTTP method to use (GET, POST, etc.)
    #[structopt(short = "X", long = "request", default_value = "GET")]
    method: String,

    /// Data to send with POST request in the form 'key1=value1&key2=value2'
    #[structopt(short = "d", long = "data")]
    data: Option<String>,

    /// JSON data to send with POST request
    #[structopt(long = "json")]
    json: Option<String>,
}
```