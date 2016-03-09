#![feature(custom_derive, lookup_host, sockaddr_setters)]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate mioco;
extern crate httparse as httparse_orig;

mod rainyproxy;
pub use rainyproxy::RainyProxy;

mod request;
mod response;
mod parsable;
mod connection;

pub mod httparse {
    pub use httparse_orig::Error;
}