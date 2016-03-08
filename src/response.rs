use std::collections::HashMap;
use std::str;
use httparse_orig;
use parsable::{Parsable, Sendable, ParseStatus};

pub struct Response {
    pub version: Option<u8>,
    pub status_code: Option<u16>,
    pub reason: Option<String>,
    pub headers: HashMap<String, Vec<u8>>,
    pub body: Option<Vec<u8>>,
}

impl Parsable for Response {
    fn new() -> Response {
        Response {
            version: None,
            status_code: None,
            reason: None,
            headers: HashMap::new(),
            body: None,
        }
    }

    fn parse(&mut self, buf: &[u8]) -> ParseStatus<usize> {
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


        let body = match parse_body(&res, &buf[res_len..]) {
            ParseStatus::Complete(bdy) => bdy,
            ParseStatus::InProgress => return ParseStatus::InProgress,
            ParseStatus::Err(err) => return ParseStatus::Err(err),
        };

        let mut headers_hm = HashMap::new();
        for h in res.headers.iter() {
            headers_hm.insert(String::from(h.name), Vec::from(h.value));
        }

        self.version = Some(res.version.unwrap());
        self.status_code = Some(res.code.unwrap());
        self.reason = Some(String::from(res.reason.unwrap()));
        self.headers = headers_hm;
        self.body = body;
        return ParseStatus::Complete(0);
    }
}

fn parse_body(req: &httparse_orig::Response, buf: &[u8]) -> ParseStatus<Option<Vec<u8>>> {
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

impl Sendable for Response {
    fn to_bytes(&self) -> Vec<u8> {
        let mut payload: Vec<u8> = Vec::new();
        let headers = self.headers
                          .iter()
                          .fold(String::new(), |acc, (k, v)| {
                              return acc +
                                     format!("{}: {}\r\n", &k, str::from_utf8(&v).unwrap())
                                         .as_str();
                          });

        let mut s = format!("HTTP/1.{} {} {}\r\n{}\r\n",
                            self.version.as_ref().unwrap(),
                            self.status_code.as_ref().unwrap(),
                            self.reason.as_ref().unwrap().as_str(),
                            headers);

        let mut payload: Vec<u8> = Vec::from(s.as_bytes());
        if self.body.is_some() {
            payload.extend_from_slice(self.body.as_ref().unwrap());
        }

        debug!("response: {}", String::from_utf8_lossy(&payload));
        payload
    }
}
