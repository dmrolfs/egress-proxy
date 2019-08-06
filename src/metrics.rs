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

pub struct MetricsCollection(pub Rc<Family> );

pub struct Family {
    pub allowed: &'static IntCounterVec,
    pub blocked: &'static IntCounterVec,
    pub body_size: &'static IntGauge,
}

impl MetricsCollection {
    pub fn new() -> Self {
        MetricsCollection(
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