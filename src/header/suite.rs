use std::fmt;

/// A [`Suite`](../block/struct.Suite.html)'s cosmetic label.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SuiteLabel {
    Suite,
    Describe,
    Given,
}

impl fmt::Display for SuiteLabel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &SuiteLabel::Suite => write!(f, "Suite"),
            &SuiteLabel::Describe => write!(f, "Describe"),
            &SuiteLabel::Given => write!(f, "Given"),
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

impl fmt::Display for SuiteHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:?}", self.label, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_fmt() {
        fn subject(label: SuiteLabel) -> String {
            format!("{}", label)
        };
        assert_eq!(subject(SuiteLabel::Suite), "Suite".to_owned());
        assert_eq!(subject(SuiteLabel::Describe), "Describe".to_owned());
        assert_eq!(subject(SuiteLabel::Given), "Given".to_owned());
    }

    #[test]
    fn header_fmt() {
        fn subject(label: SuiteLabel) -> String {
            format!("{}", SuiteHeader::new(label, "Test"))
        };
        assert_eq!(subject(SuiteLabel::Suite), "Suite \"Test\"".to_owned());
        assert_eq!(
            subject(SuiteLabel::Describe),
            "Describe \"Test\"".to_owned()
        );
        assert_eq!(subject(SuiteLabel::Given), "Given \"Test\"".to_owned());
    }
}
