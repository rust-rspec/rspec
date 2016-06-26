use runner;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Event {
    StartRunner,
    FinishedRunner(runner::RunnerResult),
}

pub trait EventHandler {
    fn trigger(&mut self, event: Event);
}
