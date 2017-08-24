use std::fmt;

use header::Header;

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
pub struct ExampleHeader {
    pub label: ExampleLabel,
    pub name: &'static str,
}

impl ExampleHeader {
    pub fn new(label: ExampleLabel, name: &'static str) -> Self {
        ExampleHeader {
            label: label,
            name: name,
        }
    }
}

impl Header for ExampleHeader {
    fn label(&self) -> &str {
        self.label.into()
    }

    fn name(&self) -> &str {
        &self.name[..]
    }
}

impl fmt::Display for ExampleHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let label: &str = self.label.into();
        write!(f, "{} {:?}", label, self.name)?;

        Ok(())
    }
}
