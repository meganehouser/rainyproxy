use std::net::{SocketAddr, ToSocketAddrs};
use std::iter::Iterator;
use std::io::Result as IoResult;
use mioco;
use mioco::tcp::TcpListener;
use request::Request;
use response::Response;
use connection::{Connection, ConnResult};

macro_rules! try_com {
    ($com: expr, err => $errexpr: expr) => {
        match $com {
            ConnResult::Ok(x) => x,
            ConnResult::ParseErr(_) => $errexpr,
            ConnResult::IoError(_) => $errexpr,
        }
    }
}

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
                                let request: Request = try_com!(src_conn.recieve(), err=>break);
                                debug!("receive from client.");

                                // connect to the server
                                let mut dest_conn = match Connection::from(&request.host()
                                                                                   .as_str(),
                                                                           &request.port()) {
                                    Some(conn) => conn,
                                    None => break,
                                };
                                debug!("connect to server.");

                                // send request to destination host
                                try_com!(dest_conn.send(&request), err=>break);

                                debug!("send to server.");

                                // recieve destination response
                                let response: Response = try_com!(dest_conn.recieve(), err=>break);

                                debug!("recieved from server.");

                                // send response to source host
                                try_com!(src_conn.send(&response), err=>break);

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
