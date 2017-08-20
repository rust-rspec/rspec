//! The formatter module expose different formatters that can be attached to the Runner and display
//! the progression of the run under various forms
//!
//! # Examples
//!
//! ```
//! extern crate rspec;
//!
//! use std::io;
//! use std::sync::{Arc, Mutex};
//!
//! use rspec::prelude::*;
//!
//! pub fn main() {
//!     #[derive(Clone, Debug)]
//!     struct Environment {
//!         // ...
//!     }
//!
//!     let environment = Environment {
//!         // ...
//!     };
//!     let simple = rspec::formatter::Simple::new(io::stdout());
//!     let formatter = Arc::new(Mutex::new(simple));
//!     let configuration = Configuration::default().parallel(false);
//!     let runner = Runner::new(configuration, vec![formatter]);
//!
//!     runner.run_or_exit(rspec::suite("a test suite", environment, |ctx| {
//!         ctx.context("opens a context labeled 'context'", |ctx| {
//!             // …
//!         });
//!     }));
//! }
//! ```

pub mod formatter;
pub mod simple;

/// The Simple formatter is similar to the default Rspec formatter
pub use formatter::simple::Simple;
