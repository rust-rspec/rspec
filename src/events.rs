//! Events are send by the Runner to signal the progression in the test suite, with the results

use context;
use runner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    StartRunner,
    FinishedRunner(runner::RunnerResult),
    StartDescribe(String),
    EndDescribe,
    StartTest(String),
    EndTest(context::TestResult), /* {Start,End}Before
                                   * {Start,End}After */
}

pub trait EventHandler {
    fn trigger(&mut self, event: &Event);
}
