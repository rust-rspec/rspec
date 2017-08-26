//! Events are sent by the Runner to signal the progression in the test suite, with the results

use header::SuiteHeader;
use header::ContextHeader;
use header::ExampleHeader;
use report::SuiteReport;
use report::ContextReport;
use report::ExampleReport;

/// `RunnerObserver`s can be attached to a [`Runner`](../runner/struct.Runner.html) to observe a
pub trait RunnerObserver: Send + Sync {
    fn enter_suite(&self, suite: &SuiteHeader);
    fn exit_suite(&self, suite: &SuiteHeader, report: &SuiteReport);
    fn enter_context(&self, context: &ContextHeader);
    fn exit_context(&self, context: &ContextHeader, _report: &ContextReport);
    fn enter_example(&self, example: &ExampleHeader);
    fn exit_example(&self, example: &ExampleHeader, report: &ExampleReport);
}

#[cfg(feature = "specialization")]
default impl<T> RunnerObserver for T {
    fn enter_suite(&self, suite: &SuiteHeader) {}
    fn exit_suite(&self, suite: &SuiteHeader, report: &SuiteReport) {}
    fn enter_context(&self, context: &ContextHeader) {}
    fn exit_context(&self, context: &ContextHeader, _report: &ContextReport) {}
    fn enter_example(&self, example: &ExampleHeader) {}
    fn exit_example(&self, example: &ExampleHeader, report: &ExampleReport) {}
}

#[cfg(test)]
mod tests {
    // Nothing to test here, yet.
}
