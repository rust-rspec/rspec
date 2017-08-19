use report::example::ExampleReport;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExampleLabel {
    It,
    Example,
    Then,
}

impl From<ExampleLabel> for &'static str {
    fn from(label: ExampleLabel) -> Self {
        match label {
            ExampleLabel::It => "It",
            ExampleLabel::Example => "Example",
            ExampleLabel::Then => "Then",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExampleInfo {
    pub label: ExampleLabel,
    pub name: String,
    pub failure: Option<String>,
}

pub struct Example<T> {
    pub(crate) info: ExampleInfo,
    pub(crate) function: Box<Fn(&T) -> ExampleReport>,
}

impl<T> Example<T> {
    pub(crate) fn new<F>(info: ExampleInfo, f: F) -> Self
        where F: 'static + Fn(&T) -> ExampleReport
    {
        Example {
            info: info,
            function: Box::new(f),
        }
    }
}

unsafe impl<T> Send for Example<T> where T: Send {}
unsafe impl<T> Sync for Example<T> where T: Sync {}
