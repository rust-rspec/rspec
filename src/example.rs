use example_report::ExampleReport;

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

pub struct Example<'a, T>
    where T: 'a
{
    pub(crate) info: ExampleInfo,
    pub(crate) function: Box<Fn(&T) -> ExampleReport + 'a>,
}

impl<'a, T> Example<'a, T>
    where T: 'a
{
    pub(crate) fn new<F>(info: ExampleInfo, f: F) -> Self
        where F: Fn(&T) -> ExampleReport + 'a
    {
        Example {
            info: info,
            function: Box::new(f),
        }
    }
}

unsafe impl<'a, T> Send for Example<'a, T> where T: 'a + Send {}
unsafe impl<'a, T> Sync for Example<'a, T> where T: 'a + Sync {}
