# rainyproxy
Customizable local proxy server in Rust. It's work in progress.

    USAGE:
	    rainy [FLAGS] [OPTIONS]
    
    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information
    
    OPTIONS:
        -a, --address <ADDRESS>       listen address (ip address : port no)
        -l, --loglevel <LOG_LEVEL>

# Customize
 
You also can implement your own proxy server.

Cargo.toml

```toml
[dependencies.rainyproxy]
git = "https://github.com/meganehouser/rainyproxy.git"

[dependencies]
hyper = { git = "https://github.com/hyperium/hyper", branch="master"}
```

main.rs

```rust
extern crate hyper;
extern crate rainyproxy;

use rainyproxy::RainyProxy;
use hyper::server::Response;
use hyper::status::StatusCode;

const BODY: &[u8] = b"<html><body>Don't worry. Answer is in your heart.</body></html>\r\n";

fn main() {
    let proxy = RainyProxy::new("127.0.0.1:8800".parse().unwrap());
    proxy.serve_custom(|request| {
                           if uri.contains("q.hatena.ne.jp") {
                               Some(Response::new()
                                        .with_status(StatusCode::Ok)
                                        .with_body(BODY))
                           } else {
                               None
                           }
                       },
                       |_| None);
}
```

# Todo
- [ ] chained proxy (with authentication)
- [ ] support HTTPS
- [ ] implement timeout of read and write
- [ ] add tests, tests, and tests!
