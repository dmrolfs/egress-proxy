#[macro_use] extern crate log;
extern crate env_logger;

use std::any::Any;
use std::collections::HashMap;
use listenfd::ListenFd;
use futures::Future;
use futures::future::{ok as fut_ok};
use actix_web::{web, App, HttpServer, Error, HttpResponse};
use actix_web::middleware::Logger;
//use actix_web::client::Client;

type P = web::Path<String>;
type Q = web::Query<HashMap<String, String>>;

//fn index( info: (P, Q) ) -> impl Responder {
//    format!( "Hello path:{} query:{:?}", info.0, info.1 )
//}

fn proxy( info: (P, Q) ) -> impl Future<Item = HttpResponse, Error = Error> {
    fut_ok(
        HttpResponse::from( format!( "Hello path:{} query:{:?}", info.0, info.1 ) )
    )
}

fn main() -> std::io::Result<()> {
    std::env::set_var( "RUST_LOG", "debug" );
    env_logger::init();

    info!( "listening for port..." );
    let mut listenfd = ListenFd::from_env();
    info!( "starting up at {:?} len:{}...", listenfd.type_id(), listenfd.len() );

    let mut server = HttpServer::new(
        || App::new()
            .wrap( Logger::new( r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %D"# ) )
            .service(
                web::resource( "egress/{tail:.*}" )
                    .route( web::get().to_async(proxy))
//                    .to( proxy )
            )
    );

    server = if let Some(l) = listenfd.take_tcp_listener( 0 ).unwrap() {
        server.listen( l ).unwrap()
    } else {
        server.bind( "127.0.0.1:8080" ).unwrap()
    };

    server.run()
}
