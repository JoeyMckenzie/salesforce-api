//! Common Blizzard HTTP interactions and downstream calls, leveraging [reqwest](https://docs.rs/reqwest/latest/reqwest/) for
//! connecting to Blizzard API endpoints.

#![forbid(unsafe_code)]
#![warn(
    dead_code,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    unused_allocation,
    trivial_numeric_casts,
    clippy::single_char_pattern
)]

mod config;
