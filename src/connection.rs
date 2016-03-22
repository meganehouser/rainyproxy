use std::io;
use std::net::lookup_host;
use mioco;
use httparse_orig;
use parsable::{Parsable, Sendable, ParseStatus};

pub enum ConnResult<T> {
    Ok(T),
    ZeroPacket,
    ParseErr(httparse_orig::Error),
    IoError(io::Error),
}

pub struct Connection {
    stream: mioco::tcp::TcpStream,
}

impl Connection {
    pub fn new(stream: mioco::tcp::TcpStream) -> Connection {
        Connection { stream: stream }
    }

    pub fn from(host: &str, port: &u16) -> Option<Connection> {
        let lookupd = match lookup_host(host) {
            Ok(l) => l,
            Err(_) => return None,
        };

        for result in lookupd {
            let mut addr = match result {
                Ok(a) => a,
                Err(e) => return None,
            };

            addr.set_port(*port);
            match mioco::tcp::TcpStream::connect(&addr) {
                Ok(c) => return Some(Connection::new(c)),
                Err(e) => continue,
            };
        }
        None
    }

    pub fn recieve<P: Parsable>(&mut self) -> ConnResult<P::Parsed> {
        let mut buf_i = 0;
        let mut buf_vec: Vec<u8> = Vec::from(&[0u8; 1024] as &[u8]);

        loop {
            {
                let mut buf = &mut buf_vec[..];
                let read_result = self.stream.try_read(&mut buf[buf_i..]);
                match read_result {
                    Ok(opt_size) => {
                        match opt_size {
                            None => {}
                            Some(len) => {
                                if len == 0 {
                                    return ConnResult::ZeroPacket;
                                } else {
                                    buf_i += len
                                }
                            }
                        }
                    }
                    Err(err) => return ConnResult::IoError(err),
                }

                let parse_result = P::parse(&buf[0..buf_i]);
                match parse_result {
                    ParseStatus::Complete(parsed) => return ConnResult::Ok(parsed),
                    ParseStatus::InProgress => {}
                    ParseStatus::Err(err) => return ConnResult::ParseErr(err),
                }
            }

            if buf_i == buf_vec.len() {
                buf_vec.extend_from_slice(&[0u8; 1024] as &[u8]);
            }
        }
    }

    pub fn send<S: Sendable>(&mut self, o: &S) -> ConnResult<()> {
        let bytes = o.to_bytes();
        let len = bytes.len();
        let mut send_len = 0;

        while send_len < len {
            let result = self.stream.try_write(&bytes[send_len..]);
            match result {
                Ok(option_size) => {
                    match option_size {
                        Some(size) => send_len += size,
                        None => continue,
                    };
                }
                Err(e) => return ConnResult::IoError(e),
            };
        }

        ConnResult::Ok(())
    }
}
