use futures::{ Future, IntoFuture };
use actix_web::Error;
use actix_web::dev::ServiceRequest;
use url::HostAndPort;
use actix_http::error::ErrorForbidden;

pub mod host_control;

pub trait BorderControl {
    /// Requests challenged by BorderControl.
//    type Request = HttpRequest;

    /// Errors produced by the BorderControl challenge
//    type Error;

//    type Future: Future<Item = bool, Error = Self::Error>;

    /// Process the request, possibly considering past requests, to make a determination
    /// whether to allow it to pass.
//    fn allow( &mut self, req: &HttpRequest ) -> Self::Future;
    fn request_visa(&self, req: &ServiceRequest ) -> Result<&HostAndPort, Error>;
}

pub trait BorderControlBuilder {
    fn build( self ) -> Box<dyn BorderControl>;
}


#[derive(Clone)]
struct ClosedBorder;

impl ClosedBorder {
    fn new() -> Self { ClosedBorder {} }
}

impl BorderControl for ClosedBorder {
    fn request_visa(&self, req: &ServiceRequest ) -> Result<&HostAndPort, Error> {
        Err( ErrorForbidden("closed egress proxy. no destinations allowed" ) )
    }
}
