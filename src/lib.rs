#![forbid(unsafe_code)]
#![warn(
    dead_code,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    unused_allocation,
    trivial_numeric_casts,
    clippy::single_char_pattern
)]

pub mod config;
pub mod errors;
pub mod extractors;
pub mod requests;
pub mod responses;
pub mod router;
pub mod salesforce;
