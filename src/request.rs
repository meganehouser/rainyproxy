use std::collections::HashMap;
use std::str;
use httparse_orig;
use parsable::{Parsable, Sendable, ParseStatus, parse_body};

pub struct Request {
    pub method: String,
    pub path: String,
    pub version: u8,
    pub headers: HashMap<String, Vec<u8>>,
    pub body: Option<Vec<u8>>,
}

impl Request {
    pub fn host(&self) -> String {
        self.path
            .trim_left_matches("http://")
            .split("/")
            .nth(0)
            .unwrap()
            .split(":")
            .nth(0)
            .unwrap()
            .to_string()
    }
    pub fn port(&self) -> u16 {
        let port_str = match self.path.split(":").nth(2) {
                           Some(s) => s,
                           None => return 80,
                       }
                       .split("/")
                       .nth(0)
                       .unwrap();
        str::FromStr::from_str(port_str).ok().unwrap_or(80)
    }

    pub fn must_close(&self) -> bool {
        let conn = self.headers.get("Connection");

        if self.version == 0 && conn.is_some() && conn.unwrap().as_slice() != b"Keep-Alive" {
            return true;
        } else if self.version == 1 && conn.is_some() && conn.unwrap().as_slice() != b"Close" {
            return false;
        }
        return true;
    }
}

impl Parsable for Request {
    pub fn new() -> Request {
        Request {
            method: String::from("GET"),
            path: String::from("/"),
            version: 1,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn parse(&mut self, buf: &[u8]) -> ParseStatus<usize> {
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

        let body = match parse_body(&req.headers, &buf[req_len..]) {
            ParseStatus::Complete(bdy) => bdy,
            ParseStatus::InProgress => return ParseStatus::InProgress,
            ParseStatus::Err(err) => return ParseStatus::Err(err),
        };

        let mut headers_hm = HashMap::new();
        for h in req.headers.iter() {
            headers_hm.insert(String::from(h.name), Vec::from(h.value));
        }

        self.method = String::from(req.method.unwrap());
        self.path = String::from(req.path.unwrap());
        self.version = req.version.unwrap();
        self.headers = headers_hm;
        self.body = body;

        let length = (match self.body.as_ref() {
            Some(ref b) => b.len(),
            None => 0,
        }) + req_len;

        ParseStatus::Complete(length)
    }
}


impl Sendable for Request {
    pub fn to_bytes(&self) -> Vec<u8> {
        let path = self.path
                       .split("/")
                       .skip(3)
                       .fold(String::from(""), |acc, s| acc + "/" + s);

        let mut headers = self.headers.clone();
        headers.insert(String::from("Connection"), Vec::from("Close"));

        let hs = headers.iter()
                        .filter(|h| h.0.as_str() != "Keep-Alive")
                        .fold(String::new(), |acc, (k, v)| {
                            return acc +
                                   format!("{}: {}\r\n", &k, str::from_utf8(&v).unwrap()).as_str();
                        });

        let s = format!("{} {} HTTP/1.{}\r\n{}\r\n",
                        self.method.as_str(),
                        path.as_str(),
                        self.version,
                        hs);

        let mut payload: Vec<u8> = Vec::from(s.as_bytes());
        if self.body.is_some() {
            payload.extend_from_slice(self.body.as_ref().unwrap());
        }

        payload
    }
}
