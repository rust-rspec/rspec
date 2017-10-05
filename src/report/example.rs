use std::convert::From;

use time::Duration;

use report::Report;

#[cfg(feature = "expectest_compat")]
use expectest::core::TestResult as ExpectestResult;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExampleResult {
    Success,
    Failure(Option<String>),
    Ignored,
}

impl ExampleResult {
    fn is_success(&self) -> bool {
        if &ExampleResult::Success == self {
            true
        } else {
            false
        }
    }

    fn is_failure(&self) -> bool {
        if let &ExampleResult::Failure(_) = self {
            true
        } else {
            false
        }
    }

    fn get_passed(&self) -> u32 {
        if &ExampleResult::Success == self {
            1
        } else {
            0
        }
    }

    fn get_failed(&self) -> u32 {
        if let &ExampleResult::Failure(_) = self {
            1
        } else {
            0
        }
    }

    fn get_ignored(&self) -> u32 {
        if &ExampleResult::Ignored == self {
            1
        } else {
            0
        }
    }
}

/// rspec considers examples returning `()` a success.
impl From<()> for ExampleResult {
    fn from(_other: ()) -> ExampleResult {
        ExampleResult::Success
    }
}

/// rspec considers examples returning `true` a success, `false` a failure.
impl From<bool> for ExampleResult {
    fn from(other: bool) -> ExampleResult {
        if other {
            ExampleResult::Success
        } else {
            ExampleResult::Failure(Some(
                "assertion failed: `expected condition to be true`"
                    .to_owned(),
            ))
        }
    }
}

/// rspec considers examples returning `Result::Ok(…)` a success, `Result::Err(…)` a failure.
impl<T1, T2> From<Result<T1, T2>> for ExampleResult
where
    T2: ::std::fmt::Debug,
{
    fn from(other: Result<T1, T2>) -> ExampleResult {
        match other {
            Ok(_) => ExampleResult::Success,
            Err(error) => ExampleResult::Failure(Some(format!("{:?}", error))),
        }
    }
}

/// rspec considers examples returning `ExpectestResult::Ok(…)` a success, `ExpectestResult::Err(…)` a failure.
#[cfg(feature = "expectest_compat")]
impl From<ExpectestResult> for ExampleResult {
    fn from(other: ExpectestResult) -> ExampleResult {
        match other {
            ExpectestResult::Success => ExampleResult::Success,
            ExpectestResult::Failure(failure) => {
                ExampleResult::Failure(Some(format!("{:?}", failure)))
            }
        }
    }
}

/// `ExampleReport` holds the results of a context example's test execution.
#[derive(Clone, PartialEq, Eq, Debug, new)]
pub struct ExampleReport {
    result: ExampleResult,
    duration: Duration,
}

impl ExampleReport {
    pub fn get_result(&self) -> &ExampleResult {
        &self.result
    }
}

impl Report for ExampleReport {
    fn is_success(&self) -> bool {
        self.result.is_success()
    }

    fn is_failure(&self) -> bool {
        self.result.is_failure()
    }

    fn get_passed(&self) -> u32 {
        self.result.get_passed()
    }

    fn get_failed(&self) -> u32 {
        self.result.get_failed()
    }

    fn get_ignored(&self) -> u32 {
        self.result.get_ignored()
    }

    fn get_duration(&self) -> Duration {
        self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_void() {
        assert!(ExampleResult::from(()).is_success());
    }

    #[test]
    fn from_bool() {
        assert!(ExampleResult::from(true).is_success());
        assert!(ExampleResult::from(false).is_failure());
    }

    #[test]
    fn from_result() {
        let ok_result: Result<(), ()> = Ok(());
        let err_result: Result<(), ()> = Err(());
        assert!(ExampleResult::from(ok_result).is_success());
        assert!(ExampleResult::from(err_result).is_failure());
    }

    #[cfg(feature = "expectest_compat")]
    #[test]
    #[should_panic]
    fn from_expectest_result() {
        let ok_result = ExpectestResult::new_success();
        // A failure ExpectestResult panics on drop, hence the `#[should_panic]`.
        let err_result = ExpectestResult::new_failure("dummy".to_owned(), None);
        assert!(ExampleResult::from(ok_result).is_success());
        assert!(ExampleResult::from(err_result).is_failure());
    }
}
