#![feature(duration_float)]
use std::time::Duration;

use std::rc::Rc;
use log::info;
use lazy_static::*;
use futures::future::{ok, FutureResult};
use futures::{Future, Poll};
use actix_service::{Service, Transform};
use actix_web::{Error, dev::{ServiceRequest, ServiceResponse}, http::header};
use header::HeaderValue;
use prometheus::{IntCounterVec, IntGauge, Histogram, HistogramVec};
use prometheus::IntGaugeVec;
use stopwatch::Stopwatch;

lazy_static! {
    static ref HTTP_PROXY_TOTAL_LATENCY_HISTOGRAM: HistogramVec = register_histogram_vec!(
        histogram_opts!(
            "egress_http_proxy_total_duration_seconds",
            "The total time in the egress proxy and egress request"
        ).const_labels(
            labels! {
                String::from("realm") => String::from("ex-realm"),
                String::from("pipeline_id") => String::from("ex-pipeline_id"),
            }
        ),
        &["method"]
    )
    .unwrap();

    static ref HTTP_PROXY_OVERHEAD_HISTOGRAM: HistogramVec = register_histogram_vec!(
        histogram_opts!(
            "egress_http_proxy_overhead_duration_seconds",
            "The overhead time in egress proxy outside of egress request"
        ).const_labels(
            labels! {
                String::from("realm") => String::from("ex-realm"),
                String::from("pipeline_id") => String::from("ex-pipeline_id"),
            }
        ),
        &["method"]
    )
    .unwrap();

    static ref HTTP_EGRESS_REQ_LATENCY_HISTOGRAM: HistogramVec = register_histogram_vec!(
        histogram_opts!(
            "egress_http_request_duration_seconds",
            "The external HTTP request latencies in seconds."
        ).const_labels(
            labels! {
                String::from("realm") => String::from("ex-realm"),
                String::from("pipeline_id") => String::from("ex-pipeline-id"),
            }
        ),
        &["method"]
    )
    .unwrap();
}


pub struct MeasureLatencyCollection( Rc<Family> );

struct Family {
    total: &'static HistogramVec,
    egress: &'static HistogramVec,
    overhead: &'static HistogramVec,
}

impl MeasureLatencyCollection {
    pub fn new() -> Self {
        MeasureLatencyCollection(
            Rc::new(
                Family {
                    total: &HTTP_PROXY_TOTAL_LATENCY_HISTOGRAM,
                    egress: &HTTP_EGRESS_REQ_LATENCY_HISTOGRAM,
                    overhead: &HTTP_PROXY_OVERHEAD_HISTOGRAM,
                }
            )
        )
    }
}

impl<S, B> Transform<S> for MeasureLatencyCollection
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MeasureLatencyMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform( &self, service: S ) -> Self::Future {
        ok( MeasureLatencyMiddleware { service, family: self.0.clone(), } )
    }
}

pub struct MeasureLatencyMiddleware<S> {
    family: Rc<Family>,
    service: S,
}

impl<S, B> Service for MeasureLatencyMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Box<dyn Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready( &mut self ) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call( &mut self, svc_req: ServiceRequest ) -> Self::Future {
        let total_timer = Stopwatch::start_new();

        let sel = labels!{ "method" => svc_req.method().as_str(), };

        let total_histogram = self.family.total.with( &sel );
        let egress_histogram = self.family.egress.with( &sel );
        let overhead_histogram = self.family.overhead.with( &sel );

        Box::new(
        self.service
            .call( svc_req )
            .and_then( move |resp| {
                let total_dur = total_timer.elapsed();
                total_histogram.observe( total_dur.as_secs_f64() );

                if let Some(req) = resp.response().extensions().get::<Duration>() {
                    egress_histogram.observe( req.as_secs_f64() );

                    let overhead = ( total_dur - *req );
                    overhead_histogram.observe( overhead.as_secs_f64() );
                };

                Ok( resp )
            } )
        )
    }
}
