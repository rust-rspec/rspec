//! The Context module holds all the functionality for the test declaration, that is: `describe`,
//! `before`, `after`, `it` and their variants.
//!
//! A Context can also holds reference to children Contextes, for whom the before closures will be
//! executed after the before closures of the current context. The order of execution of tests
//! respect the order of declaration of theses tests.
//!
//! Running these tests and doing asserts is not the job of the Context, but the Runner, which is
//! a struct returned by the root context declaration.
//!
//! # Examples
//! ```
//! use rspec::context::*;
//!
//! // `rdescribe` instanciate a runner and run it transparently
//! rdescribe("Context", |ctx| {
//!     describe("Context::describe", |ctx| {
//!         ctx.it("can be nested", || Ok(()) as Result<(),()>);
//!         ctx.it("use a `ctx` object", || Ok(()) as Result<(),()>)
//!     });
//!
//!     describe("Context::it", |ctx| {
//!         ctx.it("uses a Result returns", || Ok(()) as Result<(),()>);
//!         ctx.it("can also use asserts", || {
//!             assert_eq!(42, 12 + 30);
//!             Ok(()) as Result<(),()> // don't forget the result type
//!         })
//!     });
//! });
//!

use runner::*;
use example_result::ExampleResult;

/// This is the type used by the closure given as argument of a `Context::before()` call.
///
/// It is Send and Sync for forward compatibility reasons.
///
/// **Please Note** that `before` is effectively a `before each child of current context` function.
pub type BeforeFunction<'a> = FnMut() -> () + 'a + Send + Sync;
/// This is the type used by the closure given as argument of a `Context::after()` call.
///
/// This is Send and Sync for forward compatibility reasons.
///
/// **Please Note** that `after` is effectively a `after each child of current context` function.
pub type AfterFunction<'a> = BeforeFunction<'a>;
/// This is the type used by the closure given as argument of a `Context::it()` call.
///
/// This is Send and Sync for forward compatibility reasons.
pub type TestFunction<'a> = FnMut() -> ExampleResult + 'a + Send + Sync;
/// The type used for a test result
pub type TestResult = Result<(), ()>;

/// This enum is used to build a tree of named tests and contextes.
pub enum Testable<'a> {
    /// Name and Test body
    Test(String, Box<TestFunction<'a>>),
    /// Name and Describe body
    Describe(String, Context<'a>),
}

/// A Context holds a collection of tests, a collection of closures to call before running any
/// tests, and a collection of closure to call _after_ all the tests..
///
/// This is effectively the struct we fill when calling `ctx.it()`
#[derive(Default)]
pub struct Context<'a> {
    pub tests: Vec<Testable<'a>>,
    pub before_each: Vec<Box<BeforeFunction<'a>>>,
    pub after_each: Vec<Box<AfterFunction<'a>>>,
}

impl<'a> Context<'a> {
    /// Open and name a new example group, which will be keeped as a child context of the current
    /// context.
    ///
    /// Note that the order of declaration is respected for running the tests.
    ///
    /// # Examples
    ///
    /// ```
    /// use rspec::context::rdescribe;
    ///
    /// // `rdescribe` instanciate a runner and run it transparently
    /// rdescribe("inside this describe, we use the context", |ctx| {
    ///
    ///     ctx.it("should run first", || Ok(()) as Result<(),()>);
    ///
    ///     ctx.describe("open describe", |ctx| {
    ///
    ///         ctx.it("should run second", || Ok(()) as Result<(),()>);
    ///
    ///         ctx.describe("in a describe", |ctx| {
    ///
    ///             ctx.describe("in a describe", |_ctx| {});
    ///
    ///             ctx.it("should run third", || Ok(()) as Result<(),()>);
    ///
    ///         });
    ///     });
    ///
    ///     ctx.it("should run last", || Ok(()) as Result<(),()>);
    /// });
    /// ```
    pub fn describe<F>(&mut self, name: &'a str, mut body: F)
        where F: 'a + Send + Sync + FnMut(&mut Context<'a>) -> ()
    {

        let mut child = Context::default();
        body(&mut child);
        self.tests.push(Testable::Describe(String::from(name), child))
    }

