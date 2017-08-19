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
//! let simple = rspec::formatter::Simple::new(io::stdout());
//! let formatter = Arc::new(Mutex::new(simple));
//!
//! #[derive(Clone, Debug)]
//! struct Environment {
//!     // ...
//! }
//!
//! let environment = Environment {
//!     // ...
//! };
//!
//! let mut runner = rspec::given("Some title", environment, |ctx| {
//!     // ...
//! });
//!
//! runner.add_event_handler(formatter);
//! runner.run_or_exit();
//! ```

pub mod formatter;
pub mod simple;

/// The Simple formatter is similar to the default Rspec formatter
pub use formatter::simple::Simple;
