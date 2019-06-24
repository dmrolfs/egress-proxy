#[macro_use] extern crate log;

use actix_web::{client::Client, middleware::Logger, App, HttpServer, web, HttpResponse};

use egress_proxy::{
    config::Config,
    handlers::proxy,
    handlers::metrics,
    middleware::latency::MeasureLatencyCollection,
    metrics::ProxyCollection,
};

fn main() -> std::io::Result<()> {
    std::env::set_var( "RUST_LOG", "egress_proxy=trace,actix_server=trace,actix_web=trace,main=trace" );
//    std::env::set_var( "RUST_LOG", "trace" );
    env_logger::init();

    let cfg = Config::from_args();
    let forward_url = cfg.forward_url.clone();
    info!( "App Config = {:?}", cfg );

    HttpServer::new( move || {
        App::new()
            .data( Client::new() )
            .data( forward_url.clone() )
            .data( ProxyCollection::new() )
            .wrap( Logger::new( r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %D"# ) )
            .default_service(
                web::resource("")
                    .wrap(MeasureLatencyCollection::new() )
                    .to_async( proxy::forward )
            )
            .service(
                web::resource("/__proxy/metrics" )
                    .default_service(
                        web::route().to( || HttpResponse::MethodNotAllowed() ),
                    )
                    .route(web::get().to_async(metrics::gather ) ),
            )
    } )
        .listen( cfg.tcp_listener().unwrap() )?
        .system_exit()
        .run()
}
