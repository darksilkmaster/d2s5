use futures;
use futures::{Future};

use hyper;
use hyper::{Client, StatusCode, Body};
use hyper::client::HttpConnector;
use hyper::server::{Service, Request, Response};

pub struct Proxy {
    pub client: Client<HttpConnector, Body>,
}

impl Service for Proxy {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let fut = {
            // Find the most specific match (unwrap called here because of the above check)
            let other_site = "http://localhost:8000";
            let url = other_site.parse().unwrap();

            println!("forward request to {}", url);
            let mut proxied_request = hyper::client::Request::new(req.method().clone(), url);
            *proxied_request.headers_mut() = req.headers().clone();
            let req =
                self.client.request(proxied_request);
            Box::new(req.then(|res| {
                println!("got response back!");
                if let Ok(res) = res {
                    futures::future::ok(
                        Response::new()
                            .with_status(res.status().clone())
                            .with_headers(res.headers().clone())
                            .with_body(res.body()))
                } else {
                    futures::future::ok(
                        Response::new()
                            .with_status(StatusCode::ServiceUnavailable))
                }
            })) as Self::Future
        };
        fut
    }
}
