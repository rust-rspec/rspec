//! Events are send by the Runner to signal the progression in the test suite, with the results

use example_report::ExampleReport;
use context_report::ContextReport;

use suite::SuiteInfo;
use context::ContextInfo;
use example::ExampleInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    EnterSuite(SuiteInfo),
    ExitSuite(ContextReport),
    EnterContext(ContextInfo),
    ExitContext(ContextReport),
    EnterExample(ExampleInfo),
    ExitExample(ExampleReport),
}

pub trait EventHandler: Send + Sync {
    fn handle(&mut self, event: &Event);
}
