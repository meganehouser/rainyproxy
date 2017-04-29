use std::sync::Arc;
use std::net::SocketAddr;
use hyper::server::{self, Http};
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use futures::stream::Stream;
use proxy::Proxy;

pub struct RainyProxy {
    pub addr: SocketAddr,
}

impl RainyProxy {
    pub fn new(addr: SocketAddr) -> RainyProxy {
        RainyProxy { addr: addr }
    }

    pub fn serve(&self) {
        self.serve_custom(|_| None, |_| None);
    }

    pub fn serve_custom<RQ, RS>(&self, on_request: RQ, on_response: RS)
        where RQ: Fn(&server::Request) -> Option<server::Response> + 'static + Send,
              RS: Fn(&server::Response) -> Option<server::Response> + 'static + Send
    {
        let on_req = Arc::new(on_request);
        let on_resp = Arc::new(on_response);

        let http = Http::new();
        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let listener = TcpListener::bind(&self.addr, &handle).unwrap();
        info!("Proxy server listen at {}", &self.addr);

        let server = listener.incoming()
                             .for_each(|(sock, addr)| {
                                 let service = Proxy::new(handle.clone(),
                                                          on_req.clone(),
                                                          on_resp.clone());
                                 http.bind_connection(&handle, sock, addr, service);
                                 Ok(())
                             });

        core.run(server).unwrap();
    }
}
