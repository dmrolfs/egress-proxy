#[macro_use] extern crate log;

use actix_web::{client::Client, middleware::Logger, App, HttpServer, web, HttpResponse};
use egress_proxy::{
    config::Config,
    handlers::proxy,
    handlers::metrics,
    middleware::latency::MeasureLatencyCollection,
    metrics::MetricsCollection,
};
use egress_proxy::middleware::proxy_filter::ProxyFilterCollection;
use egress_proxy::border::host_control::HostControlBuilder;
use egress_proxy::border::BorderControlBuilder;

fn setup_logger() {
    std::env::set_var( "RUST_LOG", "egress_proxy=debug,actix_server=debug,actix_web=debug,main=debug,mio=info,tokio_reactor=info" );
//    std::env::set_var( "RUST_LOG", "trace" );
//    env_logger::init();

    let log_env = env_logger::Env::default();
    let logger = env_logger::Builder::from_env( log_env ).build();

//    let logger = env_logger::init_from_env( log_env );
//    let logger = env_logger::Builder::new()
//        .filter( None, log::LevelFilter::Trace )
//        .build();

    async_log::Logger::wrap( logger, || 12 )
        .start( log::LevelFilter::Trace )
        .unwrap();
}

fn main() -> std::io::Result<()> {
    setup_logger();

    let cfg = Config::from_args();
    let forward_url = cfg.forward_url.clone();
    info!( "App Config = {:?}", cfg );

//    let border = Arc::new(HostControl::new(  forward_url ) );
//    let border = Arc::new(
//        HostControlBuilder::new()
//            .with_default_destination( forward_url )
//            .build()
//    );

    HttpServer::new( move || {
        App::new()
            .data( Client::new() )
//            .data( forward_url.clone() )
            .data( MetricsCollection::new() )
            .wrap( Logger::new( r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %D"# ) )
            .default_service(
                web::resource("")
                    .wrap(MeasureLatencyCollection::new() )
                    .wrap(
                        ProxyFilterCollection::new().with_border(
                            HostControlBuilder::new()
                                .with_default_destination( forward_url.clone() )
                                .build()
                        )
                    )
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
