use runner;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Event {
    StartRunner,
    FinishedRunner(runner::RunnerResult),
    // {Start,End}Describe
    // {Start,End}Test
    // {Start,End}Before
    // {Start,End}After
}

pub trait EventHandler {
    fn trigger(&mut self, event: Event);
}
