//! The Context module holds all the functionality for the test declaration, that is:
//! `before`, `after`, `suite`, `context`, `it` and their variants.
//!
//! A Context can also holds reference to children Contextes, for whom the before closures will be
//! executed after the before closures of the current context. The order of execution of tests
//! respect the order of declaration of theses tests.
//!
//! Running these tests and doing asserts is not the job of the Context, but the Runner, which is
//! a struct returned by the root context declaration.
//!

pub mod context;
pub mod context_member;
pub mod example;

#[cfg(test)]
mod context_test;

pub use self::context::*;
pub use self::context_member::*;
pub use self::example::*;

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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ContextInfo {
    pub label: ContextLabel,
    pub name: &'static str,
}
