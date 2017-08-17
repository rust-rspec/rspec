use std::convert::From;

#[cfg(feature = "use_expectest")]
use expectest::core::TestResult as ExpectestResult;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Failure {
    pub message: Option<String>,
}

impl Failure {
    pub fn new<S>(message: Option<S>) -> Self
        where String: From<S>
    {
        Failure {
            message: message.map(|s| s.into())
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExampleReport {
    Success,
    Ignored,
    Failure(Failure),
}

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct ExampleReport(Result<(), String>);

impl ExampleReport {
    pub fn err<S>(message: Option<S>) -> Self
        where String: From<S>
    {
        ExampleReport::Failure(Failure::new(message))
    }

    pub fn is_ok(&self) -> bool {
        *self == ExampleReport::Success
    }

    pub fn is_err(&self) -> bool {
        *self != ExampleReport::Success
    }
}

impl From<()> for ExampleReport {
    fn from(_other: ()) -> ExampleReport {
        ExampleReport::Success
    }
}

impl From<bool> for ExampleReport {
    fn from(other: bool) -> ExampleReport {
        if other {
            ExampleReport::Success
        } else {
            ExampleReport::Failure(
                Failure::new(Some("assertion failed: `expected true`"))
            )
        }
    }
}

impl<T1, T2> From<Result<T1, T2>> for ExampleReport
    where T2: ::std::fmt::Debug
{
    fn from(other: Result<T1, T2>) -> ExampleReport {
        match other {
            Ok(_) => ExampleReport::Success,
            Err(error) => ExampleReport::Failure(
                Failure::new(Some(format!("{:?}", error)))
            )
        }
    }
}

#[cfg(feature = "use_expectest")]
impl From<ExpectestResult> for ExampleReport {
    fn from(other: ExpectestResult) -> ExampleReport {
        match other {
            ExpectestResult::Success => ExampleReport::Success,
            ExpectestResult::Failure(failure) => {
                ExampleReport::Failure(
                    Failure::new(Some(format!("{:?}", failure)))
                )
            },
        }
    }
}
