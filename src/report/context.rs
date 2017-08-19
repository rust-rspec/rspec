use std::ops::{Add, AddAssign};
use std::iter::Sum;

use report::example::ExampleReport;

#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct ContextReport {
    pub passed: u32,
    pub failed: u32,
    pub ignored: u32,
    pub measured: u32,
}

impl ContextReport {
    pub fn new(passed: u32, failed: u32) -> Self {
        ContextReport {
            passed: passed,
            failed: failed,
            ignored: 0,
            measured: 0,
        }
    }

    pub fn add<T>(&mut self, report: T)
        where T: Into<ContextReport>
    {
        let report: ContextReport = report.into();
        self.passed += report.passed;
        self.failed += report.failed;
        self.ignored += report.ignored;
        self.measured += report.measured;
    }
}

impl From<ExampleReport> for ContextReport {
    fn from(result: ExampleReport) -> Self {
        let (passed, failed) = if result.is_ok() { (1, 0) } else { (0, 1) };
        ContextReport {
            passed: passed,
            failed: failed,
            ignored: 0,
            measured: 0,
        }
    }
}

impl Add<Self> for ContextReport {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        ContextReport {
            passed: self.passed + other.passed,
            failed: self.failed + other.failed,
            ignored: self.ignored + other.ignored,
            measured: self.measured + other.measured,
        }
    }
}

impl AddAssign<Self> for ContextReport {
    fn add_assign(&mut self, rhs: Self) {
        self.passed += rhs.passed;
        self.failed += rhs.failed;
        self.ignored += rhs.ignored;
        self.measured += rhs.measured;
    }
}

impl Sum<Self> for ContextReport {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut report = ContextReport::default();
        for sub_report in iter {
            report += sub_report;
        }
        report
    }
}
