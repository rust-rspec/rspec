//! The Context module holds all the functionalities for the test declaration, that is:
//! `before`, `after`, `suite`, `context`, `it` and their variants.
//!
//! A Context can also holds reference to children Contextes, for which the `before`
//! [closures](https://rustbyexample.com/fn/closures.html) will be executed after the `before`
//! [closures](https://rustbyexample.com/fn/closures.html) of the current context.
//!
//! Running these tests and doing asserts is not the job of the Context, but the Runner.
//!

use block::{Block, Example};
use header::{ContextLabel, ContextHeader, ExampleLabel, ExampleHeader};
use report::ExampleReport;
use std::default::Default;

/// Test contexts are a convenient tool for adding structure and code sharing to a test suite.
pub struct Context<T> {
    pub(crate) header: Option<ContextHeader>,
    pub(crate) blocks: Vec<Block<T>>,
    pub(crate) before_all: Vec<Box<Fn(&mut T)>>,
    pub(crate) before_each: Vec<Box<Fn(&mut T)>>,
    pub(crate) after_all: Vec<Box<Fn(&mut T)>>,
    pub(crate) after_each: Vec<Box<Fn(&mut T)>>,
}

impl<T> Context<T> {
    pub(crate) fn new(header: Option<ContextHeader>) -> Self {
        Context {
            header: header,
            blocks: vec![],
            before_all: vec![],
            before_each: vec![],
            after_all: vec![],
            after_each: vec![],
        }
    }

    pub fn num_blocks(&self) -> usize {
        self.blocks.len()
    }

    pub fn num_examples(&self) -> usize {
        self.blocks.iter().map(|b| b.num_examples()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

// Both `Send` and `Sync` are necessary for parallel threaded execution.
unsafe impl<T> Send for Context<T>
where
    T: Send,
{
}

// Both `Send` and `Sync` are necessary for parallel threaded execution.
unsafe impl<T> Sync for Context<T>
where
    T: Sync,
{
}

impl<T> Context<T>
where
    T: Clone,
{
    /// Open and name a new context within the current context.
    ///
    /// Note that the order of execution **IS NOT** guaranteed to match the declaration order.
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
    /// runner.run(&rspec::suite("a test suite", (), |ctx| {
    ///     ctx.context("opens a context labeled 'context'", |_ctx| {
    ///         // …
    ///     });
    /// }));
    /// # }
    /// ```
    ///
    /// Corresponding console output:
    ///
    /// ```no-run
    /// tests:
    /// Suite "a test suite":
    ///     Context "opens a sub-context":
    ///         …
    /// ```
    ///
    /// Available aliases:
    ///
    /// - [`specify`](struct.Context.html#method.specify).
    /// - [`when`](struct.Context.html#method.when).
    pub fn context<'a, F>(&mut self, name: &'static str, body: F)
    where
        F: FnOnce(&mut Context<T>) -> (),
        T: ::std::fmt::Debug,
    {
        let header = ContextHeader {
            label: ContextLabel::Context,
            name: name,
        };
        self.context_internal(Some(header), body)
    }

    /// Alias for [`context`](struct.Context.html#method.context), see for more info.
    ///
    /// Available further aliases:
    ///
    /// - [`when`](struct.Context.html#method.when).
    pub fn specify<'a, F>(&mut self, name: &'static str, body: F)
    where
        F: FnOnce(&mut Context<T>) -> (),
        T: ::std::fmt::Debug,
    {
        let header = ContextHeader {
            label: ContextLabel::Specify,
            name: name,
        };
        self.context_internal(Some(header), body)
    }

    /// Alias for [`context`](struct.Context.html#method.context), see for more info.
    ///
    /// Available further aliases:
    ///
    /// - [`specify`](struct.Context.html#method.specify).
    pub fn when<'b, F>(&mut self, name: &'static str, body: F)
    where
        F: FnOnce(&mut Context<T>) -> (),
        T: ::std::fmt::Debug,
    {
        let header = ContextHeader {
            label: ContextLabel::When,
            name: name,
        };
        self.context_internal(Some(header), body)
    }

