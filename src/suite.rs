use context::Context;

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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SuiteInfo {
    pub label: SuiteLabel,
    pub name: String,
}

pub struct Suite<T> {
    pub info: SuiteInfo,
    pub(crate) context: Context<T>,
}

impl<T> Suite<T> {
    pub(crate) fn new(info: SuiteInfo, context: Context<T>) -> Self {
        Suite {
            info: info,
            context: context,
        }
    }
}

unsafe impl<T> Send for Suite<T> where T: Send {}
unsafe impl<T> Sync for Suite<T> where T: Sync {}

/// This creates a test suite's root context and returns a [Runner](../runner/struct.Runner.html) ready to run the test suite.
///
/// # Examples
///
/// ```
/// # extern crate rspec;
/// #
/// # use std::io;
/// # use std::sync::{Arc, Mutex};
/// #
/// # use rspec::prelude::*;
/// #
/// # pub fn main() {
/// #     let simple = rspec::formatter::Simple::new(io::stdout());
/// #     let formatter = Arc::new(Mutex::new(simple));
/// #     let configuration = Configuration::default().parallel(false);
/// #     let runner = Runner::new(configuration, vec![formatter]);
/// #
/// runner.run_or_exit(rspec::suite("a test suite", (), |ctx| {
///     // …
/// }));
/// # }
/// ```
///
/// Corresponding console output:
///
/// ```no-run
/// running tests
/// Suite "a test suite":
///     …
/// ```
///
/// Available aliases:
///
/// - [`describe`](fn.describe.html).
/// - [`given`](fn.given.html).
pub fn suite<S, F, T>(name: S, environment: T, body: F) -> (Suite<T>, T)
    where S: Into<String>,
          F: FnOnce(&mut Context<T>) -> (),
          T: Clone + ::std::fmt::Debug
{
    let info = SuiteInfo {
        label: SuiteLabel::Suite,
        name: name.into(),
    };
    suite_internal(info, environment, body)
}

/// Alias for [`suite`](fn.suite.html), see for more info.
///
/// Available further aliases:
///
/// - [`given`](fn.describe.html).
pub fn describe<S, F, T>(name: S, environment: T, body: F) -> (Suite<T>, T)
    where S: Into<String>,
          F: FnOnce(&mut Context<T>) -> (),
          T: Clone + ::std::fmt::Debug
{
    let info = SuiteInfo {
        label: SuiteLabel::Describe,
        name: name.into(),
    };
    suite_internal(info, environment, body)
}

/// Alias for [`suite`](fn.suite.html), see for more info.
///
/// Available further aliases:
///
/// - [`describe`](fn.describe.html).
pub fn given<S, F, T>(name: S, environment: T, body: F) -> (Suite<T>, T)
    where S: Into<String>,
          F: FnOnce(&mut Context<T>) -> (),
          T: Clone + ::std::fmt::Debug
{
    let info = SuiteInfo {
        label: SuiteLabel::Given,
        name: name.into(),
    };
    suite_internal(info, environment, body)
}

fn suite_internal<'a, F, T>(info: SuiteInfo, environment: T, body: F) -> (Suite<T>, T)
    where F: FnOnce(&mut Context<T>) -> (),
          T: Clone + ::std::fmt::Debug
{
    // Note: root context's info get's ignored.
    let mut ctx = Context::new(None);
    body(&mut ctx);
    (Suite::new(info, ctx), environment)
}
