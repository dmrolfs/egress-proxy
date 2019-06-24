use actix_web::{client::Client, Error, HttpRequest, HttpResponse};
use actix_web::web::{Data, Payload};
use url::Url;
use futures::Future;
use prometheus::HistogramVec;
use log::info;
use actix_http::http::{HeaderName, header};
use stopwatch::Stopwatch;
use core::borrow::{BorrowMut, Borrow};
use std::time::Duration;
use crate::metrics::ProxyCollection;

fn include_header( h: &HeaderName ) -> bool {
    match h {
        &header::CONNECTION => false,
        &header::CONTENT_LENGTH => false,
        &header::CONTENT_ENCODING => false,
//        &header::X_FRAME_OPTIONS => true,
//        &header::DATE => true,
//        &header::CONTENT_TYPE => true,
//        &header::ACCESS_CONTROL_ALLOW_ORIGIN => true,
//        &header::ACCESS_CONTROL_ALLOW_CREDENTIALS => true,
//        &header::SERVER => true,
//        &header::X_CONTENT_TYPE_OPTIONS => true,
//        &header::X_XSS_PROTECTION => true,
//        &header::REFERRER_POLICY => true,
        _ => true,
    }
}

pub fn forward(
    req: HttpRequest,
    payload: Payload,
    url: Data<Url>,
    client: Data<Client>,
    proxy_collection: Data<ProxyCollection>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let mut new_url = url.get_ref().clone();
    new_url.set_path( req.uri().path() );
    new_url.set_query( req.uri().query() );

    info!( "REQUEST: {:?}", req );
    let forwarded_req = client.request_from( new_url.as_str(), req.head() );
    let forwarded_req = if let Some(addr) = req.head().peer_addr {
        forwarded_req.header( "x-forwarded-for", format!( "{}", addr.ip() ) )
    } else {
        forwarded_req
    };

    let method_sel = labels!{ "method" => req.method().as_str(), };
    let family = proxy_collection.get_ref();

    let allowed = family.allowed.with( method_sel );
    allowed.inc();

    if Some(size_value) = req.headers().get( header::CONTENT_LENGTH ) {
        let size = size_value.parse::<i64>().unwrap();
        family.body_size.set( size );
    }

    let mut request_timer = Stopwatch::start_new();

    forwarded_req
        .send_stream( payload )
        .map_err( Error::from )
        .map( move |res| {
            let request_duration = request_timer.elapsed();
//            request_timer.observe_duration();

            let mut client_resp = HttpResponse::build( res.status() );
            client_resp.extensions_mut().insert( request_duration );

            info!( "STATUS: {:?}", res.status() );
            for ( header_name, header_value) in
                res.headers().iter().filter( |(h, _)| include_header(h) )
                {
                    info!( "HEADER: {}={:?}", header_name, header_value );
                    client_resp.header( header_name.clone(), header_value.clone() );
                }

            let r = client_resp.streaming( res );
            info!( "RESPONSE: {:?}", &r );
            info!( "RESPONSE_TIME: {:?}", r.extensions().get::<Duration>());
            r
        })
}