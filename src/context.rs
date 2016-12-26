//! The Context module holds all the functionality for the test declaration, that is:
//! `suite`, `context`, `it` and their variants.
//!
//! Running these tests and doing asserts is not the job of the Context, but the Runner, which is
//! a struct returned by the root context declaration.
//!
//! # Examples
//! ```no_run
//! ```
//!

use std::panic::{catch_unwind, AssertUnwindSafe};

use runner::*;
use events::Event;
use example_result::{ExampleResult, Failure};

pub trait Visitable<T> {
    fn accept(&mut self, visitor: &mut T) -> TestReport;
}

/// The type used for a test result
pub type TestResult = Result<(), ()>;

pub struct Named<T>(String, T);

/// This enum is used to build a tree of named tests and contextes.
pub enum Testable<'a, T>
    where T: 'a
{
    Test(Test<'a, T>),
    Context(Context<'a, T>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TestLabel {
    It,
    Example,
    Then,
}

impl From<TestLabel> for &'static str {
    fn from(label: TestLabel) -> Self {
        match label {
            TestLabel::It => "It",
            TestLabel::Example => "Example",
            TestLabel::Then => "Then",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestInfo {
    pub label: TestLabel,
    pub name: String,
    pub failure: Option<String>,
}

pub struct Test<'a, T>
    where T: 'a
{
    pub info: TestInfo,
    environment: T,
    function: Box<FnMut(&T) -> ExampleResult + 'a>,
}

impl<'a, T> Test<'a, T>
    where T: 'a
{
    pub fn new<F>(info: TestInfo, environment: T, f: F) -> Self
        where F: FnMut(&T) -> ExampleResult + 'a
    {
        Test {
            info: info,
            environment: environment,
            function: Box::new(f),
        }
    }
}

impl<'a, T> Visitable<Runner<'a, T>> for Test<'a, T>
    where T: 'a + Clone + ::std::fmt::Debug
{
    fn accept(&mut self, runner: &mut Runner<'a, T>) -> TestReport {
        runner.broadcast(Event::EnterTest(self.info.clone()));
        let function = &mut self.function;
        let environment = &mut self.environment;
        let result = function(&environment);
        runner.broadcast(Event::ExitTest(result.clone()));
        result.into()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SuiteLabel {
    Describe,
    Suite,
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

pub struct Suite<'a, T>
    where T: 'a
{
    pub info: SuiteInfo,
    root: Context<'a, T>,
}

impl<'a, T> Suite<'a, T>
    where T: 'a
{
    pub fn new(info: SuiteInfo, root: Context<'a, T>) -> Self {
        Suite {
            info: info,
            root: root,
        }
    }
}

impl<'a, T> Visitable<Runner<'a, T>> for Suite<'a, T>
    where T: 'a + Clone + ::std::fmt::Debug
{
    fn accept(&mut self, runner: &mut Runner<'a, T>) -> TestReport {
        runner.broadcast(Event::EnterSuite(self.info.clone()));
        let report = self.root.accept(runner);
        runner.broadcast(Event::ExitSuite(report.clone()));
        report
    }
}

/// A Context holds a collection of tests, a collection of closures to call _before_ running any
/// tests, and a collection of closure to call _after_ all the tests..
///
/// This is effectively the struct we fill when calling `ctx.it()`
pub struct Context<'a, T>
    where T: 'a
{
    info: ContextInfo,
    environment: T,
    testables: Vec<Testable<'a, T>>,
}

impl<'a, T> Context<'a, T>
    where T: 'a
{
    pub fn new(info: ContextInfo, environment: T) -> Self {
        Context {
            info: info,
            environment: environment,
            testables: vec![],
        }
    }
}

impl<'a, T> Visitable<Runner<'a, T>> for Context<'a, T>
    where T: 'a + Clone + ::std::fmt::Debug
{
    fn accept(&mut self, runner: &mut Runner<'a, T>) -> TestReport {
        let mut report = TestReport::default();
        runner.broadcast(Event::EnterContext(self.info.clone()));
        for testable in self.testables.iter_mut() {
            let test_res = {
                let result = match testable {
                    &mut Testable::Test(ref mut test) => {
                        test.environment = self.environment.clone();
                        let result = test.accept(runner);
                        result.into()
                    }
                    &mut Testable::Context(ref mut ctx) => {
                        ctx.environment = self.environment.clone();
                        let report = ctx.accept(runner);
                        report
                    }
                };
                result
            };
            report.add(test_res);
        }
        runner.broadcast(Event::ExitContext(report.clone()));
        report
    }
}

impl<'a, T> Context<'a, T>
    where T: Clone
{
    //    /// Open and name a new example group, which will be keeped as a child context of the current
    //    /// context.
    //    ///
    //    /// Note that the order of declaration is respected for running the tests.
    //    ///
    //    /// # Examples
    //    ///
    //    /// ```no_run
    //    /// use rspec;
    //    ///
    //    /// // `describe` instanciates a runner with a test suite and runs it transparently:
    //    /// rspec::describe("inside this describe, we use the context", (), |ctx| {
    //    ///
    //    ///     ctx.it("should run first", |_| Ok(()) as Result<(),()>);
    //    ///
    //    ///     ctx.describe("open describe", (), |ctx| {
    //    ///
    //    ///         ctx.it("should run second", |_| Ok(()) as Result<(),()>);
    //    ///
    //    ///         ctx.describe("in a describe", (), |ctx| {
    //    ///
    //    ///             ctx.describe("in a describe", (), |_| {});
    //    ///
    //    ///             ctx.it("should run third", |_| Ok(()) as Result<(),()>);
    //    ///
    //    ///         });
    //    ///     });
    //    ///
    //    ///     ctx.it("should run last", |_| Ok(()) as Result<(),()>);
    //    /// });
    //    /// ```
    pub fn context<S, F>(&mut self, name: S, body: F)
        where S: Into<String>,
              F: 'a + FnOnce(&mut Context<'a, T>) -> (),
              T: ::std::fmt::Debug
    {
        let info = ContextInfo {
            label: ContextLabel::Context,
            name: name.into(),
        };
        self.context_internal(info, body)
    }

    /// Alias for [`context`](struct.Context.html#method.context).
    ///
    /// See [`context`](struct.Context.html#method.context) for more info.
    pub fn specify<S, F>(&mut self, name: S, body: F)
        where S: Into<String>,
              F: 'a + FnOnce(&mut Context<'a, T>) -> (),
              T: ::std::fmt::Debug
    {
        let info = ContextInfo {
            label: ContextLabel::Specify,
            name: name.into(),
        };
        self.context_internal(info, body)
    }

    /// Alias for [`context`](struct.Context.html#method.context).
    ///
    /// See [`context`](struct.Context.html#method.context) for more info.
    pub fn when<S, F>(&mut self, name: S, body: F)
        where S: Into<String>,
              F: 'a + FnOnce(&mut Context<'a, T>) -> (),
              T: ::std::fmt::Debug
    {
        let info = ContextInfo {
            label: ContextLabel::When,
            name: name.into(),
        };
        self.context_internal(info, body)
    }

    fn context_internal<F>(&mut self, info: ContextInfo, body: F)
        where F: 'a + FnOnce(&mut Context<'a, T>) -> (),
              T: ::std::fmt::Debug
    {
        let environment = self.environment.clone();
        let mut child = Context::new(info, environment);
        body(&mut child);
        self.testables.push(Testable::Context(child))
    }

    //    /// Register and name a closure as an example
    //    ///
    //    /// # Examples
    //    ///
    //    /// ```no_run
    //    /// use rspec::context::rdescribe;
    //    ///
    //    /// // `rdescribe` instanciate a runner and run it transparently
    //    /// rdescribe("inside this describe, we use the context", (), |ctx| {
    //    ///
    //    ///     ctx.it("test at the root", || Ok(()) as Result<(),()>);
    //    ///
    //    ///     ctx.describe("a group of examples", (), |ctx| {
    //    ///
    //    ///         ctx.it("should be usable inside a describe", || Ok(()) as Result<(),()>);
    //    ///
    //    ///         ctx.describe("a nested describe", (), |ctx| {
    //    ///
    //    ///             ctx.it("should be usabel inside multiple describes", || Ok(()) as Result<(),()>);
    //    ///             ctx.it("should be able to declare multiple tests", || Ok(()) as Result<(),()>);
    //    ///
    //    ///         });
    //    ///
    //    ///         ctx.it("doesn't care if it's before or after a describe", || Ok(()) as Result<(),()>);
    //    ///     });
    //    /// });
    //    /// ```
    pub fn it<S, F, U>(&mut self, name: S, body: F)
        where S: Into<String>,
              F: 'a + FnMut(&T) -> U,
              U: Into<ExampleResult>
    {
        let info = TestInfo {
            label: TestLabel::It,
            name: name.into(),
            failure: None,
        };
        self.it_internal(info, body)
    }

    /// Alias for [`it`](struct.Context.html#method.it).
    ///
    /// See [`it`](struct.Context.html#method.it) for more info.
    pub fn example<S, F, U>(&mut self, name: S, body: F)
        where S: Into<String>,
              F: 'a + FnMut(&T) -> U,
              U: Into<ExampleResult>
    {
        let info = TestInfo {
            label: TestLabel::Example,
            name: name.into(),
            failure: None,
        };
        self.it_internal(info, body)
    }

    /// Alias for [`it`](struct.Context.html#method.it).
    ///
    /// See [`it`](struct.Context.html#method.it) for more info.
    pub fn then<S, F, U>(&mut self, name: S, body: F)
        where S: Into<String>,
              F: 'a + FnMut(&T) -> U,
              U: Into<ExampleResult>
    {
        let info = TestInfo {
            label: TestLabel::Then,
            name: name.into(),
            failure: None,
        };
        self.it_internal(info, body)
    }

    pub fn it_internal<F, U>(&mut self, info: TestInfo, mut body: F)
        where F: 'a + FnMut(&T) -> U,
              U: Into<ExampleResult>
    {
        let test = Test::new(info, self.environment.clone(), move |environment| {
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
                    ExampleResult::Failure(Failure::new(message))
                },
            }
        });
        self.testables.push(Testable::Test(test))
    }
}

// /// This is the root describe. It will instanciate a root `Context` that you can use to declare
// /// examples, and will returns a Runner ready to run the tests.
// ///
// /// See [`rdescribe`](fn.rdescribe.html) if you want an helper which will setup and run the tests
// /// for you.
// ///
// /// # Examples
// ///
// /// ```no_run
// /// use rspec::context::describe;
// ///
// /// let mut runner = describe("inside this describe, we use the context", (), |ctx| {
// ///
// ///     ctx.it("test at the root", || Ok(()) as Result<(),()>);
// ///
// ///     ctx.describe("a group of examples", (), |ctx| {
// ///
// ///         ctx.it("should be usable inside a describe", || Ok(()) as Result<(),()>);
// ///
// ///         ctx.describe("a nested describe", (), |ctx| {
// ///
// ///             ctx.it("should be usabel inside multiple describes", || Ok(()) as Result<(),()>);
// ///             ctx.it("should be able to declare multiple tests", || Ok(()) as Result<(),()>);
// ///
// ///         });
// ///
// ///         ctx.it("doesn't care if it's before or after a describe", || Ok(()) as Result<(),()>);
// ///     });
// /// });
// /// let result = runner.run();
// /// ```
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
    let ctx_info = ContextInfo {
        label: ContextLabel::Context,
        name: "HANDS OFF!".into(),
    };
    let mut ctx = Context::new(ctx_info, environment);
    body(&mut ctx);
    let suite = Suite::new(info, ctx);
    Runner::new(suite)
}