    /// Register and name a closure as an example
    ///
    /// # Examples
    ///
    /// ```
    /// use rspec::context::rdescribe;
    ///
    /// // `rdescribe` instanciate a runner and run it transparently
    /// rdescribe("inside this describe, we use the context", |ctx| {
    ///
    ///     ctx.it("test at the root", || Ok(()) as Result<(),()>);
    ///
    ///     ctx.describe("a group of examples", |ctx| {
    ///
    ///         ctx.it("should be usable inside a describe", || Ok(()) as Result<(),()>);
    ///
    ///         ctx.describe("a nested describe", |ctx| {
    ///
    ///             ctx.it("should be usabel inside multiple describes", || Ok(()) as Result<(),()>);
    ///             ctx.it("should be able to declare multiple tests", || Ok(()) as Result<(),()>);
    ///
    ///         });
    ///
    ///         ctx.it("doesn't care if it's before or after a describe", || Ok(()) as Result<(),()>);
    ///     });
    /// });
    /// ```
    pub fn it<F, T>(&mut self, name: &'a str, mut body: F)
        where F: 'a + Send + Sync + FnMut() -> T,
              T: Into<ExampleResult>
    {

        let f = move || { body().into() };
        self.tests.push(Testable::Test(String::from(name), Box::new(f)))
    }

    /// Declares a closure that will be executed before each test of the current Context.
    ///
    /// **PLEASE NOTE**: due to a bug in current versions of rspec, the before closures **WILL BE
    /// CALLED ONLY ONCE** for all the children of the current context.
    ///
    /// # Examples
    ///
    /// ```
    /// use rspec::context::rdescribe;
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    ///
    /// let counter = &mut AtomicUsize::new(0);
    ///
    /// // `rdescribe` instanciate a runner and run it transparently
    /// rdescribe("inside this describe, we use the context", |ctx| {
    ///
    ///     // This will increment the counter at each test
    ///     ctx.before(|| { counter.fetch_add(1, Ordering::SeqCst); });
    ///
    ///     ctx.it("should run after the before", || {
    ///         assert_eq!(1, counter.load(Ordering::SeqCst));
    ///         Ok(()) as Result<(),()>
    ///     });
    ///
    ///     ctx.describe("a group of examples", |ctx| {
    ///
    ///         ctx.it("should see 1 increment", || {
    ///             assert_eq!(2, counter.load(Ordering::SeqCst));
    ///             Ok(()) as Result<(),()>
    ///         });
    ///
    ///         // XXX - note that the before has not been applied another time
    ///         ctx.it("should NOT see another increment", || {
    ///             assert_eq!(2, counter.load(Ordering::SeqCst));
    ///             Ok(()) as Result<(),()>
    ///         });
    ///     });
    ///
    ///     ctx.it("should run after the all the befores AND the previous it", || {
    ///         assert_eq!(3, counter.load(Ordering::SeqCst));
    ///         Ok(()) as Result<(),()>
    ///     });
    /// });
    /// ```
    pub fn before<F>(&mut self, body: F)
        where F: 'a + Send + Sync + FnMut() -> ()
    {

        self.before_each.push(Box::new(body))
    }

    /// Declares a closure that will be executed after each test of the current Context.
    ///
    /// **PLEASE NOTE**: due to a bug in current versions of rspec, the after closures **WILL BE
    /// CALLED ONLY ONCE** for all the children of the current context.
    ///
    /// # Examples
    ///
    /// ```
    /// use rspec::context::rdescribe;
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    ///
    /// let counter = &mut AtomicUsize::new(0);
    ///
    /// // `rdescribe` instanciate a runner and run it transparently
    /// rdescribe("inside this describe, we use the context", |ctx| {
    ///
    ///     // This will increment the counter at each test
    ///     ctx.after(|| { counter.fetch_add(1, Ordering::SeqCst); });
    ///
    ///     ctx.it("should run after the after", || {
    ///         assert_eq!(0, counter.load(Ordering::SeqCst));
    ///         Ok(()) as Result<(),()>
    ///     });
    ///
    ///     ctx.describe("a group of examples", |ctx| {
    ///
    ///         ctx.it("should see 1 increment", || {
    ///             assert_eq!(1, counter.load(Ordering::SeqCst));
    ///             Ok(()) as Result<(),()>
    ///         });
    ///
    ///         // XXX - note that the after has not been applied another time
    ///         ctx.it("should NOT see another increment", || {
    ///             assert_eq!(1, counter.load(Ordering::SeqCst));
    ///             Ok(()) as Result<(),()>
    ///         });
    ///     });
    ///
    ///     ctx.it("should run after the all the afters AND the previous it", || {
    ///         assert_eq!(2, counter.load(Ordering::SeqCst));
    ///         Ok(()) as Result<(),()>
    ///     });
    /// });
    /// ```
    pub fn after<F>(&mut self, body: F)
        where F: 'a + Send + Sync + FnMut() -> ()
    {

        self.after_each.push(Box::new(body))
    }
}

/// This is the root describe. It will instanciate a root `Context` that you can use to declare
/// examples, and will returns a Runner ready to run the tests.
///
/// See [`rdescribe`](fn.rdescribe.html) if you want an helper which will setup and run the tests
/// for you.
///
/// # Examples
///
/// ```
/// use rspec::context::describe;
///
/// let mut runner = describe("inside this describe, we use the context", |ctx| {
///
///     ctx.it("test at the root", || Ok(()) as Result<(),()>);
///
///     ctx.describe("a group of examples", |ctx| {
///
///         ctx.it("should be usable inside a describe", || Ok(()) as Result<(),()>);
///
///         ctx.describe("a nested describe", |ctx| {
///
///             ctx.it("should be usabel inside multiple describes", || Ok(()) as Result<(),()>);
///             ctx.it("should be able to declare multiple tests", || Ok(()) as Result<(),()>);
///
///         });
///
///         ctx.it("doesn't care if it's before or after a describe", || Ok(()) as Result<(),()>);
///     });
/// });
/// runner.run().unwrap();
/// ```
pub fn describe<'a, 'b, F>(_block_name: &'b str, body: F) -> Runner<'a>
    where F: 'a + FnOnce(&mut Context<'a>) -> ()
{

    let mut c = Context::default();
    body(&mut c);
    Runner::new(c)
}

/// This is the root describe with a sugar. It will instanciate a root `Context` that you can use
/// to declare examples, will instanciate a `Runner` for the test and run them.
///
/// See [`describe`](fn.describe.html) if you want to control the runner precisely.
///
/// # Panics
///
/// If the runner failed, which could means that one or more examples failed (likely) or that
/// another kind of error made the run to stop (unlikely).
///
/// # Examples
///
/// ```
/// use rspec::context::rdescribe;
///
/// // `rdescribe` instanciate a runner and run it transparently
/// rdescribe("inside this describe, we use the context", |ctx| {
///
///     ctx.it("test at the root", || Ok(()) as Result<(),()>);
///
///     ctx.describe("a group of examples", |ctx| {
///
///         ctx.it("should be usable inside a describe", || Ok(()) as Result<(),()>);
///
///         ctx.describe("a nested describe", |ctx| {
///
///             ctx.it("should be usabel inside multiple describes", || Ok(()) as Result<(),()>);
///             ctx.it("should be able to declare multiple tests", || Ok(()) as Result<(),()>);
///
///         });
///
///         ctx.it("doesn't care if it's before or after a describe", || Ok(()) as Result<(),()>);
///     });
/// });
/// ```
pub fn rdescribe<'a, 'b, F>(block_name: &'b str, body: F) -> ()
    where F: 'a + FnOnce(&mut Context<'a>) -> ()
{

    let mut runner = describe(block_name, body);
    runner.run().expect("run should be ok");
    let result = runner.result();
    assert!(result.is_ok(),
            "Tests ran with one mor more failures: {:?}",
            result)
}


#[cfg(test)]
mod tests {
    pub use super::*;
    pub use example_result::ExampleResult;

