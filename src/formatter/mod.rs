//! The formatter module expose different formatters that can be attached to the Runner and display
//! the progression of the run under various forms
//!
//! # Examples
//!
//! ```
//! use rspec::context::describe;
//! use rspec::formatter::Simple;
//! use std::io;
//!
//! let mut stdout = io::stdout();
//! let mut formatter = Simple::new(&mut stdout);
//!
//! let mut runner = describe("a test suite", |_| {});
//! runner.add_event_handler(&mut formatter);
//! // use the formatter
//! runner.run().unwrap();
//! ```

pub mod formatter;
pub mod simple;

/// The Simple formatter is similar to the default Rspec formatter
pub use formatter::simple::Simple;
