#![doc(html_root_url = "https://mackwic.github.io/rspec")]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![allow(dead_code)]

#[macro_use]
extern crate derive_builder;

#[cfg(feature = "expectest_compat")]
extern crate expectest;

extern crate colored;
extern crate rayon;

pub mod block;
pub mod header;
pub mod report;
pub mod runner;
pub mod formatter;

mod visitor;

pub use block::{suite, describe, given};
pub use formatter::Formatter;
pub use runner::{Configuration, ConfigurationBuilder, Runner};

/// A wrapper-macro for conveniently running a test suite with
/// the default configuration with considerebly less glue-code.
///
/// # Examples
///
/// ```
/// # #[macro_use(rspec_run)]
/// # extern crate rspec;
/// #
/// # pub fn main() {
/// #[derive(Clone, Debug)]
/// struct Environment {
///     // ...
/// }
///
/// let environment = Environment {
///     // ...
/// };
///
/// rspec_run!(given "a scenario", environment, |ctx| {
///     ctx.when("...", |ctx| {
///         // ...
///     });
///
///     ctx.then("...", |env| { /* ... */ });
/// });
/// # }
/// ```
#[macro_export]
macro_rules! rspec_run {
    ($label:ident $name:expr, $env:expr, |$ctx:ident| $block:block) => ({
        use std::io;
        use std::sync::Arc;

        let formatter = Arc::new(rspec::Formatter::new(io::stdout()));
        let configuration = rspec::ConfigurationBuilder::default().build().unwrap();
        let runner = rspec::Runner::new(configuration, vec![formatter]);

        runner.run(rspec::$label($name, $env, |$ctx| $block))
    })
}

#[cfg(test)]
mod tests {

    pub use super::*;
    pub use block::*;

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
