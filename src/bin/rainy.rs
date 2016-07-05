extern crate chrono;
extern crate clap;
extern crate log;
extern crate env_logger;
extern crate rainyproxy;

use std::env;
use chrono::*;
use clap::{App, Arg};
use log::LogRecord;
use env_logger::LogBuilder;
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

    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:8800");
    let mut builder = init_builder(matches.value_of("loglevel").unwrap_or("info"));
    builder.init().unwrap();

    let proxy = RainyProxy::new(&addr);
    proxy.serve();
}

fn init_builder(level: &str) -> LogBuilder {
    let rainy_level = format!("rainy={}", level);
    let format = |record: &LogRecord| {
        format!("[{}] {} {}",
                UTC::now().to_rfc3339(),
                record.level(),
                record.args())
    };

    let mut builder = LogBuilder::new();
    builder.format(format);

    let log_level = match env::var("RUST_LOG") {
        Ok(ref rust_log) => {
            [rust_log.as_str(), rainy_level.as_str()]
                .join(",")
                .to_string()
        }
        Err(_) => rainy_level,
    };

    builder.parse(&log_level);

    builder
}
