//! The Context module holds all the functionality for the test declaration, that is:
//! `before`, `after`, `suite`, `context`, `it` and their variants.
//!
//! A Context can also holds reference to children Contextes, for whom the before closures will be
//! executed after the before closures of the current context. The order of execution of tests
//! respect the order of declaration of theses tests.
//!
//! Running these tests and doing asserts is not the job of the Context, but the Runner, which is
//! a struct returned by the root context declaration.

use std::panic::{catch_unwind, AssertUnwindSafe};

use context_member::ContextMember;
use example_report::{ExampleReport, Failure};
use runner::Runner;
use suite::{Suite, SuiteInfo, SuiteLabel};
use example::{Example, ExampleInfo, ExampleLabel};

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
    pub name: String,
}

/// Contexts are a powerful method to make your tests clear and well organized.
/// In the long term this practice will keep tests easy to read.
pub struct Context<'a, T>
    where T: 'a
{
    pub(crate) info: Option<ContextInfo>,
    pub(crate) members: Vec<ContextMember<'a, T>>,
    pub(crate) before_all: Vec<Box<Fn(&mut T) + 'a>>,
    pub(crate) before_each: Vec<Box<Fn(&mut T) + 'a>>,
    pub(crate) after_all: Vec<Box<Fn(&mut T) + 'a>>,
    pub(crate) after_each: Vec<Box<Fn(&mut T) + 'a>>,
}

impl<'a, T> Context<'a, T>
    where T: 'a
{
    pub(crate) fn new(info: Option<ContextInfo>) -> Self {
        Context {
            info: info,
            members: vec![],
            before_all: vec![],
            before_each: vec![],
            after_all: vec![],
            after_each: vec![],
        }
    }
}

unsafe impl<'a, T> Send for Context<'a, T> where T: 'a + Send {}
unsafe impl<'a, T> Sync for Context<'a, T> where T: 'a + Sync {}

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
/// # pub fn main() {
/// #     let simple = rspec::formatter::Simple::new(io::stdout());
/// #     let formatter = Arc::new(Mutex::new(simple));
/// #
/// let mut runner = rspec::suite("a test suite", (), |ctx| {
///     // …
/// });
/// #
/// #     runner.add_event_handler(formatter);
/// #     runner.run_or_exit();
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
pub fn suite<'a, 'b, S, F, T>(name: S, environment: T, body: F) -> Runner<'a, T>
    where S: Into<String>,
          F: 'a + FnOnce(&mut Context<'a, T>) -> (),
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
pub fn describe<'a, 'b, S, F, T>(name: S, environment: T, body: F) -> Runner<'a, T>
    where S: Into<String>,
          F: 'a + FnOnce(&mut Context<'a, T>) -> (),
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
pub fn given<'a, 'b, S, F, T>(name: S, environment: T, body: F) -> Runner<'a, T>
    where S: Into<String>,
          F: 'a + FnOnce(&mut Context<'a, T>) -> (),
          T: Clone + ::std::fmt::Debug
{
    let info = SuiteInfo {
        label: SuiteLabel::Given,
        name: name.into(),
    };
    suite_internal(info, environment, body)
}

fn suite_internal<'a, 'b, F, T>(info: SuiteInfo, environment: T, body: F) -> Runner<'a, T>
    where F: 'a + FnOnce(&mut Context<'a, T>) -> (),
          T: Clone + ::std::fmt::Debug
{
    // Note: root context's info get's ignored.
    let mut ctx = Context::new(None);
    body(&mut ctx);
    let suite = Suite::new(info, ctx);
    Runner::new(suite, environment)
}

