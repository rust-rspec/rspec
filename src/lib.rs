#![allow(dead_code)]

#[macro_use(expect)]
extern crate expectest;
pub use expectest::prelude::*;

mod events;
mod context;
mod runner;

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
     * - check that runner displays the tests names and their results
     * x check that we can use before in a describe
     * x check that we can use after in a describe
     * - check that after/before are run in all child contextes
     * - beforeAll
     * - afterAll
     * - bench ?
     */
}
