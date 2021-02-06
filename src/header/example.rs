use std::fmt;

/// How the [`Example`](../block/struct.Example.html) will be printed by the [`Logger`](../logger/index.html).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExampleLabel {
    It,
    Example,
    Then,
}

impl fmt::Display for ExampleLabel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExampleLabel::It => write!(f, "It"),
            ExampleLabel::Example => write!(f, "Example"),
            ExampleLabel::Then => write!(f, "Then"),
        }
    }
}

/// A [`Header`](trait.Header.html) with label and name of an [`Example`](../block/struct.Example.html).
#[derive(Clone, PartialEq, Eq, Debug, new)]
pub struct ExampleHeader {
    pub label: ExampleLabel,
    pub name: &'static str,
}

#[cfg(test)]
impl Default for ExampleHeader {
    /// Used for testing
    fn default() -> Self {
        ExampleHeader::new(ExampleLabel::It, "example")
    }
}

impl fmt::Display for ExampleHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:?}", self.label, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_fmt() {
        fn subject(label: ExampleLabel) -> String {
            format!("{}", label)
        };
        assert_eq!(subject(ExampleLabel::Example), "Example".to_owned());
        assert_eq!(subject(ExampleLabel::It), "It".to_owned());
        assert_eq!(subject(ExampleLabel::Then), "Then".to_owned());
    }

    #[test]
    fn header_fmt() {
        fn subject(label: ExampleLabel) -> String {
            format!("{}", ExampleHeader::new(label, "Test"))
        };
        assert_eq!(
            subject(ExampleLabel::Example),
            "Example \"Test\"".to_owned()
        );
        assert_eq!(subject(ExampleLabel::It), "It \"Test\"".to_owned());
        assert_eq!(subject(ExampleLabel::Then), "Then \"Test\"".to_owned());
    }
}
