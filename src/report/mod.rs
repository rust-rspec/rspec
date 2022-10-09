//! Reports provide information about an evaluated test unit.

mod context;
mod example;
mod suite;

pub use time::Duration;

pub use crate::report::context::*;
pub use crate::report::example::*;
pub use crate::report::suite::*;

use crate::header::ContextHeader;
use crate::header::ExampleHeader;

/// `Report` holds the results of a structural group's test execution.
pub trait Report {
    fn is_success(&self) -> bool;
    fn is_failure(&self) -> bool;

    fn get_passed(&self) -> u32;
    fn get_failed(&self) -> u32;
    fn get_ignored(&self) -> u32;

    fn get_duration(&self) -> Duration;
}

/// `BlockReport` holds the results of a context block's test execution.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum BlockReport {
    Context(Option<ContextHeader>, context::ContextReport),
    Example(ExampleHeader, example::ExampleReport),
}

impl BlockReport {
    pub fn get_blocks(&self) -> Option<&[BlockReport]> {
        match self {
            BlockReport::Context(_, ref report) => Some(report.get_blocks()),
            BlockReport::Example(_, _) => None,
        }
    }
}

impl Report for BlockReport {
    fn is_success(&self) -> bool {
        match self {
            BlockReport::Context(_, ref report) => report.is_success(),
            BlockReport::Example(_, ref report) => report.is_success(),
        }
    }

    fn is_failure(&self) -> bool {
        match self {
            BlockReport::Context(_, ref report) => report.is_failure(),
            BlockReport::Example(_, ref report) => report.is_failure(),
        }
    }

    fn get_passed(&self) -> u32 {
        match self {
            BlockReport::Context(_, ref report) => report.get_passed(),
            BlockReport::Example(_, ref report) => report.get_passed(),
        }
    }

    fn get_failed(&self) -> u32 {
        match self {
            BlockReport::Context(_, ref report) => report.get_failed(),
            BlockReport::Example(_, ref report) => report.get_failed(),
        }
    }

    fn get_ignored(&self) -> u32 {
        match self {
            BlockReport::Context(_, ref report) => report.get_ignored(),
            BlockReport::Example(_, ref report) => report.get_ignored(),
        }
    }

    fn get_duration(&self) -> Duration {
        match self {
            BlockReport::Context(_, ref report) => report.get_duration(),
            BlockReport::Example(_, ref report) => report.get_duration(),
        }
    }
}
