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

    [dependencies.rainyproxy]
    git = "https://github.com/meganehouser/rainyproxy.git"


main.rs

    extern crate rainyproxy;
    
    use rainyproxy::RainyProxy;
    use rainyproxy::Request;
    use rainyproxy::Response;
    use rainyproxy::Parsable;
    
    fn main() {
        let proxy = RainyProxy::new(&"localhost:8800");
        proxy.serve_custom(|req| {
            let response = if req.path.starts_with("http://q.hatena.ne.jp") {
                let body = b"<html><body>Don't worry. Answer is in your heart.</body></html>\r\n";
                let body_len = body.len();
                let mut res = Response::new();
                res.status_code = 200;
                res.reason = String::from("OK");
                res.headers.insert(String::from("Content-Type"), Vec::from(b"text/html" as &[u8]));
                res.headers.insert(String::from("Content-Length"),
                Vec::from(body_len.to_string().as_str()));
                res.body = Some(Vec::from(body as &[u8]));
                Some(res)
            }else {
                None
            };
    
            response
        },
        |&mut _| {});
    }

# Todo
- [ ] chained proxy (with authentication)
- [ ] support HTTPS
- [ ] support persistent connection
- [ ] implement timeout of read and write
