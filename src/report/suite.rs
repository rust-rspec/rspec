use std::process;

use header::suite::SuiteHeader;
use report::Report;
use report::context::ContextReport;

/// The runner assembles a `SuiteReport` for each context during test execution.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SuiteReport {
    header: SuiteHeader,
    context: ContextReport,
}

impl SuiteReport {
    pub fn new(header: SuiteHeader, context: ContextReport) -> Self {
        SuiteReport {
            header: header,
            context: context,
        }
    }

    pub fn get_header(&self) -> &SuiteHeader {
        &self.header
    }

    pub fn get_context(&self) -> &ContextReport {
        &self.context
    }

    pub fn or_exit(self) -> Self {
        if self.is_failure() {
            // XXX Cargo test failure returns 101.
            //
            // > "We use 101 as the standard failure exit code because it's something unique
            // > that the test runner can check for in run-fail tests (as opposed to something
            // > like 1, which everybody uses). I don't expect this behavior can ever change.
            // > This behavior probably dates to before 2013,
            // > all the way back to the creation of compiletest." – @brson

            process::exit(101);
        }
        self
    }

    // pub fn get_blocks(&self) -> &[BlockReport] {
    //     self.context.get_blocks()
    // }
}

impl Report for SuiteReport {
    fn is_success(&self) -> bool {
        self.context.is_success()
    }

    fn is_failure(&self) -> bool {
        self.context.is_failure()
    }

    fn get_passed(&self) -> u32 {
        self.context.get_passed()
    }

    fn get_failed(&self) -> u32 {
        self.context.get_failed()
    }

    fn get_ignored(&self) -> u32 {
        self.context.get_ignored()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
