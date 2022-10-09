use crate::header::ExampleHeader;
use crate::report::ExampleResult;

/// Test examples are the smallest unit of a testing framework, wrapping one or more assertions.
pub struct Example<T> {
    pub(crate) header: ExampleHeader,
    pub(crate) function: Box<dyn Fn(&T) -> ExampleResult>,
}

impl<T> Example<T> {
    pub(crate) fn new<F>(header: ExampleHeader, assertion: F) -> Self
    where
        F: 'static + Fn(&T) -> ExampleResult,
    {
        Example {
            header,
            function: Box::new(assertion),
        }
    }

    /// Used for testing purpose
    #[cfg(test)]
    pub fn fixture_success() -> Self {
        Example::new(ExampleHeader::default(), |_| ExampleResult::Success)
    }

    /// Used for testing purpose
    #[cfg(test)]
    pub fn fixture_ignored() -> Self {
        Example::new(ExampleHeader::default(), |_| ExampleResult::Ignored)
    }

    /// Used for testing purpose
    #[cfg(test)]
    pub fn fixture_failed() -> Self {
        Example::new(ExampleHeader::default(), |_| ExampleResult::Failure(None))
    }
}
