use std::collections::HashMap;
use std::str;
use httparse_orig;
use regex::Regex;
use parsable::{Parsable, Sendable, ParseStatus, parse_body};

pub struct Request {
    pub method: String,
    pub path: String,
    pub version: u8,
    pub headers: HashMap<String, Vec<u8>>,
    pub body: Option<Vec<u8>>,
}

impl Request {
    pub fn disassembly_path(&self) -> (&str, &str, u16, &str) {
        let re: Regex = Regex::new(r"^(.+?)://(.+?):?(\d+)?(/.*)?$").unwrap();
        for cap in re.captures_iter(self.path.as_str()) {
            return (cap.at(1).unwrap_or(""),
                    cap.at(2).unwrap_or(""),
                    str::FromStr::from_str(cap.at(3).unwrap_or("")).ok().unwrap_or(80),
                    cap.at(4).unwrap_or(""));
        }

        ("", "", 80, "")
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
    type Parsed = Request;

    fn parse(buf: &[u8]) -> ParseStatus<Self::Parsed> {
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

        let req = Request {
            method: String::from(req.method.unwrap()),
            path: String::from(req.path.unwrap()),
            version: req.version.unwrap(),
            headers: headers_hm,
            body: body,
        };

        ParseStatus::Complete(req)
    }
}


impl Sendable for Request {
    fn to_bytes(&self) -> Vec<u8> {
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

#[cfg(test)]
mod tests {
    #[test]
    fn disassembly_request_path() {
        use super::Request;
        use super::super::Parsable;

        let src_path = "http://example.com:8888/index.html";
        let mut req = Request::new();
        req.path = String::from(src_path);
        let (protocol, host, port, path) = req.disassembly_path();

        assert_eq!(protocol, "http");
        assert_eq!(host, "example.com");
        assert_eq!(port, 8888);
        assert_eq!(path, "/index.html");
    }
}
