use std::io;

use header::SuiteHeader;
use header::ContextHeader;
use header::ExampleHeader;
use runner::RunnerObserver;
use report::BlockReport;
use report::SuiteReport;
use report::ContextReport;
use report::ExampleReport;
use formatter::SerialFormatter;

/// Preferred formatter for parallel test suite execution
/// (see [`Configuration.parallel`](struct.Configuration.html#fields)).
pub struct ParallelFormatter<T: io::Write> {
    serial: SerialFormatter<T>,
}

impl<T: io::Write> ParallelFormatter<T>
where
    T: Send + Sync,
{
    pub fn new(buffer: T) -> ParallelFormatter<T> {
        ParallelFormatter {
            serial: SerialFormatter::new(buffer),
        }
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
            },
            &BlockReport::Example(ref header, ref report) => {
                self.replay_example(header, report);
            },
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

impl<T: io::Write> RunnerObserver for ParallelFormatter<T>
where
    T: Send + Sync,
{
    fn enter_suite(&self, _suite: &SuiteHeader) {

    }

    fn exit_suite(&self, suite: &SuiteHeader, report: &SuiteReport) {
        self.replay_suite(suite, report);
    }

    fn enter_context(&self, _context: &ContextHeader) {

    }

    fn exit_context(&self, _context: &ContextHeader, _report: &ContextReport) {

    }

    fn enter_example(&self, _example: &ExampleHeader) {

    }

    fn exit_example(&self, _example: &ExampleHeader, _report: &ExampleReport) {

    }
}
