use std::io;

use header::{SuiteHeader, ContextHeader, ExampleHeader};
use report::{BlockReport, SuiteReport, ContextReport, ExampleReport};
use runner::RunnerObserver;
use logger::SerialLogger;

/// Preferred logger for parallel test suite execution
/// (see [`Configuration.parallel`](struct.Configuration.html#fields)).
pub struct ParallelLogger<T: io::Write> {
    serial: SerialLogger<T>,
}

impl<T: io::Write> ParallelLogger<T>
where
    T: Send + Sync,
{
    pub fn new(buffer: T) -> ParallelLogger<T> {
        ParallelLogger { serial: SerialLogger::new(buffer) }
    }

    fn replay_suite(&self, suite: &SuiteHeader, report: &SuiteReport) {
        self.serial.enter_suite(suite);
        self.replay_context(None, report.get_context());
        self.serial.exit_suite(suite, report);
    }

    fn replay_block(&self, report: &BlockReport) {
        match report {
            &BlockReport::Context(ref header, ref report) => {
                self.replay_context(header.as_ref(), report);
            }
            &BlockReport::Example(ref header, ref report) => {
                self.replay_example(header, report);
            }
        }
    }

    fn replay_context(&self, context: Option<&ContextHeader>, report: &ContextReport) {
        if let Some(header) = context {
            self.serial.enter_context(header);
        }
        for report in report.get_blocks() {
            self.replay_block(report);
        }
        if let Some(header) = context {
            self.serial.exit_context(header, report);
        }
    }

    fn replay_example(&self, example: &ExampleHeader, report: &ExampleReport) {
        self.serial.enter_example(example);
        self.serial.exit_example(example, report);
    }
}

impl<T: io::Write> RunnerObserver for ParallelLogger<T>
where
    T: Send + Sync,
{
    fn exit_suite(&self, header: &SuiteHeader, report: &SuiteReport) {
        self.replay_suite(header, report);
    }
}
