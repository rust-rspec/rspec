use runner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    StartRunner,
    FinishedRunner(runner::RunnerResult),
    // {Start,End}Describe
    StartTest(String), /* {Start,End}Test
                        * {Start,End}Before
                        * {Start,End}After */
}

pub trait EventHandler {
    fn trigger(&mut self, event: &Event);
}
