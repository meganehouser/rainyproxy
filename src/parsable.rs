use httparse_orig;
use std::str;

pub enum ParseStatus<T> {
    Complete(T),
    InProgress,
    Err(httparse_orig::Error),
}

impl<T> ParseStatus<T> {
    pub fn is_complete(&self) -> bool {
        match *self {
            ParseStatus::Complete(_) => true,
            _ => false,
        }
    }
}

pub trait Parsable {
    fn new() -> Self;
    fn parse(&mut self, buf: &[u8]) -> ParseStatus<usize>;
}

pub trait Sendable {
    fn to_bytes(&self) -> Vec<u8>;
}

pub fn parse_body(headers: &[httparse_orig::Header], buf: &[u8]) -> ParseStatus<Option<Vec<u8>>> {
    match headers.iter().find(|&&h| h.name == "Content-Length") {
        Some(h) => {
            let len_str = str::from_utf8(h.value).unwrap();
            let len: usize = str::FromStr::from_str(len_str).unwrap();

            if len <= buf.len() {
                return ParseStatus::Complete(Some(Vec::from(buf)));
            } else {
                return ParseStatus::InProgress;
            }
        }
        None => return ParseStatus::Complete(None),
    }
}