    /// Open a new name-less context within the current context which won't show up in the logs.
    ///
    /// This can be useful for adding additional structure (and `before`/`after` blocks) to your
    /// tests without their labels showing up as noise in the console output.
    /// As such one might want to selectively assign two contexts/examples an additional `before`
    /// block without them getting visually separated from their neighboring contexts/examples.
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
    /// runner.run(&rspec::suite("a suite",(), |ctx| {
    ///     ctx.context("a context", |ctx| {
    ///         ctx.scope(|ctx| {
    ///             ctx.example("an example", |_env| {
    ///                 // …
    ///             });
    ///         });
    ///     });
    /// }));
    /// # }
    /// ```
    ///
    /// Corresponding console output:
    ///
    /// ```no-run
    /// tests:
    /// Suite "a suite":
    ///     Context "a context":
    ///         Example "an example"
    /// ```
    ///
    /// The `before_each(…)` block gets executed before `'It "tests a"'` and `'It "tests a"'`,
    /// but not before `'It "tests c"'`.
    pub fn scope<F>(&mut self, body: F)
    where
        F: FnOnce(&mut Context<T>) -> (),
        T: ::std::fmt::Debug,
    {
        self.context_internal(None, body)
    }

    fn context_internal<F>(&mut self, header: Option<ContextHeader>, body: F)
    where
        F: FnOnce(&mut Context<T>) -> (),
        T: ::std::fmt::Debug,
    {
        let mut child = Context::new(header);
        body(&mut child);
        self.blocks.push(Block::Context(child))
    }

