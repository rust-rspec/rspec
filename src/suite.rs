use context::Context;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SuiteLabel {
    Suite,
    Describe,
    Given,
}

impl From<SuiteLabel> for &'static str {
    fn from(label: SuiteLabel) -> Self {
        match label {
            SuiteLabel::Suite => "Suite",
            SuiteLabel::Describe => "Describe",
            SuiteLabel::Given => "Given",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SuiteInfo {
    pub label: SuiteLabel,
    pub name: String,
}

pub struct Suite<'a, T>
    where T: 'a
{
    pub info: SuiteInfo,
    pub(crate) context: Context<'a, T>,
}

impl<'a, T> Suite<'a, T>
    where T: 'a
{
    pub(crate) fn new(info: SuiteInfo, context: Context<'a, T>) -> Self {
        Suite {
            info: info,
            context: context,
        }
    }
}

unsafe impl<'a, T> Send for Suite<'a, T> where T: 'a + Send {}
unsafe impl<'a, T> Sync for Suite<'a, T> where T: 'a + Sync {}
