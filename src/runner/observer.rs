//! Events are sent by the Runner to signal the progression in the test suite, with the results

use header::{ContextHeader, ExampleHeader, SuiteHeader};
use report::{ContextReport, ExampleReport, SuiteReport};
use runner::Runner;

/// `RunnerObserver`s can be attached to a [`Runner`](../runner/struct.Runner.html) to observe a
#[allow(unused_variables)]
pub trait RunnerObserver: Send + Sync {
    fn enter_suite(&self, runner: &Runner, header: &SuiteHeader) {}
    fn exit_suite(&self, runner: &Runner, header: &SuiteHeader, report: &SuiteReport) {}
    fn enter_context(&self, runner: &Runner, header: &ContextHeader) {}
    fn exit_context(&self, runner: &Runner, header: &ContextHeader, report: &ContextReport) {}
    fn enter_example(&self, runner: &Runner, header: &ExampleHeader) {}
    fn exit_example(&self, runner: &Runner, header: &ExampleHeader, report: &ExampleReport) {}
}

#[cfg(test)]
mod tests {
    // Nothing to test here, yet.
}
