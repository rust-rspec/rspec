//! Events are send by the Runner to signal the progression in the test suite, with the results

use report::suite::SuiteReport;
use report::context::ContextReport;
use report::example::ExampleReport;

use suite::SuiteInfo;
use context::ContextInfo;
use example::ExampleInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    EnterSuite(SuiteInfo),
    ExitSuite(SuiteReport),
    EnterContext(ContextInfo),
    ExitContext(ContextReport),
    EnterExample(ExampleInfo),
    ExitExample(ExampleReport),
}

pub trait EventHandler: Send + Sync {
    fn handle(&mut self, event: &Event);
}
