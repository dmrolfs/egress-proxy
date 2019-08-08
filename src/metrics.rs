use lazy_static::*;
use prometheus::IntGauge;
use std::rc::Rc;

lazy_static! {
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
    pub body_size: &'static IntGauge,
}

impl MetricsCollection {
    pub fn new() -> Self {
        MetricsCollection(
            Rc::new(
                Family {
                    body_size: &HTTP_BODY_GAUGE_BYTES,
                }
            )
        )
    }
}