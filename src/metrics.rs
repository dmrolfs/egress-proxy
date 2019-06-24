use lazy_static::*;
use prometheus::{IntCounterVec, IntGauge, IntGaugeVec, Histogram, HistogramVec};
use std::rc::Rc;

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
    ).unwrap();

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

    pub static ref HTTP_BODY_GAUGE_BYTES: IntGauge = register_int_gauge!(
        opts!(
            "egress_http_response_size_bytes",
            "The HTTP response sizes in bytes.",
            labels! {
                "realm" => "ex-realm",
                "pipeline_id" => "ex-pipeline-id",
            }
        )
    )
    .unwrap();
}

pub struct ProxyCollection( Rc<Family> );

pub struct Family {
    allowed: &'static IntCounterVec,
    blocked: &'static IntCounterVec,
    body_size: &'static IntGauge,
}

impl ProxyCollection {
    pub fn new() -> Self {
        ProxyCollection(
            Rc::new(
                Family {
                    allowed: &ALLOWED_TOTAL,
                    blocked: &BLOCKED_TOTAL,
                    body_size: &HTTP_BODY_GAUGE_BYTES,
                }
            )
        )
    }
}