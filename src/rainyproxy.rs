use std::net::{SocketAddr, ToSocketAddrs};
use std::iter::Iterator;
use std::io::Result as IoResult;
use std::sync::Arc;
use mioco;
use mioco::tcp::TcpListener;
use request::Request;
use response::Response;
use connection::{Connection, ConnResult};

macro_rules! try_com {
    ($com: expr, err => $errexpr: expr) => {
        match $com {
            ConnResult::Ok(x) => x,
            ConnResult::ZeroPacket => return Ok(()),
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
        self.serve_custom(|&mut _, &mut _| {}, |&mut _| {})
    }

    pub fn serve_custom<RQ, RS>(&self, on_request: RQ, on_response: RS) -> IoResult<()>
        where RQ: Fn(&mut Request, &mut Option<Response>) + 'static + Send + Sync,
              RS: Fn(&mut Response) + 'static + Send + Sync
    {
        let listener = try!(TcpListener::bind(&self.addr));
        debug!("Proxy server listen at {}", &self.addr);

        let handlers = Arc::new((on_request, on_response));
        mioco::start(move || -> IoResult<()> {
            for _ in 0..mioco::thread_num() {
                let listener: TcpListener = try!(listener.try_clone());
                let _handlers = handlers.clone();

                mioco::spawn(move || -> IoResult<()> {
                    loop {
                        let mut src_conn = Connection::new(try!(listener.accept()));
                        let __handlers = _handlers.clone();

                        mioco::spawn(move || -> IoResult<()> {
                            // recieve source request
                            let mut request: Request = try_com!(src_conn.recieve(), err=>return
                            Ok(()));
                            debug!("receive from client.");

                            // connect to the server
                            let mut dest_conn = match Connection::from(&request.host()
                                                                               .as_str(),
                                                                       &request.port()) {
                                Some(conn) => conn,
                                None => return Ok(()),
                            };
                            debug!("connect to server.");

                            let mut user_res = None;
                            __handlers.0(&mut request, &mut user_res);

                            // send request to destination host
                            try_com!(dest_conn.send(&request), err=>return Ok(()));

                            debug!("send to server.");

                            // recieve destination response
                            let mut response: Response = match user_res {
                                Some(r) => r,
                                None => try_com!(dest_conn.recieve(), err=>return Ok(())),
                            };

                            debug!("recieved from server.");

                            __handlers.1(&mut response);

                            // send response to source host
                            try_com!(src_conn.send(&response), err=>return Ok(()));

                            debug!("send to client");

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
