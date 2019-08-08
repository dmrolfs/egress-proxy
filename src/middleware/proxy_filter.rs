use std::rc::Rc;
use std::sync::{Arc, Mutex};
use prometheus::IntCounterVec;
use lazy_static::*;
use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{http, Error, HttpResponse};
use futures::{Poll, future::{ok, Either, FutureResult}};
use crate::border::{BorderControl, host_control::HostControlBuilder, BorderControlBuilder};


lazy_static! {
    pub static ref ALLOWED_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!(
            "egress_http_request_allowed_total",
            "Total number of egress HTTP requests allowed.",
            labels! {
                "realm" => "ex-realm",
                "pipeline_id" => "ex-pipeline-id",
            }
        ),
        &["method"]
    )
    .unwrap();

    pub static ref BLOCKED_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!(
            "egress_http_request_blocked_total",
            "Total number of egress HTTP requests blocked.",
            labels! {
                "realm" => "ex-realm",
                "pipeline_id" => "ex-pipeline-id",
            }
        ),
        &["method"]
    )
    .unwrap();
}

pub struct ProxyFilterCollection( Rc<Family> );

struct Family {
    border: Box<dyn BorderControl>,
    allowed: &'static IntCounterVec,
    blocked: &'static IntCounterVec,
}

impl Default for ProxyFilterCollection {
    fn default() -> Self {
        ProxyFilterCollection(
            Rc::new(
                Family {
                    border: HostControlBuilder::new().build(),
                    allowed: &ALLOWED_TOTAL,
                    blocked: &BLOCKED_TOTAL,
                }
            )
        )
    }
}

impl ProxyFilterCollection {
    pub fn new() -> Self {
        ProxyFilterCollection::default()
    }

    pub fn with_border( mut self, border: Box<dyn BorderControl> ) -> Self {
        let family = Rc::get_mut( &mut self.0 ).unwrap();
        family.border = border;
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
        ok( ProxyFilterMiddleware { service, family: self.0.clone(), } )
    }
}

pub struct ProxyFilterMiddleware<S> {
    family: Rc<Family>,
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
        let method_sel = labels!{ "method" => req.method().as_str(), };

        if let Ok(destination) = self.family.border.request_visa( &req ) {
            let allowed = self.family.allowed.with( &method_sel );
            allowed.inc();

            Either::A( self.service.call(req) )
        } else {
            let blocked = self.family.blocked.with( &method_sel );
            blocked.inc();

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