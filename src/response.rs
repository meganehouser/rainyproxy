use std::collections::HashMap;
use std::str;
use httparse_orig;
use parsable::{Parsable, Sendable, ParseStatus, parse_body};

pub struct Response {
    pub version: u8,
    pub status_code: u16,
    pub reason: String,
    pub headers: HashMap<String, Vec<u8>>,
    pub body: Option<Vec<u8>>,
}

impl Response {
    pub fn new(version: u8, status_code: u16, reason: &str) -> Response {
        Response {
            version: 1,
            status_code: 200,
            reason: String::new(),
            headers: HashMap::new(),
            body: None,
        }
    }
}

impl Parsable for Response {
    type Parsed = Response;

    fn parse(buf: &[u8]) -> ParseStatus<Self::Parsed> {
        let mut headers = [httparse_orig::EMPTY_HEADER; 100];

        let mut res = httparse_orig::Response::new(&mut headers);
        let parse_status = match res.parse(&buf) {
            Ok(status) => status,
            Err(err) => return ParseStatus::Err(err),
        };

        let res_len = match parse_status.is_complete() {
            true => parse_status.unwrap(),
            false => return ParseStatus::InProgress,
        };

        let body = match parse_body(&res.headers, &buf[res_len..]) {
            ParseStatus::Complete(bdy) => bdy,
            ParseStatus::InProgress => return ParseStatus::InProgress,
            ParseStatus::Err(err) => return ParseStatus::Err(err),
        };

        let mut headers_hm = HashMap::new();
        for h in res.headers.iter() {
            headers_hm.insert(String::from(h.name), Vec::from(h.value));
        }

        let res = Response {
            version: res.version.unwrap(),
            status_code: res.code.unwrap(),
            reason: String::from(res.reason.unwrap()),
            headers: headers_hm,
            body: body,
        };

        ParseStatus::Complete(res)
    }
}

impl Sendable for Response {
    fn to_bytes(&self) -> Vec<u8> {
        let mut headers = self.headers.clone();
        headers.insert(String::from("Connection"), Vec::from("Close"));

        let hs = headers.iter()
                        .filter(|h| h.0.as_str() != "Keep-Alive")
                        .fold(String::new(), |acc, (k, v)| {
                            return acc +
                                   format!("{}: {}\r\n", &k, str::from_utf8(&v).unwrap()).as_str();
                        });

        let s = format!("HTTP/1.{} {} {}\r\n{}\r\n",
                        self.version,
                        self.status_code,
                        self.reason.as_str(),
                        hs);

        let mut payload: Vec<u8> = Vec::from(s.as_bytes());
        if self.body.is_some() {
            payload.extend_from_slice(self.body.as_ref().unwrap());
        }

        payload
    }
}
