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
//! let stdout = &mut io::stdout();
//! let mut formatter = rspec::formatter::Simple::new(stdout);
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
//! runner.add_event_handler(&mut formatter);
//! runner.run_or_exit();
//! ```

pub mod formatter;
pub mod simple;

/// The Simple formatter is similar to the default Rspec formatter
pub use formatter::simple::Simple;
