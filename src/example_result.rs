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
pub enum ExampleResult {
    Success,
    Failure(Failure),
}

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct ExampleResult(Result<(), String>);

impl ExampleResult {
    pub fn err<S>(message: Option<S>) -> Self
        where String: From<S>
    {
        ExampleResult::Failure(Failure::new(message))
    }

    pub fn is_ok(&self) -> bool {
        *self == ExampleResult::Success
    }

    pub fn is_err(&self) -> bool {
        *self != ExampleResult::Success
    }
}

impl From<()> for ExampleResult {
    fn from(_other: ()) -> ExampleResult {
        ExampleResult::Success
    }
}

impl From<bool> for ExampleResult {
    fn from(other: bool) -> ExampleResult {
        if other {
            ExampleResult::Success
        } else {
            ExampleResult::Failure(
                Failure::new(Some("assertion failed: `expected true`"))
            )
        }
    }
}

impl<T1, T2> From<Result<T1, T2>> for ExampleResult
    where T2: ::std::fmt::Debug
{
    fn from(other: Result<T1, T2>) -> ExampleResult {
        match other {
            Ok(_) => ExampleResult::Success,
            Err(error) => ExampleResult::Failure(
                Failure::new(Some(format!("{:?}", error)))
            )
        }
    }
}

#[cfg(feature = "use_expectest")]
impl From<ExpectestResult> for ExampleResult {
    fn from(other: ExpectestResult) -> ExampleResult {
        match other {
            ExpectestResult::Success => ExampleResult::Success,
            ExpectestResult::Failure(failure) => {
                ExampleResult::Failure(
                    Failure::new(Some(format!("{:?}", failure)))
                )
            },
        }
    }
}
