use std::net::{SocketAddr, ToSocketAddrs, lookup_host};
use std::str::FromStr;
use std::iter::Iterator;
use std::io::Result as IoResult;
use mioco;
use mioco::tcp::{TcpListener, TcpStream};
use request::Request;
use response::Response;
use connection::{Connection, ConnResult};

///
/// HTTP Proxy Server implemenation
///
pub struct RainyProxy {
    pub addr: SocketAddr,
}

impl RainyProxy {
    pub fn new<T: ToSocketAddrs>(addr: &T) -> RainyProxy {
        let sock = addr.to_socket_addrs().unwrap().next().unwrap();
        RainyProxy { addr: sock }
    }

    pub fn serve(&self) -> IoResult<()> {
        let listener = try!(TcpListener::bind(&self.addr));
        debug!("Proxy server listen at {}", &self.addr);

        mioco::start(move || {
            for _ in 0..mioco::thread_num() {
                let listener: TcpListener = try!(listener.try_clone());
                mioco::spawn(move || -> IoResult<()> {
                    loop {
                        let mut src_conn = Connection::new(try!(listener.accept()));
                        mioco::spawn(move || -> IoResult<()> {
                            loop {
                                // recieve source request
                                let mut request: Request = match src_conn.recieve() {
                                    ConnResult::Ok(req) => req,
                                    ConnResult::ParseErr(p_err) => break,
                                    ConnResult::IoError(io_err) => break,
                                };

                                // lookup and connect to destination host
                                let addr = match lookup_dest(&mut request) {
                                    Some(sock) => sock,
                                    None => {
                                        debug!("cannot lookup host {:?}", request.path);
                                        break;
                                    }
                                };
                                request.path = Some(request.path
                                                           .as_ref()
                                                           .unwrap()
                                                           .split("/")
                                                           .skip(3)
                                                           .fold(String::from(""),
                                                                 |acc, s| acc + "/" + s));

                                let mut dest_conn = Connection::new(TcpStream::connect(&addr)
                                                                        .unwrap());
                                debug!("connecting to server {}", addr);

                                // send request to destination host
                                match dest_conn.send(&request) {
                                    ConnResult::Ok(()) => {}
                                    ConnResult::ParseErr(p_err) => break,
                                    ConnResult::IoError(io_err) => break,
                                }

                                debug!("send to server.");

                                // recieve destination response
                                let response: Response = match dest_conn.recieve() {
                                    ConnResult::Ok(req) => req,
                                    ConnResult::ParseErr(p_err) => break,
                                    ConnResult::IoError(io_err) => break,
                                };

                                debug!("recieved from server.");

                                // send response to source host
                                match src_conn.send(&response) {
                                    ConnResult::Ok(()) => {}
                                    ConnResult::ParseErr(p_err) => break,
                                    ConnResult::IoError(io_err) => break,
                                };

                                debug!("send to client");
                            }

                            Ok(())
                        });
                    }
                });
            }

            Ok(())
        });
        Ok(())
    }
}


fn lookup_dest(req: &mut Request) -> Option<SocketAddr> {
    if req.path.is_none() {
        return None;
    }

    let path = req.path.as_ref().unwrap();
    let host = extract_host(&path);
    let mut lookuped: SocketAddr = match lookup_host(&host) {
        Ok(mut lkup_host) => lkup_host.nth(0).unwrap().unwrap(),
        Err(e) => return None,
    };

    let to_port = extract_port(path).unwrap_or(80);
    lookuped.set_port(to_port);
    debug!("forword host: {}", &lookuped);

    return Some(lookuped);
}

fn extract_host(path: &str) -> String {
    path.trim_left_matches("http://")
        .split("/")
        .nth(0)
        .unwrap()
        .split(":")
        .nth(0)
        .unwrap()
        .to_string()
}

fn extract_port(path: &str) -> Option<u16> {
    let port_str = match path.split(":").nth(2) {
                       Some(s) => s,
                       None => return None,
                   }
                   .split("/")
                   .nth(0)
                   .unwrap();
    FromStr::from_str(port_str).ok()
}
