use crate::block::Context;
use crate::header::{SuiteHeader, SuiteLabel};

/// Test suites bundle a set of closely related test examples into a logical execution group.
#[derive(new)]
pub struct Suite<T> {
    pub(crate) header: SuiteHeader,
    pub(crate) environment: T,
    pub(crate) context: Context<T>,
}

impl<T> Suite<T> {
    pub fn num_blocks(&self) -> usize {
        self.context.num_blocks()
    }

    pub fn num_examples(&self) -> usize {
        self.context.num_examples()
    }

    pub fn is_empty(&self) -> bool {
        self.context.is_empty()
    }
}

/// Creates a test suite from a given root context.
///
/// # Examples
///
/// ```
/// # extern crate rspec;
/// #
/// # use std::io;
/// # use std::sync::Arc;
/// #
/// # pub fn main() {
/// #     let logger = Arc::new(rspec::Logger::new(io::stdout()));
/// #     let configuration = rspec::ConfigurationBuilder::default().build().unwrap();
/// #     let runner = rspec::Runner::new(configuration, vec![logger]);
/// #
/// runner.run(&rspec::suite("a test suite", (), |_ctx| {
///     // …
/// }));
/// # }
/// ```
///
/// Corresponding console output:
///
/// ```text
/// tests
/// Suite "a test suite":
///     …
/// ```
///
/// Available aliases:
///
/// - [`describe`](fn.describe.html).
/// - [`given`](fn.given.html).
pub fn suite<F, T>(name: &'static str, environment: T, body: F) -> Suite<T>
where
    F: FnOnce(&mut Context<T>),
    T: Clone,
{
    let header = SuiteHeader {
        label: SuiteLabel::Suite,
        name,
    };
    suite_internal(header, environment, body)
}

/// Alias for [`suite`](fn.suite.html), see for more info.
///
/// Available further aliases:
///
/// - [`given`](fn.describe.html).
pub fn describe<F, T>(name: &'static str, environment: T, body: F) -> Suite<T>
where
    F: FnOnce(&mut Context<T>),
    T: Clone,
{
    let header = SuiteHeader {
        label: SuiteLabel::Describe,
        name,
    };
    suite_internal(header, environment, body)
}

/// Alias for [`suite`](fn.suite.html), see for more info.
///
/// Available further aliases:
///
/// - [`describe`](fn.describe.html).
pub fn given<F, T>(name: &'static str, environment: T, body: F) -> Suite<T>
where
    F: FnOnce(&mut Context<T>),
    T: Clone,
{
    let header = SuiteHeader {
        label: SuiteLabel::Given,
        name,
    };
    suite_internal(header, environment, body)
}

fn suite_internal<F, T>(header: SuiteHeader, environment: T, body: F) -> Suite<T>
where
    F: FnOnce(&mut Context<T>),
    T: Clone,
{
    let mut ctx = Context::new(None);
    body(&mut ctx);
    Suite::new(header, environment, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_suite() {
        let suite = suite("name", (), |_| {});
        assert_eq!(suite.header.label, SuiteLabel::Suite);
        assert_eq!(suite.header.name, "name");
        assert_eq!(suite.environment, ());
        assert_eq!(suite.is_empty(), true);
        assert_eq!(suite.num_examples(), 0);
    }

    #[test]
    fn empty_describe() {
        let describe = describe("name", (), |_| {});
        assert_eq!(describe.header.label, SuiteLabel::Describe);
        assert_eq!(describe.header.name, "name");
        assert_eq!(describe.environment, ());
        assert_eq!(describe.is_empty(), true);
        assert_eq!(describe.num_examples(), 0);
    }

    #[test]
    fn empty_given() {
        let given = given("name", (), |_| {});
        assert_eq!(given.header.label, SuiteLabel::Given);
        assert_eq!(given.header.name, "name");
        assert_eq!(given.environment, ());
        assert_eq!(given.is_empty(), true);
        assert_eq!(given.num_examples(), 0);
    }

    #[test]
    fn non_empty_suite() {
        let suite = suite("suite", (), |ctx| {
            ctx.context("context", |_| {});
        });
        assert_eq!(suite.header.label, SuiteLabel::Suite);
        assert_eq!(suite.header.name, "suite");
        assert_eq!(suite.environment, ());
        assert_eq!(suite.is_empty(), false);
        assert_eq!(suite.num_examples(), 0);
    }
}
