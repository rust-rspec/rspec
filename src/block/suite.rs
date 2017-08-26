use header::{SuiteLabel, SuiteHeader};
use block::Context;

/// Test suites bundle a set of closely related test examples into a logical execution group.
pub struct Suite<T> {
    pub(crate) header: SuiteHeader,
    pub(crate) environment: T,
    pub(crate) context: Context<T>,
}

impl<T> Suite<T> {
    pub(crate) fn new(header: SuiteHeader, environment: T, context: Context<T>) -> Self {
        Suite {
            header: header,
            environment: environment,
            context: context,
        }
    }

    pub fn num_examples(&self) -> usize {
        self.context.num_examples()
    }
}

unsafe impl<T> Send for Suite<T>
where
    T: Send,
{
}
unsafe impl<T> Sync for Suite<T>
where
    T: Sync,
{
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
/// #     let formatter = Arc::new(rspec::Formatter::new(io::stdout()));
/// #     let configuration = rspec::ConfigurationBuilder::default().build().unwrap();
/// #     let runner = rspec::Runner::new(configuration, vec![formatter]);
/// #
/// runner.run(rspec::suite("a test suite", (), |_ctx| {
///     // …
/// }));
/// # }
/// ```
///
/// Corresponding console output:
///
/// ```no-run
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
    F: FnOnce(&mut Context<T>) -> (),
    T: Clone + ::std::fmt::Debug,
{
    let header = SuiteHeader {
        label: SuiteLabel::Suite,
        name: name,
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
    F: FnOnce(&mut Context<T>) -> (),
    T: Clone + ::std::fmt::Debug,
{
    let header = SuiteHeader {
        label: SuiteLabel::Describe,
        name: name,
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
    F: FnOnce(&mut Context<T>) -> (),
    T: Clone + ::std::fmt::Debug,
{
    let header = SuiteHeader {
        label: SuiteLabel::Given,
        name: name,
    };
    suite_internal(header, environment, body)
}

fn suite_internal<'a, F, T>(header: SuiteHeader, environment: T, body: F) -> Suite<T>
where
    F: FnOnce(&mut Context<T>) -> (),
    T: Clone + ::std::fmt::Debug,
{
    let mut ctx = Context::new(None);
    body(&mut ctx);
    Suite::new(header, environment, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {

        let suite = suite("name", (), |_| {});

        assert_eq!(suite.header.label, SuiteLabel::Suite);
        assert_eq!(suite.header.name, "name");
        assert_eq!(suite.environment, ());
    }
}
