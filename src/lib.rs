#![allow(dead_code)]

#[macro_use(expect)]
extern crate expectest;
pub use expectest::prelude::*;

mod events;
mod context;
mod runner;
mod formatter;

#[cfg(test)]
mod tests {
    pub use super::*;
    pub use context::*;

    /*
     * Test list:
     * x check that tests can call `assert_eq!`
     * x check that tests can return Err or Ok
     * x runner can count the tests
     * x runner can count the success and failures
     * x check that we can use before in a describe
     * x check that we can use after in a describe
     * x check that after/before are run in all child contextes
     * [] runner broadcasts run events
     * - progress formatter is an event handler
     * - pluggable formatters via formatter trait
     * - stats time events is an event handler
     * - beforeAll
     * - afterAll
     * - bench ? --> see what's the protocol
     */
}
