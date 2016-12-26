//! Events are send by the Runner to signal the progression in the test suite, with the results

use example_result::ExampleResult;
use runner::TestReport;
use context::{SuiteInfo, ContextInfo, TestInfo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    EnterSuite(SuiteInfo),
    ExitSuite(TestReport),
    EnterContext(ContextInfo),
    ExitContext(TestReport),
    EnterTest(TestInfo),
    ExitTest(ExampleResult),
}

pub trait EventHandler {
    fn trigger(&mut self, event: &Event);
}
