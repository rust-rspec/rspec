use std::fmt;

/// How the [`Context`](../block/struct.Context.html) will be printed by the [`Logger`](../logger/index.html).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContextLabel {
    Context,
    Specify,
    When,
}

impl fmt::Display for ContextLabel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContextLabel::Context => write!(f, "Context"),
            ContextLabel::Specify => write!(f, "Specify"),
            ContextLabel::When => write!(f, "When"),
        }
    }
}

/// A [`Header`](trait.Header.html) with label and name of a [`Context`](../block/struct.Context.html).
#[derive(Clone, PartialEq, Eq, Debug, new)]
pub struct ContextHeader {
    pub label: ContextLabel,
    pub name: &'static str,
}

impl fmt::Display for ContextHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:?}", self.label, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_fmt() {
        fn subject(label: ContextLabel) -> String {
            format!("{}", label)
        }
        assert_eq!(subject(ContextLabel::Context), "Context".to_owned());
        assert_eq!(subject(ContextLabel::Specify), "Specify".to_owned());
        assert_eq!(subject(ContextLabel::When), "When".to_owned());
    }

    #[test]
    fn header_fmt() {
        fn subject(label: ContextLabel) -> String {
            format!("{}", ContextHeader::new(label, "Test"))
        }
        assert_eq!(
            subject(ContextLabel::Context),
            "Context \"Test\"".to_owned()
        );
        assert_eq!(
            subject(ContextLabel::Specify),
            "Specify \"Test\"".to_owned()
        );
        assert_eq!(subject(ContextLabel::When), "When \"Test\"".to_owned());
    }
}