impl<'a, T> Context<'a, T>
    where T: Clone
{
    /// Open and name a new context within the current context.
    ///
    /// Note that the order of execution is not guaranteed to match the declaration order.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate rspec;
    /// #
    /// # use std::io;
    /// # use std::sync::{Arc, Mutex};
    /// #
    /// # pub fn main() {
    /// #     let simple = rspec::formatter::Simple::new(io::stdout());
    /// #     let formatter = Arc::new(Mutex::new(simple));
    /// #
    /// let mut runner = rspec::suite("a test suite", (), |ctx| {
    ///     ctx.context("opens a context labeled 'context'", |ctx| {
    ///         // …
    ///     });
    /// });
    /// #
    /// #     runner.add_event_handler(formatter);
    /// #     runner.run_or_exit();
    /// # }
    /// ```
    ///
    /// Corresponding console output:
    ///
    /// ```no-run
    /// running tests
    /// Suite "a test suite":
    ///     Context "opens a sub-context":
    ///         …
    /// ```
    ///
    /// Available aliases:
    ///
    /// - [`specify`](struct.Context.html#method.specify).
    /// - [`when`](struct.Context.html#method.when).
    pub fn context<'b, S, F>(&mut self, name: S, body: F)
        where S: Into<Option<&'b str>>,
              F: 'a + FnOnce(&mut Context<'a, T>) -> (),
              T: ::std::fmt::Debug
    {
        let info = name.into().map(|name| {
            ContextInfo {
                label: ContextLabel::Context,
                name: name.to_owned(),
            }
        });
        self.context_internal(info, body)
    }

    /// Alias for [`context`](struct.Context.html#method.context), see for more info.
    ///
    /// Available further aliases:
    ///
    /// - [`when`](struct.Context.html#method.when).
    pub fn specify<'b, S, F>(&mut self, name: S, body: F)
        where S: Into<Option<&'b str>>,
              F: 'a + FnOnce(&mut Context<'a, T>) -> (),
              T: ::std::fmt::Debug
    {
        let info = name.into().map(|name| {
            ContextInfo {
                label: ContextLabel::Specify,
                name: name.to_owned(),
            }
        });
        self.context_internal(info, body)
    }

    /// Alias for [`context`](struct.Context.html#method.context), see for more info.
    ///
    /// Available further aliases:
    ///
    /// - [`specify`](struct.Context.html#method.specify).
    pub fn when<'b, S, F>(&mut self, name: S, body: F)
        where S: Into<Option<&'b str>>,
              F: 'a + FnOnce(&mut Context<'a, T>) -> (),
              T: ::std::fmt::Debug
    {
        let info = name.into().map(|name| {
            ContextInfo {
                label: ContextLabel::When,
                name: name.to_owned(),
            }
        });
        self.context_internal(info, body)
    }

    /// Open a new name-less context within the current context which won't show up in the logs.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate rspec;
    /// #
    /// # use std::io;
    /// # use std::sync::{Arc, Mutex};
    /// #
    /// # pub fn main() {
    /// #     let simple = rspec::formatter::Simple::new(io::stdout());
    /// #     let formatter = Arc::new(Mutex::new(simple));
    /// #
    /// let mut runner = rspec::suite("a suite",(), |ctx| {
    ///     ctx.context("a context", |ctx| {
    ///         ctx.scope(|ctx| {
    ///             ctx.example("an example", |env| {
    ///                 // …
    ///             });
    ///         });
    ///     });
    /// });
    /// #
    /// #     runner.add_event_handler(formatter);
    /// #     runner.run_or_exit();
    /// # }
    /// ```
    ///
    /// Corresponding console output:
    ///
    /// ```no-run
    /// running tests
    /// Suite "a suite":
    ///     Context "a context":
    ///         Example "an example"
    /// ```
    ///
    /// The `before_each(…)` block gets executed before `'It "tests a"'` and `'It "tests a"'`,
    /// but not before `'It "tests c"'`.
    pub fn scope<F>(&mut self, body: F)
        where F: 'a + FnOnce(&mut Context<'a, T>) -> (),
              T: ::std::fmt::Debug
    {
        self.context_internal(None, body)
    }

    fn context_internal<F>(&mut self, info: Option<ContextInfo>, body: F)
        where F: 'a + FnOnce(&mut Context<'a, T>) -> (),
              T: ::std::fmt::Debug
    {
        let mut child = Context::new(info);
        body(&mut child);
        self.members.push(ContextMember::Context(child))
    }

    /// Open and name a new example within the current context.
    ///
    /// Note that the order of execution is not guaranteed to match the declaration order.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate rspec;
    /// #
    /// # use std::io;
    /// # use std::sync::{Arc, Mutex};
    /// #
    /// # pub fn main() {
    /// #     let simple = rspec::formatter::Simple::new(io::stdout());
    /// #     let formatter = Arc::new(Mutex::new(simple));
    /// #
    /// let mut runner = rspec::suite("a test suite", (), |ctx| {
    ///     ctx.example("an example", |env| {
    ///         // …
    ///     });
    /// });
    /// #
    /// #     runner.add_event_handler(formatter);
    /// #     runner.run_or_exit();
    /// # }
    /// ```
    ///
    /// Corresponding console output:
    ///
    /// ```no-run
    /// running tests
    /// Suite "a test suite":
    ///     Example "an example":
    ///         …
    /// ```
    ///
    /// Available aliases:
    ///
    /// - [`it`](struct.Context.html#method.it).
    /// - [`then`](struct.Context.html#method.then).
    pub fn example<S, F, U>(&mut self, name: S, body: F)
        where S: Into<String>,
              F: 'a + Fn(&T) -> U,
              U: Into<ExampleReport>
    {
        let info = ExampleInfo {
            label: ExampleLabel::Example,
            name: name.into(),
            failure: None,
        };
        self.example_internal(info, body)
    }

    /// Alias for [`example`](struct.Context.html#method.example), see for more info.
    ///
    /// Available further aliases:
    ///
    /// - [`it`](struct.Context.html#method.it).
    pub fn it<S, F, U>(&mut self, name: S, body: F)
        where S: Into<String>,
              F: 'a + Fn(&T) -> U,
              U: Into<ExampleReport>
    {
        let info = ExampleInfo {
            label: ExampleLabel::It,
            name: name.into(),
            failure: None,
        };
        self.example_internal(info, body)
    }

    /// Alias for [`example`](struct.Context.html#method.example), see for more info.
    ///
    /// Available further aliases:
    ///
    /// - [`it`](struct.Context.html#method.it).
    pub fn then<S, F, U>(&mut self, name: S, body: F)
        where S: Into<String>,
              F: 'a + Fn(&T) -> U,
              U: Into<ExampleReport>
    {
        let info = ExampleInfo {
            label: ExampleLabel::Then,
            name: name.into(),
            failure: None,
        };
        self.example_internal(info, body)
    }

    fn example_internal<F, U>(&mut self, info: ExampleInfo, body: F)
        where F: 'a + Fn(&T) -> U,
              U: Into<ExampleReport>
    {
        let test = Example::new(info, move |environment| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                body(&environment).into()
            }));
            match result {
                Ok(result) => result,
                Err(error) => {
                    use std::borrow::Cow;
                    let error_as_str = error.downcast_ref::<&str>().map(|s| Cow::from(*s) );
                    let error_as_string = error.downcast_ref::<String>().map(|s| Cow::from(s.clone()) );
                    let message = error_as_str.or(error_as_string).map(|cow| {
                        let message = cow.to_string();
                        format!("thread panicked at '{:?}'.", message)
                    });
                    ExampleReport::Failure(Failure::new(message))
                },
            }
        });
        self.members.push(ContextMember::Example(test))
    }

    /// Declares a closure that will be executed once before any
    /// of the context's context or example members are being executed.
    ///
    /// Note that the order of execution is guaranteed to match the declaration order.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate rspec;
    /// #
    /// # use std::io;
    /// # use std::sync::{Arc, Mutex};
    /// #
    /// # pub fn main() {
    /// #     let simple = rspec::formatter::Simple::new(io::stdout());
    /// #     let formatter = Arc::new(Mutex::new(simple));
    /// #
    /// let mut runner = rspec::suite("a test suite", (), |ctx| {
    ///     ctx.before_all(|env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("an example", |env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("another example", |env| {
    ///         // …
    ///     });
    /// });
    /// #
    /// #     runner.add_event_handler(formatter);
    /// #     runner.run_or_exit();
    /// # }
    /// ```
    ///
    /// Corresponding console output:
    ///
    /// ```no-run
    /// running tests
    /// Suite "a test suite":
    ///     Example "an example":
    ///         …
    ///     Example "another example":
    ///         …
    /// ```
    ///
    /// Available aliases:
    ///
    /// - [`before`](struct.Context.html#method.before).
    pub fn before_all<F>(&mut self, body: F)
        where F: 'a + Fn(&mut T)
    {
        self.before_all.push(Box::new(body))
    }

    /// Alias for [`before_all`](struct.Context.html#method.before_all), see for more info.
    pub fn before<F>(&mut self, body: F)
        where F: 'a + Fn(&mut T)
    {
        self.before_all(body)
    }

    /// Declares a closure that will be executed once before each
    /// of the context's context or example members.
    ///
    /// Note that the order of execution is guaranteed to match the declaration order.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate rspec;
    /// #
    /// # use std::io;
    /// # use std::sync::{Arc, Mutex};
    /// #
    /// # pub fn main() {
    /// #     let simple = rspec::formatter::Simple::new(io::stdout());
    /// #     let formatter = Arc::new(Mutex::new(simple));
    /// #
    /// let mut runner = rspec::suite("a test suite", (), |ctx| {
    ///     ctx.before_each(|env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("an example", |env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("another example", |env| {
    ///         // …
    ///     });
    /// });
    /// #
    /// #     runner.add_event_handler(formatter);
    /// #     runner.run_or_exit();
    /// # }
    /// ```
    ///
    /// Corresponding console output:
    ///
    /// ```no-run
    /// running tests
    /// Suite "a test suite":
    ///     Example "an example":
    ///         …
    ///     Example "another example":
    ///         …
    /// ```
    pub fn before_each<F>(&mut self, body: F)
        where F: 'a + Fn(&mut T)
    {
        self.before_each.push(Box::new(body))
    }

    /// Declares a closure that will be executed once after any
    /// of the context's context or example members have been executed.
    ///
    /// Note that the order of execution is guaranteed to match the declaration order.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate rspec;
    /// #
    /// # use std::io;
    /// # use std::sync::{Arc, Mutex};
    /// #
    /// # pub fn main() {
    /// #     let simple = rspec::formatter::Simple::new(io::stdout());
    /// #     let formatter = Arc::new(Mutex::new(simple));
    /// #
    /// let mut runner = rspec::suite("a test suite", (), |ctx| {
    ///     ctx.after_all(|env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("an example", |env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("another example", |env| {
    ///         // …
    ///     });
    /// });
    /// #
    /// #     runner.add_event_handler(formatter);
    /// #     runner.run_or_exit();
    /// # }
    /// ```
    ///
    /// Corresponding console output:
    ///
    /// ```no-run
    /// running tests
    /// Suite "a test suite":
    ///     Example "an example":
    ///         …
    ///     Example "another example":
    ///         …
    /// ```
    ///
    /// Available aliases:
    ///
    /// - [`after`](struct.Context.html#method.after).
    pub fn after_all<F>(&mut self, body: F)
        where F: 'a + Fn(&mut T)
    {
        self.after_all.push(Box::new(body))
    }

    /// Alias for [`after_all`](struct.Context.html#method.after_all), see for more info.
    pub fn after<F>(&mut self, body: F)
        where F: 'a + Fn(&mut T)
    {
        self.after_all(body)
    }

    /// Declares a closure that will be executed once after each
    /// of the context's context or example members.
    ///
    /// Note that the order of execution is guaranteed to match the declaration order.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate rspec;
    /// #
    /// # use std::io;
    /// # use std::sync::{Arc, Mutex};
    /// #
    /// # pub fn main() {
    /// #     let simple = rspec::formatter::Simple::new(io::stdout());
    /// #     let formatter = Arc::new(Mutex::new(simple));
    /// #
    /// let mut runner = rspec::suite("a test suite", (), |ctx| {
    ///     ctx.after_each(|env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("an example", |env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("another example", |env| {
    ///         // …
    ///     });
    /// });
    /// #
    /// #     runner.add_event_handler(formatter);
    /// #     runner.run_or_exit();
    /// # }
    /// ```
    ///
    /// Corresponding console output:
    ///
    /// ```no-run
    /// running tests
    /// Suite "a test suite":
    ///     Example "an example":
    ///         …
    ///     Example "another example":
    ///         …
    /// ```
    pub fn after_each<F>(&mut self, body: F)
        where F: 'a + Fn(&mut T)
    {
        self.after_each.push(Box::new(body))
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;
    pub use example_report::ExampleReport;

    mod describe {
        pub use super::*;

        macro_rules! test_suite_alias {
            ($suite: ident) => {
                $suite("suite (or alias)", (), |_| {});
            }
        }

        #[test]
        fn it_has_root_functions() {
            test_suite_alias!(suite);
            test_suite_alias!(describe);
            test_suite_alias!(given);
        }

        macro_rules! test_context_alias {
            ($suite: ident, $context: ident) => {
                $suite("suite (or alias)", (), |ctx| {
                    ctx.$context("context (or alias)", |_| {})
                });
            }
        }

        #[test]
        fn it_has_contextual_function_context() {
            test_context_alias!(suite, context);
            test_context_alias!(describe, context);
            test_context_alias!(given, context);
        }

        #[test]
        fn it_has_contexual_function_specify() {
            test_context_alias!(suite, specify);
            test_context_alias!(describe, specify);
            test_context_alias!(given, specify);
        }

        #[test]
        fn it_has_contexual_function_when() {
            test_context_alias!(suite, when);
            test_context_alias!(describe, when);
            test_context_alias!(given, when);
        }

        macro_rules! test_example_alias {
            ($suite: ident, $context: ident, $example: ident) => {
                $suite("suite (or alias)", (), |ctx| {
                    ctx.$context("context (or alias)", |ctx| {
                        ctx.$example("example (or alias)", |_| {

                        })
                    })
                });
            }
        }

        #[test]
        fn it_has_check_function_example() {
            test_example_alias!(suite, context, example);
            test_example_alias!(suite, specify, example);
            test_example_alias!(suite, when, example);

            test_example_alias!(describe, context, example);
            test_example_alias!(describe, specify, example);
            test_example_alias!(describe, when, example);

            test_example_alias!(given, context, example);
            test_example_alias!(given, specify, example);
            test_example_alias!(given, when, example);
        }

        #[test]
        fn it_has_check_function_it() {
            test_example_alias!(suite, context, it);
            test_example_alias!(suite, specify, it);
            test_example_alias!(suite, when, it);

            test_example_alias!(describe, context, it);
            test_example_alias!(describe, specify, it);
            test_example_alias!(describe, when, it);

            test_example_alias!(given, context, it);
            test_example_alias!(given, specify, it);
            test_example_alias!(given, when, it);
        }

        #[test]
        fn it_has_check_function_then() {
            test_example_alias!(suite, context, then);
            test_example_alias!(suite, specify, then);
            test_example_alias!(suite, when, then);

            test_example_alias!(describe, context, then);
            test_example_alias!(describe, specify, then);
            test_example_alias!(describe, when, then);

            test_example_alias!(given, context, then);
            test_example_alias!(given, specify, then);
            test_example_alias!(given, when, then);
        }
    }
}
