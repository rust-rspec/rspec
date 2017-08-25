//! The formatter module expose different formatters that can be attached to the Runner and display
//! the progression of the run under various forms
//!
//! # Examples
//!
//! ```
//! extern crate rspec;
//!
//! use std::io;
//! use std::sync::Arc;
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
//!     let formatter = Arc::new(rspec::Formatter::new(io::stdout()));
//!     let configuration = rspec::ConfigurationBuilder::default().build().unwrap();
//!     let runner = rspec::Runner::new(configuration, vec![formatter]);
//!
//!     runner.run(rspec::suite("a test suite", environment, |ctx| {
//!         ctx.context("opens a context labeled 'context'", |_ctx| {
//!             // …
//!         });
//!     }));
//! }
//! ```

pub mod serial;
pub mod parallel;

/// The Simple formatter is similar to the default Rspec formatter
// pub use formatter::serial::Formatter;
pub use formatter::parallel::Formatter;
