use std::io::Result as IoResult;
use std::io;
use mioco;
use httparse_orig;
use parsable::{Parsable, Sendable};

pub enum ConnResult<T> {
    Ok(T),
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

    pub fn recieve<P: Parsable>(&mut self) -> ConnResult<P> {
        let mut buf_i = 0;
        let mut buf_vec: Vec<u8> = Vec::from(&[0u8; 1024] as &[u8]);
        let mut parsable = P::new();

        loop {
            {
                let mut buf = &mut buf_vec[..];
                let read_result = self.stream.try_read(&mut buf[buf_i..]);

                match read_result {
                    Ok(opt_size) => {
                        match opt_size {
                            None => {}
                            Some(len) => buf_i += len,
                        }
                    }
                    Err(err) => return ConnResult::IoError(err),
                }

                let parse_result = parsable.parse(&buf[0..buf_i]);
                if parse_result.is_complete() {
                    return ConnResult::Ok(parsable);
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
