use futures;
use futures::{Future, BoxFuture};
use futures::future::FutureResult;

use hyper;
use hyper::{Client, StatusCode, Body};
use hyper::client::HttpConnector;
use hyper::server::{Service, Request, Response};


use regex;

use tlsclient::HttpsConnector;

#[derive(Clone)]
pub struct Routes {
    pub routes: Vec<(regex::Regex, String)>,
    pub regexes: regex::RegexSet,
}

pub struct Proxy {
    pub routes: Routes,
    pub client: Client<HttpConnector, Body>,
    pub tls_client: Client<HttpsConnector, Body>,
}

impl Service for Proxy {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let uri = req.uri();
        let matches = self.routes.regexes.matches(uri.path());
        let fut = {
            if !matches.matched_any() {
                futures::future::ok(Response::new().with_status(StatusCode::NotFound)).boxed()
            } else {
                // Find the most specific match (unwrap called here because of the above check)
                let index = matches.iter().next().unwrap();
                let (ref regex, ref other_site) = self.routes.routes[index];
                let url = hyper::Url::parse(other_site).expect("configuration problem, other site not valid URL");

                    println!("forward request to {}", url);
                    let secure = url.scheme() == "https";
                    let mut proxied_request = hyper::client::Request::new(req.method().clone(), url);
                    *proxied_request.headers_mut() = req.headers().clone();
                    let req = if secure {
                        self.tls_client.request(proxied_request)
                    } else {
                        self.client.request(proxied_request)
                    };
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
            }
        };
        fut
    }
}
