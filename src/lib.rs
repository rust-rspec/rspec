#![doc(html_root_url = "https://mackwic.github.io/rspec")]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![allow(dead_code)]

#[cfg(feature = "use_expectest")]
extern crate expectest;

extern crate colored;
extern crate rayon;

pub mod suite;
pub mod context;
pub mod example;

pub use context::{describe, suite, given};

mod context_member;
pub mod context_report;
pub mod example_report;
pub mod events;
pub mod runner;
pub mod formatter;
pub mod visitor;

#[cfg(test)]
mod tests {
    pub use super::*;
    pub use context::*;

    // Test list:
    // x check that tests can call `assert_eq!`
    // x check that tests can return Err or Ok
    // x runner can count the tests
    // x runner can count the success and failed
    // x check that we can use before in a describe
    // x check that we can use after in a describe
    // x check that after/before are run in all child contextes
    // x runner broadcasts run events
    // x progress formatter is an event handler
    // x pluggable formatters via formatter trait
    // - stats time events is an event handler
    // - detect slow tests via treshold
    // - time the total running time
    // - failure-only via a tmp file
    // - filter tests
    // - coloration
    // - seed for deterministic randomization
    // - fail-fast fail at the first failed test
    // x beforeAll
    // x afterAll
    // - beforeEach ?
    // - afterEach ?
    // - use Any to return anything that can be Ok-ed or () or None or panic-ed
    // - bench ? --> see what's the protocol
    //
}
