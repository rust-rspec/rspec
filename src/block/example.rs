use report::example::ExampleReport;

use header::example::ExampleHeader;

pub struct Example<T> {
    pub(crate) header: ExampleHeader,
    pub(crate) function: Box<Fn(&T) -> ExampleReport>,
}

impl<T> Example<T> {
    pub(crate) fn new<F>(header: ExampleHeader, f: F) -> Self
    where
        F: 'static + Fn(&T) -> ExampleReport,
    {
        Example {
            header: header,
            function: Box::new(f),
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
