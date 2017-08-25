use std::fmt;

use header::Header;

/// A [`Context`](../block/struct.Context.html)'s cosmetic label.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContextLabel {
    Describe,
    Context,
    Specify,
    Given,
    When,
}

impl From<ContextLabel> for &'static str {
    fn from(label: ContextLabel) -> Self {
        match label {
            ContextLabel::Describe => "Describe",
            ContextLabel::Context => "Context",
            ContextLabel::Specify => "Specify",
            ContextLabel::Given => "Given",
            ContextLabel::When => "When",
        }
    }
}

/// A [`Header`](trait.Header.html) with label and name of a [`Context`](../block/struct.Context.html).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ContextHeader {
    pub label: ContextLabel,
    pub name: &'static str,
}

impl ContextHeader {
    pub fn new(label: ContextLabel, name: &'static str) -> Self {
        ContextHeader {
            label: label,
            name: name,
        }
    }
}

impl Header for ContextHeader {
    fn label(&self) -> &str {
        self.label.into()
    }

    fn name(&self) -> &str {
        &self.name[..]
    }
}

impl fmt::Display for ContextHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let label: &str = self.label.into();
        write!(f, "{} {:?}", label, self.name)?;

        Ok(())
    }
}
