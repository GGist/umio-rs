//! Message Based Readiness API
//!
//! This library is a thin wrapper around mio for clients who wish to
//! use a single udp socket in conjunction with message passing and
//! timeouts.

extern crate crossbeam;
extern crate mio;
extern crate threadpool;

mod buffer;
mod client;
mod dispatcher;
mod eloop;
mod provider;
//mod route;

/// Exports of bare mio types.
pub mod external;

pub use client::{Sender};
pub use dispatcher::{Dispatcher};
pub use eloop::{ELoopBuilder, ELoop};
pub use provider::{Provider};