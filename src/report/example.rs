use std::convert::From;

use report::Report;

#[cfg(feature = "expectest_compat")]
use expectest::core::TestResult as ExpectestResult;

/// `ExampleReport` holds the results of a context example's test execution.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExampleReport {
    Success,
    Failure(Option<String>),
    Ignored,
}

impl Report for ExampleReport {
    fn is_success(&self) -> bool {
        self == &ExampleReport::Success
    }

    fn is_failure(&self) -> bool {
        self != &ExampleReport::Success
    }

    fn get_passed(&self) -> u32 {
        if &ExampleReport::Success == self {
            1
        } else {
            0
        }
    }

    fn get_failed(&self) -> u32 {
        if let &ExampleReport::Failure(_) = self {
            1
        } else {
            0
        }
    }

    fn get_ignored(&self) -> u32 {
        if &ExampleReport::Ignored == self {
            1
        } else {
            0
        }
    }
}

/// rspec considers examples returning `()` a success.
impl From<()> for ExampleReport {
    fn from(_other: ()) -> ExampleReport {
        ExampleReport::Success
    }
}

/// rspec considers examples returning `true` a success, `false` a failure.
impl From<bool> for ExampleReport {
    fn from(other: bool) -> ExampleReport {
        if other {
            ExampleReport::Success
        } else {
            ExampleReport::Failure(Some(
                "assertion failed: `expected condition to be true`"
                    .to_owned(),
            ))
        }
    }
}

/// rspec considers examples returning `Result::Ok(…)` a success, `Result::Err(…)` a failure.
impl<T1, T2> From<Result<T1, T2>> for ExampleReport
where
    T2: ::std::fmt::Debug,
{
    fn from(other: Result<T1, T2>) -> ExampleReport {
        match other {
            Ok(_) => ExampleReport::Success,
            Err(error) => ExampleReport::Failure(Some(format!("{:?}", error))),
        }
    }
}

/// rspec considers examples returning `ExpectestResult::Ok(…)` a success, `ExpectestResult::Err(…)` a failure.
#[cfg(feature = "expectest_compat")]
impl From<ExpectestResult> for ExampleReport {
    fn from(other: ExpectestResult) -> ExampleReport {
        match other {
            ExpectestResult::Success => ExampleReport::Success,
            ExpectestResult::Failure(failure) => {
                ExampleReport::Failure(Some(format!("{:?}", failure)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_void() {
        assert!(ExampleReport::from(()).is_success());
    }

    #[test]
    fn from_bool() {
        assert!(ExampleReport::from(true).is_success());
        assert!(ExampleReport::from(false).is_failure());
    }

    #[test]
    fn from_result() {
        let ok_result: Result<(), ()> = Ok(());
        let err_result: Result<(), ()> = Err(());
        assert!(ExampleReport::from(ok_result).is_success());
        assert!(ExampleReport::from(err_result).is_failure());
    }

    #[cfg(feature = "expectest_compat")]
    #[test]
    #[should_panic]
    fn from_expectest_result() {
        let ok_result = ExpectestResult::new_success();
        // A failure ExpectestResult panics on drop, hence the `#[should_panic]`.
        let err_result = ExpectestResult::new_failure("dummy".to_owned(), None);
        assert!(ExampleReport::from(ok_result).is_success());
        assert!(ExampleReport::from(err_result).is_failure());
    }
}