    mod describe {
        pub use super::*;

        #[test]
        fn it_has_a_root_describe_function() {
            describe("A Test", |_ctx| {});
        }

        #[test]
        fn it_can_call_describe_inside_describe_body() {
            describe("A Root", |ctx| ctx.describe("nested describe", |_ctx| {}));
        }

        #[test]
        fn it_can_call_it_inside_describe_body() {
            describe("A root", |ctx| ctx.it("is a test", || Ok(()) as Result<(),()>));
        }
    }

    mod it {
        pub use super::*;

        #[test]
        fn it_can_return_a_unit() {
            rdescribe("A root", |ctx| {
                ctx.it("a unit is ok", || {} )
            })
        }

        #[test]
        fn is_can_return_a_bool_true() {
            rdescribe("a root", |ctx| {
                ctx.it("a bool true is err", || { true });
            });
        }

        #[test]
        fn is_can_return_a_bool_false() {
            let mut runner = describe("a root", |ctx| {
                ctx.it("a bool true is err", || { false });
            });
            runner.run().unwrap();
            assert!(runner.result().is_err())
        }

        #[test]
        fn it_can_return_a_result_ok() {
            rdescribe("a root", |ctx| {
                ctx.it("is ok", || Ok(()) as Result<(), ()>);
            });
        }

