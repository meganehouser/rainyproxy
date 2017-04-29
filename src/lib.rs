#![feature(custom_derive)]

extern crate tokio_core;
extern crate tokio_io;
extern crate futures;
extern crate hyper_tls;
extern crate hyper;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate regex;

mod rainyproxy;
pub use rainyproxy::RainyProxy;

mod proxy;
