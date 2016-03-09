extern crate clap;
extern crate log;
extern crate env_logger;
extern crate rainyproxy;

use std::env;
use clap::{App, Arg};
use rainyproxy::RainyProxy;

fn main() {
    let matches = App::new("rainy")
                      .version("0.0.1")
                      .author("meganehouser")
                      .arg(Arg::with_name("addr")
                               .short("a")
                               .long("address")
                               .value_name("ADDRESS")
                               .help("listen address (ip address : port no)")
                               .required(false))
                      .arg(Arg::with_name("loglevel")
                               .short("l")
                               .long("loglevel")
                               .value_name("LOG_LEVEL")
                               .required(false))
                      .get_matches();

    let addr = matches.value_of("address").unwrap_or("127.0.0.1:8800");
    set_log_level(matches.value_of("loglevel").unwrap_or("info"));
    env_logger::init().unwrap();

    let proxy = RainyProxy::new(&addr);
    proxy.serve();
}

fn set_log_level(level: &str) {
    let rainy_level = format!("rainy={}", level);

    let log_level = match env::var("RUST_LOG") {
        Ok(ref rust_log) => {
            [rust_log.as_str(), rainy_level.as_str()]
                .join(",")
                .to_string()
        }
        Err(_) => rainy_level,
    };

    env::set_var("RUST_LOG", &log_level);
}
