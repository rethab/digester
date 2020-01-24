#![feature(option_result_contains)]

extern crate chrono;

/// A channel is something where we can pull updates
/// from that can be sent as updates to people.
pub mod channel;
pub use channel::*;
pub mod github_release;
pub mod rss;
