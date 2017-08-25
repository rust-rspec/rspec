use std::fmt;

use header::Header;

/// A [`Suite`](../block/struct.Suite.html)'s cosmetic label.
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

/// A [`Header`](trait.Header.html) with label and name of a [`Suite`](../block/struct.Suite.html).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SuiteHeader {
    pub label: SuiteLabel,
    pub name: &'static str,
}

impl SuiteHeader {
    pub fn new(label: SuiteLabel, name: &'static str) -> Self {
        SuiteHeader {
            label: label,
            name: name,
        }
    }
}

impl Header for SuiteHeader {
    fn label(&self) -> &str {
        self.label.into()
    }

    fn name(&self) -> &str {
        &self.name[..]
    }
}

impl fmt::Display for SuiteHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let label: &str = self.label.into();
        write!(f, "{} {:?}", label, self.name)?;

        Ok(())
    }
}
