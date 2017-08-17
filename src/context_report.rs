use example_report::ExampleReport;

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
