use time::Duration;

use header::SuiteHeader;
use report::{Report, ContextReport};

/// `SuiteReport` holds the results of a context suite's test execution.
#[derive(PartialEq, Eq, Clone, Debug, new)]
pub struct SuiteReport {
    header: SuiteHeader,
    context: ContextReport,
}

impl SuiteReport {
    pub fn get_header(&self) -> &SuiteHeader {
        &self.header
    }

    pub fn get_context(&self) -> &ContextReport {
        &self.context
    }
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

    fn get_duration(&self) -> Duration {
        self.context.get_duration()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
