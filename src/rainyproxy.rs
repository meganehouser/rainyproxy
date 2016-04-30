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
    };
}

macro_rules! bad_gateway {
    ($com: expr) => {
        match $com {
            ConnResult::Ok(x) => x,
            ConnResult::ZeroPacket => return Response::bad_gateway_502(),
            ConnResult::ParseErr(_) => return Response::bad_gateway_502(),
            ConnResult::IoError(_) => return Response::bad_gateway_502(),
        }
    };
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
        self.serve_custom(|_| None, |_| {})
    }

    pub fn serve_custom<RQ, RS>(&self, on_request: RQ, on_response: RS) -> IoResult<()>
        where RQ: Fn(&mut Request) -> Option<Response> + 'static + Send + Sync,
              RS: Fn(&mut Response) + 'static + Send + Sync
    {
        let listener = try!(TcpListener::bind(&self.addr));
        debug!("Proxy server listen at {}", &self.addr);

        let handlers = Arc::new((on_request, on_response));
        mioco::start(move || -> IoResult<()> {
            for _ in 0..mioco::thread_num() {
                let _handlers = handlers.clone();
                let listener: TcpListener = try!(listener.try_clone());

                mioco::spawn(move || -> IoResult<()> {
                    loop {
                        let __handlers = _handlers.clone();
                        let mut src_conn = Connection::new(try!(listener.accept()));

                        mioco::spawn(move || -> IoResult<()> {
                            let (on_request, on_response) = (&__handlers.0, &__handlers.1);

                            debug!("receive from the source host.");
                            let mut request = try_com!(src_conn.recieve::<Request>(), err=>return
                            Ok(()));

                            let mut response = match on_request(&mut request) {
                                Some(usr_req) => usr_req,
                                None => {
                                    debug!("connect to the destination host.");
                                    match Connection::from(&request) {
                                        Some(mut dest_conn) => {
                                            (|| {
                                                debug!("send to the destination host.");
                                                bad_gateway!(dest_conn.send(&request));

                                                debug!("recieved from the destination host.");
                                                return bad_gateway!(dest_conn.recieve::<Response>());
                                            })()
                                        }
                                        None => Response::bad_gateway_502(),
                                    }
                                }
                            };

                            on_response(&mut response);

                            debug!("send to the surce host.");
                            try_com!(src_conn.send(&response), err=>return Ok(()));

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
