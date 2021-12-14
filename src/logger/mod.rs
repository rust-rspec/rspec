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

use std::io;

use crate::header::{ContextHeader, ExampleHeader, SuiteHeader};
use crate::logger::serial::SerialLogger;
use crate::report::{BlockReport, ContextReport, ExampleReport, SuiteReport};
use crate::runner::{Runner, RunnerObserver};

/// Preferred logger for test suite execution.
pub struct Logger<T: io::Write> {
    serial: SerialLogger<T>,
}

impl<T: io::Write> Logger<T>
where
    T: Send + Sync,
{
    pub fn new(buffer: T) -> Logger<T> {
        Logger {
            serial: SerialLogger::new(buffer),
        }
    }

    fn replay_suite(&self, runner: &Runner, suite: &SuiteHeader, report: &SuiteReport) {
        self.serial.enter_suite(runner, suite);
        self.replay_context(runner, None, report.get_context());
        self.serial.exit_suite(runner, suite, report);
    }

    fn replay_block(&self, runner: &Runner, report: &BlockReport) {
        match report {
            BlockReport::Context(ref header, ref report) => {
                self.replay_context(runner, header.as_ref(), report);
            }
            BlockReport::Example(ref header, ref report) => {
                self.replay_example(runner, header, report);
            }
        }
    }

    fn replay_context(
        &self,
        runner: &Runner,
        context: Option<&ContextHeader>,
        report: &ContextReport,
    ) {
        if let Some(header) = context {
            self.serial.enter_context(runner, header);
        }
        for report in report.get_blocks() {
            self.replay_block(runner, report);
        }
        if let Some(header) = context {
            self.serial.exit_context(runner, header, report);
        }
    }

    fn replay_example(&self, runner: &Runner, example: &ExampleHeader, report: &ExampleReport) {
        self.serial.enter_example(runner, example);
        self.serial.exit_example(runner, example, report);
    }
}

impl<T: io::Write> RunnerObserver for Logger<T>
where
    T: Send + Sync,
{
    fn enter_suite(&self, runner: &Runner, header: &SuiteHeader) {
        self.serial.enter_suite(runner, header);
    }

    fn exit_suite(&self, runner: &Runner, header: &SuiteHeader, report: &SuiteReport) {
        self.serial.exit_suite(runner, header, report);
    }

    fn enter_context(&self, runner: &Runner, header: &ContextHeader) {
        self.serial.enter_context(runner, header);
    }

    fn exit_context(&self, runner: &Runner, header: &ContextHeader, report: &ContextReport) {
        self.serial.exit_context(runner, header, report);
    }

    fn enter_example(&self, runner: &Runner, header: &ExampleHeader) {
        self.serial.enter_example(runner, header);
    }

    fn exit_example(&self, runner: &Runner, header: &ExampleHeader, report: &ExampleReport) {
        self.serial.exit_example(runner, header, report);
    }
}
