#![allow(unused)]
#![feature(duration_float)]

extern crate env_logger;
#[macro_use] extern crate prometheus;

pub mod config;
pub mod metrics;
pub mod handlers;
pub mod middleware;