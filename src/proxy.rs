use std::sync::Arc;
use tokio_core::reactor::Handle;
use futures::{future, Future};
use hyper::client::{self, Client};
use hyper::server::{self, Service};
use hyper::error::Error;
use hyper_tls::HttpsConnector;

pub struct Proxy<RQ, RS>
    where RQ: Fn(&server::Request) -> Option<server::Response> + 'static + Send,
          RS: Fn(&server::Response) -> Option<server::Response> + 'static + Send
{
    handle: Handle,
    on_request: Arc<RQ>,
    on_response: Arc<RS>,
}

impl<RQ, RS> Proxy<RQ, RS>
    where RQ: Fn(&server::Request) -> Option<server::Response> + 'static + Send,
          RS: Fn(&server::Response) -> Option<server::Response> + 'static + Send
{
    pub fn new(handle: Handle, on_req: Arc<RQ>, on_res: Arc<RS>) -> Proxy<RQ, RS> {
        Proxy {
            handle: handle,
            on_request: on_req,
            on_response: on_res,
        }
    }
}

impl<RQ, RS> Service for Proxy<RQ, RS>
    where RQ: Fn(&server::Request) -> Option<server::Response> + 'static + Send,
          RS: Fn(&server::Response) -> Option<server::Response> + 'static + Send
{
    type Request = server::Request;
    type Response = server::Response;
    type Error = Error;
    type Future = Box<Future<Item=Self::Response, Error = Error>>;

    fn call(&self, req: server::Request) -> Self::Future {
        if let Some(srv_resp) = (self.on_request)(&req) {
            return future::ok(srv_resp).boxed();
        }

        let method = req.method().clone();
        let uri = req.uri().clone();
        let mut client_req = client::Request::new(method, uri);
        client_req.headers_mut().extend(req.headers().iter());
        client_req.set_body(req.body());

        let client = Client::configure()
                         .connector(HttpsConnector::new(4, &self.handle))
                         .build(&self.handle);

        let func = self.on_response.clone();
        let cl_uri = client_req.uri().clone();

        let resp = client.request(client_req)
                         .then(move |result| {
                             info!("{}", cl_uri);
                             match result {
                                 Ok(client_resp) => {
                                     let srv_resp = server::Response::new()
                                                        .with_status(client_resp.status())
                                                        .with_headers(client_resp.headers()
                                                                                 .clone())
                                                        .with_body(client_resp.body());
                                     if let Some(r) = (func)(&srv_resp) {
                                         Ok(r)
                                     } else {
                                         Ok(srv_resp)
                                     }
                                 }
                                 Err(e) => Err(e),
                             }
                         });
        Box::new(resp)
    }
}
