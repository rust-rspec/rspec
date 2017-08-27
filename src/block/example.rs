use report::ExampleReport;
use header::ExampleHeader;

/// Test examples are the smallest unit of a testing framework, wrapping one or more assertions.
pub struct Example<T> {
    pub(crate) header: ExampleHeader,
    pub(crate) function: Box<Fn(&T) -> ExampleReport>,
}

impl<T> Example<T> {
    pub(crate) fn new<F>(header: ExampleHeader, assertion: F) -> Self
    where
        F: 'static + Fn(&T) -> ExampleReport,
    {
        Example {
            header: header,
            function: Box::new(assertion),
        }
    }
}

unsafe impl<T> Send for Example<T>
where
    T: Send,
{
}
unsafe impl<T> Sync for Example<T>
where
    T: Sync,
{
}