    /// Open and name a new example within the current context.
    ///
    /// Note that the order of execution **IS NOT** guaranteed to match the declaration order.
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
    /// runner.run(&rspec::suite("a test suite", (), |ctx| {
    ///     ctx.example("an example", |_env| {
    ///         // …
    ///     });
    /// }));
    /// # }
    /// ```
    ///
    /// Corresponding console output:
    ///
    /// ```no-run
    /// tests:
    /// Suite "a test suite":
    ///     Example "an example":
    ///         …
    /// ```
    ///
    /// Available aliases:
    ///
    /// - [`it`](struct.Context.html#method.it).
    /// - [`then`](struct.Context.html#method.then).
    pub fn example<F, U>(&mut self, name: &'static str, body: F)
    where
        F: 'static + Fn(&T) -> U,
        U: Into<ExampleReport>,
    {
        let header = ExampleHeader::new(ExampleLabel::Example, name);
        self.example_internal(header, body)
    }

    /// Alias for [`example`](struct.Context.html#method.example), see for more info.
    ///
    /// Available further aliases:
    ///
    /// - [`it`](struct.Context.html#method.it).
    pub fn it<F, U>(&mut self, name: &'static str, body: F)
    where
        F: 'static + Fn(&T) -> U,
        U: Into<ExampleReport>,
    {
        let header = ExampleHeader::new(ExampleLabel::It, name);
        self.example_internal(header, body)
    }

    /// Alias for [`example`](struct.Context.html#method.example), see for more info.
    ///
    /// Available further aliases:
    ///
    /// - [`it`](struct.Context.html#method.it).
    pub fn then<F, U>(&mut self, name: &'static str, body: F)
    where
        F: 'static + Fn(&T) -> U,
        U: Into<ExampleReport>,
    {
        let header = ExampleHeader::new(ExampleLabel::Then, name);
        self.example_internal(header, body)
    }

    fn example_internal<F, U>(&mut self, header: ExampleHeader, body: F)
    where
        F: 'static + Fn(&T) -> U,
        U: Into<ExampleReport>,
    {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let example = Example::new(header, move |environment| {
            let result = catch_unwind(AssertUnwindSafe(|| body(&environment).into()));
            match result {
                Ok(result) => result,
                Err(error) => {
                    use std::borrow::Cow;
                    let error_as_str = error.downcast_ref::<&str>().map(|s| Cow::from(*s));
                    let error_as_string =
                        error.downcast_ref::<String>().map(|s| Cow::from(s.clone()));
                    let message = error_as_str.or(error_as_string).map(|cow| {
                        format!("thread panicked at '{:?}'.", cow.to_string())
                    });
                    ExampleReport::Failure(message)
                }
            }
        });
        self.blocks.push(Block::Example(example))
    }

    /// Declares a closure that will be executed once before any
    /// of the context's children (context or example blocks) are being executed.
    ///
    /// Note that the order of execution **IS NOT** guaranteed to match the declaration order.
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
    /// runner.run(&rspec::suite("a test suite", (), |ctx| {
    ///     ctx.before_all(|_env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("an example", |_env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("another example", |_env| {
    ///         // …
    ///     });
    /// }));
    /// # }
    /// ```
    ///
    /// Corresponding console output:
    ///
    /// ```no-run
    /// tests:
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
    where
        F: 'static + Fn(&mut T),
    {
        self.before_all.push(Box::new(body))
    }

    /// Alias for [`before_all`](struct.Context.html#method.before_all), see for more info.
    pub fn before<F>(&mut self, body: F)
    where
        F: 'static + Fn(&mut T),
    {
        self.before_all(body)
    }

    /// Declares a closure that will be executed once before each
    /// of the context's children (context or example blocks).
    ///
    /// Note that the order of execution **IS NOT** guaranteed to match the declaration order.
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
    /// runner.run(&rspec::suite("a test suite", (), |ctx| {
    ///     ctx.before_each(|_env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("an example", |_env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("another example", |_env| {
    ///         // …
    ///     });
    /// }));
    /// # }
    /// ```
    ///
    /// Corresponding console output:
    ///
    /// ```no-run
    /// tests:
    /// Suite "a test suite":
    ///     Example "an example":
    ///         …
    ///     Example "another example":
    ///         …
    /// ```
    pub fn before_each<F>(&mut self, body: F)
    where
        F: 'static + Fn(&mut T),
    {
        self.before_each.push(Box::new(body))
    }

    /// Declares a closure that will be executed once after any
    /// of the context's children (context or example blocks) have been executed.
    ///
    /// Note that the order of execution **IS NOT** guaranteed to match the declaration order.
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
    /// runner.run(&rspec::suite("a test suite", (), |ctx| {
    ///     ctx.after_all(|_env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("an example", |_env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("another example", |_env| {
    ///         // …
    ///     });
    /// }));
    /// # }
    /// ```
    ///
    /// Corresponding console output:
    ///
    /// ```no-run
    /// tests:
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
    where
        F: 'static + Fn(&mut T),
    {
        self.after_all.push(Box::new(body))
    }

    /// Alias for [`after_all`](struct.Context.html#method.after_all), see for more info.
    pub fn after<F>(&mut self, body: F)
    where
        F: 'static + Fn(&mut T),
    {
        self.after_all(body)
    }

    /// Declares a closure that will be executed once after each
    /// of the context's children (context or example blocks).
    ///
    /// Note that the order of execution **IS NOT** guaranteed to match the declaration order.
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
    /// runner.run(&rspec::suite("a test suite", (), |ctx| {
    ///     ctx.after_each(|_env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("an example", |_env| {
    ///         // …
    ///     });
    ///
    ///     ctx.example("another example", |_env| {
    ///         // …
    ///     });
    /// }));
    /// # }
    /// ```
    ///
    /// Corresponding console output:
    ///
    /// ```no-run
    /// tests:
    /// Suite "a test suite":
    ///     Example "an example":
    ///         …
    ///     Example "another example":
    ///         …
    /// ```
    pub fn after_each<F>(&mut self, body: F)
    where
        F: 'static + Fn(&mut T),
    {
        self.after_each.push(Box::new(body))
    }
}

impl<T> Default for Context<T> {
    fn default() -> Self {
        Context::new(None)
    }
}

#[cfg(test)]
mod tests {
    use block::{suite, describe, given};

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

                    });
                });
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
