#![doc(html_root_url = "https://mackwic.github.io/rspec")]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![allow(dead_code)]

#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate derive_new;

extern crate colored;
#[cfg(feature = "expectest_compat")]
extern crate expectest;
extern crate time;

pub mod block;
pub mod header;
pub mod logger;
pub mod report;
pub mod runner;

mod visitor;

pub use crate::block::{describe, given, suite};
pub use crate::logger::Logger;
pub use crate::runner::{Configuration, ConfigurationBuilder, Runner};

use crate::block::Suite;

/// A wrapper for conveniently running a test suite with
/// the default configuration with considerebly less glue-code.
///
/// # Examples
///
/// ```
/// # extern crate rspec;
/// #
/// # pub fn main() {
/// rspec::run(&rspec::given("a scenario", (), |ctx| {
///     ctx.when("...", |ctx| {
///         // ...
///     });
///
///     ctx.then("...", |env| { /* ... */ });
/// }));
/// # }
/// ```
pub fn run<T>(suite: &Suite<T>)
where
    T: Clone,
{
    use std::io;
    use std::sync::Arc;

    let logger = Arc::new(Logger::new(io::stdout()));
    let configuration = ConfigurationBuilder::default().build().unwrap();
    let runner = Runner::new(configuration, vec![logger]);

    runner.run(suite);
}

#[cfg(test)]
mod tests {

    pub use super::*;
    pub use crate::block::*;

    // Test list:
    // x check that tests can call `assert_eq!`
    // x check that tests can return Err or Ok
    // x runner can count the tests
    // x runner can count the success and failed
    // x check that we can use before in a describe
    // x check that we can use after in a describe
    // x check that after/before are run in all child contextes
    // x runner broadcasts run events
    // x progress logger is an event handler
    // x pluggable loggers via logger trait
    // - stats time events is an event handler
    // - detect slow tests via treshold
    // x time the total running time
    // - failure-only via a tmp file
    // - filter tests
    // - coloration
    // - seed for deterministic randomization
    // - fail-fast fail at the first failed test
    // x beforeAll
    // x afterAll
    // x beforeEach
    // x afterEach
    // - use Any to return anything that can be Ok-ed or () or None or panic-ed
    // - bench ? --> see what's the protocol
    //
}