        #[test]
        fn it_can_return_a_result_err() {
            let mut runner = describe("a root", |ctx| {
                ctx.it("is err", || Err(()) as Result<(), ()>);
            });
            runner.run().unwrap();
            assert!(runner.result().is_err())
        }

        #[test]
        fn it_can_return_any_result() {
            rdescribe("a root", |ctx| {
                ctx.it("is ok", || Ok(3) as Result<i32, ()>);
            });
        }

        // XXX this MUST NOT compiles
        //#[test]
        //fn it_cant_returns_something_that_dont_implements_to_res() {
        //    let mut runner = describe("a root", |ctx| {
        //        ctx.it("a bool true is err", || { 3 });
        //    });
        //    runner.run().unwrap();
        //    assert!(runner.result().is_err())
        //}
    }

    mod before {
        pub use super::*;

        #[test]
        fn can_be_called_inside_describe() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let ran_counter = &mut AtomicUsize::new(0);

            {
                let mut runner = describe("a root", |ctx| {
                    ctx.before(|| {
                        ran_counter.fetch_add(1, Ordering::Relaxed);
                    });
                    ctx.it("first", || Ok(()) as Result<(),()>);
                    ctx.it("second", || Ok(()) as Result<(),()>);
                    ctx.it("third", || Ok(()) as Result<(),()>);
                });
                runner.run().unwrap();
            }

            assert_eq!(3, ran_counter.load(Ordering::Relaxed));
        }

        #[test]
        fn it_is_only_applied_to_childs_describe() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let ran_counter = &mut AtomicUsize::new(0);

            rdescribe("root", |ctx| {
                ctx.it("shouldn't see the before hook",
                       || (0 == ran_counter.load(Ordering::SeqCst)));
                ctx.describe("a sub-root", |ctx| {
                    ctx.before(|| {
                        ran_counter.fetch_add(1, Ordering::SeqCst);
                    });
                    ctx.it("should see the before hook",
                           || (1 == ran_counter.load(Ordering::SeqCst)));
                })

            })
        }
    }

    mod after {
        pub use super::*;

        #[test]
        fn can_be_called_inside_describe() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let ran_counter = &mut AtomicUsize::new(0);

            {
                let mut runner = describe("a root", |ctx| {
                    ctx.after(|| {
                        ran_counter.fetch_add(1, Ordering::Relaxed);
                    });
                    ctx.it("first", || Ok(()) as Result<(),()>);
                    ctx.it("second", || Ok(()) as Result<(),()>);
                    ctx.it("third", || Ok(()) as Result<(),()>);
                });
                runner.run().unwrap();
            }

            assert_eq!(3, ran_counter.load(Ordering::Relaxed));
        }

        #[test]
        fn it_is_not_like_before() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let ran_counter = &mut AtomicUsize::new(0);

            let report = {
                let mut runner = describe("a root", |ctx| {
                    ctx.after(|| {
                        ran_counter.fetch_add(1, Ordering::SeqCst);
                    });
                    ctx.it("first",
                           || 0 == ran_counter.load(Ordering::SeqCst));
                    ctx.it("second",
                           || 1 == ran_counter.load(Ordering::SeqCst));
                    ctx.it("third",
                           || 2 == ran_counter.load(Ordering::SeqCst));
                });
                runner.run().unwrap();
                runner.result()
            };

            assert!(report.is_ok());
        }
    }

    mod rdescribe {
        pub use super::*;

        #[test]
        fn it_implicitely_allocate_and_run_a_runner() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let ran_counter = &mut AtomicUsize::new(0);

            rdescribe("allocates a runner", |ctx| {
                ctx.before(|| {
                    ran_counter.fetch_add(1, Ordering::SeqCst);
                });
                ctx.it("should be runned (1)",
                       || 1 == ran_counter.load(Ordering::SeqCst));
                ctx.it("should be runned (2)",
                       || 2 == ran_counter.load(Ordering::SeqCst));
                ctx.it("should be runned (3)",
                       || 3 == ran_counter.load(Ordering::SeqCst));
            })
        }

        #[test]
        #[should_panic]
        fn it_fails_when_run_fails() {
            rdescribe("a failed root", |ctx| {
                ctx.it("a ok test", || Ok(()) as Result<(),()>);
                ctx.it("a failed test", || Err(()) as Result<(),()>);
                ctx.it("a ok test", || Ok(()) as Result<(),()>);
            })
        }
    }
}
