//! Events are sent by the Runner to signal the progression in the test suite, with the results

use header::{SuiteHeader, ContextHeader, ExampleHeader};
use report::{SuiteReport, ContextReport, ExampleReport};

/// `RunnerObserver`s can be attached to a [`Runner`](../runner/struct.Runner.html) to observe a
#[allow(unused_variables)]
pub trait RunnerObserver: Send + Sync {
    fn enter_suite(&self, header: &SuiteHeader) {}
    fn exit_suite(&self, header: &SuiteHeader, report: &SuiteReport) {}
    fn enter_context(&self, header: &ContextHeader) {}
    fn exit_context(&self, header: &ContextHeader, report: &ContextReport) {}
    fn enter_example(&self, header: &ExampleHeader) {}
    fn exit_example(&self, header: &ExampleHeader, report: &ExampleReport) {}
}

#[cfg(test)]
mod tests {
    // Nothing to test here, yet.
}
