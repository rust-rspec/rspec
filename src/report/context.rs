use time::Duration;
use report::{Report, BlockReport};

/// `ContextReport` holds the results of a context's test execution.
#[derive(PartialEq, Eq, Clone, Debug, new)]
pub struct ContextReport {
    sub_reports: Vec<BlockReport>,
    duration: Duration,
}

impl ContextReport {
    pub fn get_blocks(&self) -> &[BlockReport] {
        &self.sub_reports[..]
    }
}

impl Report for ContextReport {
    fn is_success(&self) -> bool {
        self.sub_reports.iter().fold(true, |success, report| {
            success & report.is_success()
        })
    }

    fn is_failure(&self) -> bool {
        self.sub_reports.iter().fold(false, |failure, report| {
            failure | report.is_failure()
        })
    }

    fn get_passed(&self) -> u32 {
        self.sub_reports.iter().fold(0, |count, report| {
            count + report.get_passed()
        })
    }

    fn get_failed(&self) -> u32 {
        self.sub_reports.iter().fold(0, |count, report| {
            count + report.get_failed()
        })
    }

    fn get_ignored(&self) -> u32 {
        self.sub_reports.iter().fold(0, |count, report| {
            count + report.get_ignored()
        })
    }

    fn get_duration(&self) -> Duration {
        self.duration
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
