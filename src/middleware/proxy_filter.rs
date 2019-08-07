use std::rc::Rc;
use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{http, Error, HttpResponse};
use futures::{Poll, future::{ok, Either, FutureResult}};
use crate::border::{BorderControl, host_control::HostControlBuilder, BorderControlBuilder};
use std::sync::{Arc, Mutex};


pub struct ProxyFilterCollection {
    border: Rc<Box<dyn BorderControl>>,
}

impl Default for ProxyFilterCollection {
    fn default() -> Self {
        ProxyFilterCollection {
            border: Rc::new( HostControlBuilder::new().build() ),
        }
    }
}

impl ProxyFilterCollection {
    pub fn new() -> Self {
        ProxyFilterCollection::default()
    }

    pub fn with_border( mut self, border: Box<dyn BorderControl> ) -> Self {
        *Rc::get_mut( &mut self.border ).unwrap() = border;
        self
    }
}


impl<S, B> Transform<S> for ProxyFilterCollection
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ProxyFilterMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform( &self, service: S ) -> Self::Future {
        ok( ProxyFilterMiddleware { service, border: self.border.clone(), } )
    }
}

pub struct ProxyFilterMiddleware<S> {
    border: Rc<Box<dyn BorderControl>>,
    service: S,
}

impl<S, B> Service for ProxyFilterMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, FutureResult<Self::Response, Self::Error>>;

    fn poll_ready( &mut self ) -> Poll<(), Self::Error> { self.service.poll_ready() }

    fn call( &mut self, req: ServiceRequest ) -> Self::Future {
        if let Ok(destination) = self.border.request_visa( &req ) {
            Either::A( self.service.call(req) )
        } else {
            Either::B(
                ok(
                    req.into_response(
                        HttpResponse::NotAcceptable()
                            .finish()
                            .into_body(),
                    )
                )
            )
        }
    }
}