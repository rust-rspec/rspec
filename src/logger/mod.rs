//! Loggers can be attached to the Runner to log the progression and results of a test suite.
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
//! let logger = Arc::new(rspec::Logger::new(io::stdout()));
//! let configuration = rspec::ConfigurationBuilder::default().build().unwrap();
//! let runner = rspec::Runner::new(configuration, vec![logger]);
//! # }
//! ```

mod serial;
mod parallel;

pub use logger::serial::*;
pub use logger::parallel::*;

pub use logger::parallel::ParallelLogger as Logger;
