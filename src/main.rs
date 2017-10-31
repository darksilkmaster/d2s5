extern crate hyper;
extern crate futures;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;
extern crate tokio_core;
extern crate tokio_signal;
extern crate tokio_service;

mod config;
mod errors;
mod proxy;

use std::io;
use std::io::{Write};
use std::net::SocketAddr;

use hyper::server::Http;

use futures::{Future, Stream};

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;

macro_rules! eprint {
    ($($arg:tt)*) => ($crate::io::stdout().write_fmt(format_args!($($arg)*)).unwrap());
}

macro_rules! eprintln {
    () => (eprint!("\n"));
    ($fmt:expr) => (eprint!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (eprint!(concat!($fmt, "\n"), $($arg)*));
}

fn run() -> errors::Result<()> {
    let mut core = Core::new()?;
    let handle = core.handle();

    env_logger::init()?;

    let addr : SocketAddr = "0.0.0.0:8001".parse()?;
    let sock = TcpListener::bind(&addr, &handle)?;
    let client = hyper::Client::new(&handle);
    let http = Http::new();
    println!("Listening on http://{} with 1 thread...", sock.local_addr()?);

    let server
        = sock.incoming().for_each(|(sock, remote_addr)| {
        let service = proxy::Proxy { client: client.clone() };
        futures::future::ok(remote_addr).and_then(|remote_addr| { http.bind_connection(&handle, sock, remote_addr, service); Ok(()) })
    });
    core.run(server)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);

        for e in e.iter().skip(1) {
            eprintln!("caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            eprintln!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}