#[cfg(test)]
mod tests {
    pub use super::*;
    pub use example_result::ExampleResult;

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

    // mod it {
    //     pub use super::*;
    //
    //     macro_rules! test_it_alias {
    //         ($alias: ident) => {
    //             describe("A Root", (), |ctx| ctx.$alias("nested it", || {}));
    //         }
    //     }
    //
    //     #[test]
    //     fn it_has_alias_example() {
    //         test_it_alias!(example);
    //     }
    //
    //     #[test]
    //     fn it_has_alias_then() {
    //         test_it_alias!(then);
    //     }
    //
    //     #[test]
    //     fn it_can_return_a_unit() {
    //         rdescribe("A root", (), |ctx| {
    //             ctx.it("a unit is ok", || {} )
    //         })
    //     }
    //
    //     #[test]
    //     fn is_can_return_a_bool_true() {
    //         rdescribe("a root", (), |ctx| {
    //             ctx.it("a bool true is err", || { true });
    //         });
    //     }
    //
    //     #[test]
    //     fn is_can_return_a_bool_false() {
    //         let runner = describe("a root", (), |ctx| {
    //             ctx.it("a bool true is err", || { false });
    //         });
    //         assert!(runner.run().is_err())
    //     }
    //
    //     #[test]
    //     fn it_can_return_a_result_ok() {
    //         rdescribe("a root", (), |ctx| {
    //             ctx.it("is ok", || Ok(()) as Result<(), ()>);
    //         });
    //     }
    //
    //     #[test]
    //     fn it_can_return_a_result_err() {
    //         let runner = describe("a root", (), |ctx| {
    //             ctx.it("is err", || Err(()) as Result<(), ()>);
    //         });
    //         assert!(runner.run().is_err())
    //     }
    //
    //     #[test]
    //     fn it_can_return_any_result() {
    //         rdescribe("a root", (), |ctx| {
    //             ctx.it("is ok", || Ok(3) as Result<i32, ()>);
    //         });
    //     }
    //
    //     // XXX this MUST NOT compiles
    //     //#[test]
    //     //fn it_cant_returns_something_that_dont_implements_to_res() {
    //     //    let mut runner = describe("a root", (), |ctx| {
    //     //        ctx.it("a bool true is err", || { 3 });
    //     //    });
    //     //    assert!(runner.run().is_err())
    //     //}
    // }
    //
    // mod rdescribe {
    //     pub use super::*;
    //
    //     #[test]
    //     fn it_implicitely_allocate_and_run_a_runner() {
    //         use std::sync::atomic::{AtomicUsize, Ordering};
    //         let counter = &mut AtomicUsize::new(0);
    //
    //         rdescribe("allocates a runner", (), |ctx| {
    //             ctx.before_each(|| {
    //                 counter.fetch_add(1, Ordering::SeqCst);
    //             });
    //             ctx.it("should be runned (1)",
    //                    || 1 == counter.load(Ordering::SeqCst));
    //             ctx.it("should be runned (2)",
    //                    || 2 == counter.load(Ordering::SeqCst));
    //             ctx.it("should be runned (3)",
    //                    || 3 == counter.load(Ordering::SeqCst));
    //         })
    //     }
    //
    //     #[test]
    //     #[should_panic]
    //     fn it_fails_when_run_fails() {
    //         rdescribe("a failed root", (), |ctx| {
    //             ctx.it("a ok test", || Ok(()) as Result<(),()>);
    //             ctx.it("a failed test", || Err(()) as Result<(),()>);
    //             ctx.it("a ok test", || Ok(()) as Result<(),()>);
    //         })
    //     }
    // }
}
