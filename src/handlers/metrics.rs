use futures::{IntoFuture};
use actix_web::{HttpRequest, Error};
use prometheus::Encoder;
use actix_http::error::ErrorConflict;

pub fn gather( _req: HttpRequest ) -> impl IntoFuture<Item = String, Error = Error> {
    let mut buffer = Vec::new();
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    encoder.encode( &metric_families, &mut buffer ).unwrap();

    String::from_utf8( buffer )
        .map_err( ErrorConflict )
        .map( |payload| {
            println!( "{}", payload );
            payload
        });
}