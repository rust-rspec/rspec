
use context::*;
use report::example::{ExampleReport, Failure};

/// Contexts are a powerful method to make your tests clear and well organized.
/// In the long term this practice will keep tests easy to read.
pub struct Context<T> {
    pub(crate) info: Option<ContextInfo>,
    pub(crate) members: Vec<ContextMember<T>>,
    pub(crate) before_all: Vec<Box<Fn(&mut T)>>,
    pub(crate) before_each: Vec<Box<Fn(&mut T)>>,
    pub(crate) after_all: Vec<Box<Fn(&mut T)>>,
    pub(crate) after_each: Vec<Box<Fn(&mut T)>>,
}

impl<T> Context<T> {
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

unsafe impl<T> Send for Context<T>
where
    T: Send,
{
}
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
    /// # use rspec::prelude::*;
    /// #
    /// # pub fn main() {
    /// #     let simple = rspec::formatter::Simple::new(io::stdout());
    /// #     let formatter = Arc::new(Mutex::new(simple));
    /// #     let configuration = Configuration::default().parallel(false);
    /// #     let runner = Runner::new(configuration, vec![formatter]);
    /// #
    /// runner.run_or_exit(rspec::suite("a test suite", (), |ctx| {
    ///     ctx.context("opens a context labeled 'context'", |ctx| {
    ///         // …
    ///     });
    /// }));
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
    pub fn context<'a, F>(&mut self, name: Option<&'static str>, body: F)
    where
        F: FnOnce(&mut Context<T>) -> (),
        T: ::std::fmt::Debug,
    {
        let info = name.map(|name| {
            ContextInfo {
                label: ContextLabel::Context,
                name: name,
            }
        });
        self.context_internal(info, body)
    }

    /// Alias for [`context`](struct.Context.html#method.context), see for more info.
    ///
    /// Available further aliases:
    ///
    /// - [`when`](struct.Context.html#method.when).
    pub fn specify<'a, F>(&mut self, name: Option<&'static str>, body: F)
    where
        F: FnOnce(&mut Context<T>) -> (),
        T: ::std::fmt::Debug,
    {
        let info = name.map(|name| {
            ContextInfo {
                label: ContextLabel::Specify,
                name: name,
            }
        });
        self.context_internal(info, body)
    }

    /// Alias for [`context`](struct.Context.html#method.context), see for more info.
    ///
    /// Available further aliases:
    ///
    /// - [`specify`](struct.Context.html#method.specify).
    pub fn when<'b, F>(&mut self, name: Option<&'static str>, body: F)
    where
        F: FnOnce(&mut Context<T>) -> (),
        T: ::std::fmt::Debug,
    {
        let info = name.map(|name| {
            ContextInfo {
                label: ContextLabel::When,
                name: name,
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
    /// # use rspec::prelude::*;
    /// #
    /// # pub fn main() {
    /// #     let simple = rspec::formatter::Simple::new(io::stdout());
    /// #     let formatter = Arc::new(Mutex::new(simple));
    /// #     let configuration = Configuration::default().parallel(false);
    /// #     let runner = Runner::new(configuration, vec![formatter]);
    /// #
    /// runner.run_or_exit(rspec::suite("a suite",(), |ctx| {
    ///     ctx.context("a context", |ctx| {
    ///         ctx.scope(|ctx| {
    ///             ctx.example("an example", |env| {
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
    /// running tests
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

    fn context_internal<F>(&mut self, info: Option<ContextInfo>, body: F)
    where
        F: FnOnce(&mut Context<T>) -> (),
        T: ::std::fmt::Debug,
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
    /// # use rspec::prelude::*;
    /// #
    /// # pub fn main() {
    /// #     let simple = rspec::formatter::Simple::new(io::stdout());
    /// #     let formatter = Arc::new(Mutex::new(simple));
    /// #     let configuration = Configuration::default().parallel(false);
    /// #     let runner = Runner::new(configuration, vec![formatter]);
    /// #
    /// runner.run_or_exit(rspec::suite("a test suite", (), |ctx| {
    ///     ctx.example("an example", |env| {
    ///         // …
    ///     });
    /// }));
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
    pub fn example<F, U>(&mut self, name: &'static str, body: F)
    where
        F: 'static + Fn(&T) -> U,
        U: Into<ExampleReport>,
    {
        let info = ExampleInfo {
            label: ExampleLabel::Example,
            name: name,
            failure: None,
        };
        self.example_internal(info, body)
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
        let info = ExampleInfo {
            label: ExampleLabel::It,
            name: name,
            failure: None,
        };
        self.example_internal(info, body)
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
        let info = ExampleInfo {
            label: ExampleLabel::Then,
            name: name,
            failure: None,
        };
        self.example_internal(info, body)
    }

    fn example_internal<F, U>(&mut self, info: ExampleInfo, body: F)
    where
        F: 'static + Fn(&T) -> U,
        U: Into<ExampleReport>,
    {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let test = Example::new(info, move |environment| {
            let result = catch_unwind(AssertUnwindSafe(|| body(&environment).into()));
            match result {
                Ok(result) => result,
                Err(error) => {
                    use std::borrow::Cow;
                    let error_as_str = error.downcast_ref::<&str>().map(|s| Cow::from(*s));
                    let error_as_string =
                        error.downcast_ref::<String>().map(|s| Cow::from(s.clone()));
                    let message = error_as_str.or(error_as_string).map(|cow| {
                        let message = cow.to_string();
                        format!("thread panicked at '{:?}'.", message)
                    });
                    ExampleReport::Failure(Failure::new(message))
                }
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
    /// # use rspec::prelude::*;
    /// #
    /// # pub fn main() {
    /// #     let simple = rspec::formatter::Simple::new(io::stdout());
    /// #     let formatter = Arc::new(Mutex::new(simple));
    /// #     let configuration = Configuration::default().parallel(false);
    /// #     let runner = Runner::new(configuration, vec![formatter]);
    /// #
    /// runner.run_or_exit(rspec::suite("a test suite", (), |ctx| {
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
    /// }));
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
    /// # use rspec::prelude::*;
    /// #
    /// # pub fn main() {
    /// #     let simple = rspec::formatter::Simple::new(io::stdout());
    /// #     let formatter = Arc::new(Mutex::new(simple));
    /// #     let configuration = Configuration::default().parallel(false);
    /// #     let runner = Runner::new(configuration, vec![formatter]);
    /// #
    /// runner.run_or_exit(rspec::suite("a test suite", (), |ctx| {
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
    /// }));
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
    where
        F: 'static + Fn(&mut T),
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
    /// # use rspec::prelude::*;
    /// #
    /// # pub fn main() {
    /// #     let simple = rspec::formatter::Simple::new(io::stdout());
    /// #     let formatter = Arc::new(Mutex::new(simple));
    /// #     let configuration = Configuration::default().parallel(false);
    /// #     let runner = Runner::new(configuration, vec![formatter]);
    /// #
    /// runner.run_or_exit(rspec::suite("a test suite", (), |ctx| {
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
    /// }));
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
    /// # use rspec::prelude::*;
    /// #
    /// # pub fn main() {
    /// #     let simple = rspec::formatter::Simple::new(io::stdout());
    /// #     let formatter = Arc::new(Mutex::new(simple));
    /// #     let configuration = Configuration::default().parallel(false);
    /// #     let runner = Runner::new(configuration, vec![formatter]);
    /// #
    /// runner.run_or_exit(rspec::suite("a test suite", (), |ctx| {
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
    /// }));
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
    where
        F: 'static + Fn(&mut T),
    {
        self.after_each.push(Box::new(body))
    }
}
