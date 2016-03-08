use std::collections::HashMap;
use std::str;
use httparse_orig;
use parsable::{Parsable, Sendable, ParseStatus};

pub struct Request {
    pub method: Option<String>,
    pub path: Option<String>,
    pub version: Option<u8>,
    pub headers: HashMap<String, Vec<u8>>,
    pub body: Option<Vec<u8>>,
}

impl Parsable for Request {
    fn new() -> Request {
        Request {
            method: None,
            path: None,
            version: None,
            headers: HashMap::new(),
            body: None,
        }
    }

    fn parse(&mut self, buf: &[u8]) -> ParseStatus<usize> {
        let mut headers = [httparse_orig::EMPTY_HEADER; 100];

        let mut req = httparse_orig::Request::new(&mut headers);
        let parse_status = match req.parse(&buf) {
            Ok(status) => status,
            Err(err) => return ParseStatus::Err(err),
        };

        let req_len = match parse_status.is_complete() {
            true => parse_status.unwrap(),
            false => return ParseStatus::InProgress,
        };

        let body = match parse_body(&req, &buf[req_len..]) {
            ParseStatus::Complete(bdy) => bdy,
            ParseStatus::InProgress => return ParseStatus::InProgress,
            ParseStatus::Err(err) => return ParseStatus::Err(err),
        };

        let mut headers_hm = HashMap::new();
        for h in req.headers.iter() {
            headers_hm.insert(String::from(h.name), Vec::from(h.value));
        }

        self.method = Some(String::from(req.method.unwrap()));
        self.path = Some(String::from(req.path.unwrap()));
        self.version = Some(req.version.unwrap());
        self.headers = headers_hm;
        self.body = body;
        return ParseStatus::Complete(0);
    }
}

fn parse_body(req: &httparse_orig::Request, buf: &[u8]) -> ParseStatus<Option<Vec<u8>>> {

    if req.method.unwrap() == "GET" {
        return ParseStatus::Complete(None);
    }

    match req.headers.iter().find(|&&h| h.name == "Content-Length") {
        Some(h) => {
            let len: usize = str::FromStr::from_str(str::from_utf8(h.value).unwrap()).unwrap();
            if len <= buf.len() {
                return ParseStatus::Complete(Some(Vec::from(buf)));
            } else {
                return ParseStatus::InProgress;
            }
        }
        None => return ParseStatus::Complete(None),
    }
}

impl Sendable for Request {
    fn to_bytes(&self) -> Vec<u8> {
        assert!(self.method.is_some());
        assert!(self.path.is_some());
        assert!(self.version.is_some());

        let headers = self.headers
                          .iter()
                          .fold(String::new(), |acc, (k, v)| {
                              return acc +
                                     format!("{}: {}\r\n", &k, str::from_utf8(&v).unwrap())
                                         .as_str();
                          });

        let s = format!("{} {} HTTP/1.{}\r\n{}\r\n",
                        self.method.as_ref().unwrap().as_str(),
                        self.path.as_ref().unwrap().as_str(),
                        self.version.as_ref().unwrap(),
                        headers);

        let mut payload: Vec<u8> = Vec::from(s.as_bytes());
        if self.body.is_some() {
            payload.extend_from_slice(self.body.as_ref().unwrap());
        }

        payload
    }
}
