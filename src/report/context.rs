use report::{Report, BlockReport};

/// The runner assembles a `ContextReport` for each context during test execution.
#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct ContextReport {
    sub_reports: Vec<BlockReport>,
}

impl ContextReport {
    pub fn new(sub_reports: Vec<BlockReport>) -> Self {
        ContextReport {
            sub_reports: sub_reports,
        }
    }

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
}

#[cfg(test)]
mod tests {
    // use super::*;
}
