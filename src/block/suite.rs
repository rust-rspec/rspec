use header::suite::{SuiteLabel, SuiteHeader};
use block::context::Context;

pub struct Suite<T> {
    pub header: SuiteHeader,
    pub(crate) context: Context<T>,
}

impl<T> Suite<T> {
    pub(crate) fn new(header: SuiteHeader, context: Context<T>) -> Self {
        Suite {
            header: header,
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

/// This creates a test suite's root context and returns a [Runner](../runner/struct.Runner.html) ready to run the test suite.
///
/// # Examples
///
/// ```
/// # extern crate rspec;
/// #
/// # use std::io;
/// # use std::sync::Arc;
/// #
/// # use rspec::prelude::*;
/// #
/// # pub fn main() {
/// #     let formatter = Arc::new(rspec::Formatter::new(io::stdout()));
/// #     let configuration = rspec::Configuration::default();
/// #     let runner = rspec::Runner::new(configuration, vec![formatter]);
/// #
/// runner.run(rspec::suite("a test suite", (), |_ctx| {
///     // …
/// })).or_exit();
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
pub fn suite<F, T>(name: &'static str, environment: T, body: F) -> (Suite<T>, T)
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

/// Alias for [`suite`](fn.suite.html), see for more header.
///
/// Available further aliases:
///
/// - [`given`](fn.describe.html).
pub fn describe<F, T>(name: &'static str, environment: T, body: F) -> (Suite<T>, T)
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

/// Alias for [`suite`](fn.suite.html), see for more header.
///
/// Available further aliases:
///
/// - [`describe`](fn.describe.html).
pub fn given<F, T>(name: &'static str, environment: T, body: F) -> (Suite<T>, T)
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

fn suite_internal<'a, F, T>(header: SuiteHeader, environment: T, body: F) -> (Suite<T>, T)
where
    F: FnOnce(&mut Context<T>) -> (),
    T: Clone + ::std::fmt::Debug,
{    let mut ctx = Context::new(None);
    body(&mut ctx);
    (Suite::new(header, ctx), environment)
}
