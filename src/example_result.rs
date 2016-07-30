use std::convert::From;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ExampleResult(Result<(), ()>);

impl ExampleResult {
    pub fn res(&self) -> Result<(), ()> {
        self.0
    }

    pub fn is_err(&self) -> bool { self.0.is_err() }
    pub fn is_ok(&self) -> bool { self.0.is_ok() }

    pub fn or<T: Into<ExampleResult>>(self, other: T) -> ExampleResult {
        match self.0 {
            Ok(_) => other.into(),
            Err(_) => self
        }
    }
}

pub static SUCCESS_RES : ExampleResult = ExampleResult(Ok(()));
pub static FAILED_RES : ExampleResult = ExampleResult(Err(()));

impl From<()> for ExampleResult {
    fn from(_other: ()) -> ExampleResult {
        SUCCESS_RES
    }
}

impl From<bool> for ExampleResult {
    fn from(other: bool) -> ExampleResult {
        if other {
            SUCCESS_RES
        } else {
            FAILED_RES
        }
    }
}

impl<T1,T2> From<Result<T1,T2>> for ExampleResult {
    fn from(other: Result<T1,T2>) -> ExampleResult {
        if other.is_ok() {
            SUCCESS_RES
        } else {
            FAILED_RES
        }
    }
}

