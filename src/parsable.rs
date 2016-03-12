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
        None => {
            match headers.iter().find(|&&h| h.name == "Trunsfer Encoding") {
                Some(h) => {
                    if h.value == b"chunked" {
                        return parse_chunked(buf);
                    }
                }
                None => {}
            };
        }
    }
    ParseStatus::Complete(None)
}

fn parse_chunked(buf: &[u8]) -> ParseStatus<Option<Vec<u8>>> {
    let mut vec: Vec<u8> = Vec::new();
    let mut cur_num = Vec::new();
    let mut index = 3;
    loop {
        if buf.len() < index {
            return ParseStatus::InProgress;
        }

        cur_num.push(buf[index - 2]);

        if buf[index - 1] == b'\r' && buf[index] == b'\n' {
            let num = str::FromStr::from_str(String::from_utf8(cur_num).unwrap().as_str()).unwrap();
            cur_num = Vec::new();
            if num == 0 {
                return ParseStatus::Complete(Some(Vec::from(buf)));
            }

            index += num;
        } else {
            index += 1;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{parse_chunked, ParseStatus};

    #[test]
    fn chunked_test() {
        let src = b"10\r\nabcdefghij\r\n0";
        assert!(match parse_chunked(src as &[u8]) {
            ParseStatus::Complete(_) => true,
            _ => false,
        });
    }
}
