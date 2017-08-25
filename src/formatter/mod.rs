//! Formatters can be attached to the Runner to log the progression and results of a test suite.
//!
//! # Examples
//!
//! ```
//! # extern crate rspec;
//! #
//! # use std::io;
//! # use std::sync::Arc;
//! #
//! # pub fn main() {
//! let formatter = Arc::new(rspec::Formatter::new(io::stdout()));
//! let configuration = rspec::ConfigurationBuilder::default().build().unwrap();
//! let runner = rspec::Runner::new(configuration, vec![formatter]);
//! # }
//! ```

mod serial;
mod parallel;

pub use formatter::serial::*;
pub use formatter::parallel::*;

pub use formatter::parallel::ParallelFormatter as Formatter;
