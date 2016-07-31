//! The Runner is where all the examples are actually executed.
//!
//! A Runner is instanciated by using [`context::describe`](../context/fn.describe.html) and
//! [`context::rdescribe`](../context/fn.rdescribe.html). You should not try to instanciate
//! a Runner directly.
//!
//! The main methods are `Runner::run` and `Runner::result`.


use context::*;
use events;
use events::Event;
use example_result;
use example_result::ExampleResult;

pub type RunnerResult = Result<TestReport, TestReport>;

/// Handlers is a separate struct which only holds the registered handlers.
/// This is useful to Runner so that its recursive call doesn't have to keep a refernce to `self`
#[derive(Default)]
struct Handlers<'a> {
    handlers: Vec<&'a mut events::EventHandler>,
}

impl<'a> Handlers<'a> {
    fn broadcast(&mut self, event: &events::Event) {
        for h in &mut self.handlers {
            h.trigger(event)
        }
    }
}

pub struct Runner<'a> {
    describe: Context<'a>,
    report: Option<RunnerResult>,
    handlers: Handlers<'a>,
}

impl<'a> Runner<'a> {
    pub fn new(context: Context<'a>) -> Runner<'a> {
        Runner {
            describe: context,
            handlers: Handlers::default(),
            report: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct TestReport {
    pub total_tests: u32,
    pub success_count: u32,
    pub error_count: u32,
}

impl<'a> Runner<'a> {

    #[cfg_attr(feature="clippy", allow(redundant_closure))]
    fn run_test(mut test_function: Box<TestFunction>)
                -> ExampleResult {

        use std::panic::{catch_unwind, AssertUnwindSafe};
        use example_result;

        let res = catch_unwind(AssertUnwindSafe(|| test_function()));

        res.unwrap_or_else(|_| example_result::FAILED_RES)
    }

    fn run_and_recurse(report: &mut TestReport,
                       child_ctx: &mut Context,
                       handlers: &mut Handlers)
                       -> ExampleResult {

        use std::mem;
        let mut result = example_result::SUCCESS_RES;

        // As Runner::run consomes the runner, we can deconstruct the context tree as we travel
        // among it.
        // This effectively takes ownership of the context node and replace it with an empty one.
        let ctx = mem::replace(child_ctx, Context::default());

        let tests = ctx.tests;
        let mut before_functions = ctx.before_each;
        let mut after_functions = ctx.after_each;


        for mut test_function in tests {
            let test_res = {
                for before_function in before_functions.iter_mut() {
                    before_function()
                }
                let res = match test_function {
                    Testable::Test(name, test_function) => {
                        handlers.broadcast(&Event::StartTest(name));
                        let res = Runner::run_test(test_function);
                        handlers.broadcast(&Event::EndTest(res));
                        res
                    }
                    Testable::Describe(ref name, ref mut desc) => {
                        handlers.broadcast(&Event::StartDescribe(name.clone()));
                        let res = Runner::run_and_recurse(report, desc, handlers);
                        handlers.broadcast(&Event::EndDescribe);
                        res
                    }
                };
                for after_function in after_functions.iter_mut() {
                    after_function()
                }
                res
            };

            report.total_tests += 1;

            if test_res.is_ok() {
                report.success_count += 1;
            } else {
                report.error_count += 1
            }

            result = test_res.or(result);
        }

        result
    }

    pub fn run(mut self) -> RunnerResult {
        self.handlers.broadcast(&Event::StartRunner);

        let mut report = TestReport::default();
        let result = Runner::run_and_recurse(&mut report, &mut self.describe, &mut self.handlers);
        let result = result.res().and(Ok(report)).or_else(|_| Err(report));

        self.handlers.broadcast(&Event::FinishedRunner(result));
        result
    }

    pub fn result(&self) -> RunnerResult {
        self.report.unwrap_or_else(|| Ok(TestReport::default()))
    }

    pub fn add_event_handler<H: events::EventHandler>(&mut self, handler: &'a mut H) {
        self.handlers.handlers.push(handler)
    }
}


#[cfg(test)]
mod tests {
    pub use super::*;
    pub use context::*;
    pub use example_result::*;

    mod run {
        pub use super::*;

        #[test]
        fn it_create_a_runner_that_can_be_runned() {
            let runner = describe("A root", |ctx| {
                ctx.it("is expected to run", || {
                    assert_eq!(true, true);
                    Ok(()) as Result<(),()>
                })
            });
            assert!(runner.run().is_ok());
        }

        #[test]
        fn effectively_run_tests() {
            let ran = &mut false;

            {
                let runner = describe("A root", |ctx| {
                    ctx.it("is expected to run", || {
                        *ran = true;
                        Ok(()) as Result<(),()>
                    })
                });
                runner.run().unwrap();
            }

            assert_eq!(true, *ran)
        }

        #[test]
        fn effectively_run_two_tests() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let ran_counter = &mut AtomicUsize::new(0);

            {
                let runner = describe("A root", |ctx| {
                    ctx.it("first run", || {
                        ran_counter.fetch_add(1, Ordering::Relaxed);
                        Ok(()) as Result<(),()>
                    });
                    ctx.it("second run", || {
                        ran_counter.fetch_add(1, Ordering::Relaxed);
                        Ok(()) as Result<(),()>
                    });
                });
                runner.run().unwrap();
            }

            assert_eq!(2, ran_counter.load(Ordering::Relaxed))
        }

        #[test]
        fn effectively_run_two_tests_in_nested_describe() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let ran_counter = &mut AtomicUsize::new(0);

            {
                let runner = describe("A root", |ctx| {
                    ctx.describe("first describe", |ctx| {
                        ctx.it("first run", || {
                            ran_counter.fetch_add(1, Ordering::Relaxed);
                            Ok(()) as Result<(),()>
                        });
                    });
                    ctx.describe("second describe", |ctx| {
                        ctx.it("second run", || {
                            ran_counter.fetch_add(1, Ordering::Relaxed);
                            Ok(()) as Result<(),()>
                        });
                    });
                    ctx.describe("third describe", |ctx| {
                        ctx.describe("fourth describe", |ctx| {
                            ctx.it("third run", || {
                                ran_counter.fetch_add(1, Ordering::Relaxed);
                                Ok(()) as Result<(),()>
                            });
                        })
                    })
                });
                runner.run().unwrap();
            }

            assert_eq!(3, ran_counter.load(Ordering::Relaxed))
        }

        mod events {
            pub use super::*;
            pub use events::*;
            pub use events::Event::*;

            #[derive(Default)]
            struct StubEventHandler {
                pub events: Vec<Event>,
            }

            impl EventHandler for StubEventHandler {
                fn trigger(&mut self, event: &Event) {
                    self.events.push(event.clone())
                }
            }

            #[test]
            fn start_runner_event_is_sent() {
                let mut handler = StubEventHandler::default();
                {
                    let mut runner = describe("empty but should run anyway", |_| {});
                    runner.add_event_handler(&mut handler);

                    runner.run().unwrap();
                }

                assert_eq!(Some(&StartRunner), handler.events.get(0))
            }

            #[test]
            fn no_event_when_no_run() {
                let mut handler = StubEventHandler::default();

                {
                    let mut runner = describe("empty but should run anyway", |_| {});
                    runner.add_event_handler(&mut handler);
                }

                assert_eq!(None, handler.events.get(0))
            }

            #[test]
            fn finished_runner_event_is_last_event_sent() {
                let mut handler = StubEventHandler::default();
                {
                    let mut runner = describe("empty but should run anyway", |_| {});
                    runner.add_event_handler(&mut handler);

                    runner.run().unwrap();
                }

                if let Some(&FinishedRunner(_)) = handler.events.last() {
                    assert!(true);
                } else {
                    assert!(false, "FinishedRunner event not sent at last")
                }
            }

            #[test]
            fn finished_runner_event_has_correct_test_report() {
                let mut handler = StubEventHandler::default();
                {
                    let mut runner = describe("one good test",
                                              |ctx| ctx.it("is a good test", || Ok(()) as Result<(),()>));
                    runner.add_event_handler(&mut handler);

                    runner.run().unwrap();
                }

                if let Some(&FinishedRunner(Ok(report))) = handler.events.last() {
                    let expected_report = TestReport {
                        total_tests: 1,
                        success_count: 1,
                        error_count: 0,
                    };
                    assert_eq!(expected_report, report)
                } else {
                    assert!(false, "FinishedRunner send bad TestReport")
                }
            }

            #[test]
            fn start_test_is_broadcasted() {
                let mut handler = StubEventHandler::default();

                {
                    let mut runner = describe("root", |ctx| {
                        ctx.it("should run with an event", || Ok(()) as Result<(),()>);
                    });
                    runner.add_event_handler(&mut handler);
                    runner.run().unwrap();
                }

                assert_eq!(Some(&StartTest(String::from("should run with an event"))),
                           handler.events.get(1))
            }

            #[test]
            fn end_test_is_broadcasted() {
                let mut handler = StubEventHandler::default();

                {
                    let mut runner = describe("root", |ctx| {
                        ctx.it("should run with an event", || Ok(()) as Result<(),()>);
                    });
                    runner.add_event_handler(&mut handler);
                    runner.run().unwrap();
                }

                assert_eq!(Some(&EndTest(SUCCESS_RES)), handler.events.get(2));
            }

            #[test]
            fn start_describe_is_broadcasted() {
                let mut handler = StubEventHandler::default();

                {
                    let mut runner = describe("root, no hook", |ctx| {
                        ctx.describe("this has a hook", |_| {});
                    });
                    runner.add_event_handler(&mut handler);
                    runner.run().unwrap();
                }

                assert_eq!(Some(&StartDescribe(String::from("this has a hook"))),
                           handler.events.get(1));
            }

            #[test]
            fn end_describe_is_broadcasted() {
                let mut handler = StubEventHandler::default();

                {
                    let mut runner = describe("root, no hook", |ctx| {
                        ctx.describe("this has a hook", |_| {});
                    });
                    runner.add_event_handler(&mut handler);
                    runner.run().unwrap();
                }

                assert_eq!(Some(&EndDescribe), handler.events.get(2));
            }
        }
    }

    mod results {
        pub use super::*;

        #[test]
        fn tests_can_fail_with_an_error_result() {
            let runner = describe("A root", |ctx| ctx.it("should fail", || Err(()) as Result<(),()>));
            let result = runner.run();

            assert!(result.is_err());
        }

        #[test]
        fn should_be_ok_if_tests_are_ok() {
            let runner = describe("A root", |ctx| ctx.it("should be ok", || Ok(()) as Result<(),()>));
            let result = runner.run();

            assert!(result.is_ok());
        }

        #[test]
        fn is_ok_if_no_tests_have_been_runned() {
            let runner = describe("A root", |_ctx| {});
            let result = runner.run();

            assert!(result.is_ok());
        }

        #[test]
        fn is_err_if_one_test_is_err() {
            let runner = describe("A root", |ctx| {
                ctx.it("an err", || Err(()) as Result<(),()>);
                ctx.it("an ok", || Ok(()) as Result<(),()>);
            });
            let result = runner.run();

            assert!(result.is_err());
        }

        #[test]
        fn is_ok_if_all_tests_are_ok() {
            let runner = describe("A root", |ctx| {
                ctx.it("ok 1", || Ok(()) as Result<(),()>);
                ctx.it("ok 2", || Ok(()) as Result<(),()>);
                ctx.it("ok 3", || Ok(()) as Result<(),()>);
                ctx.it("ok 4", || Ok(()) as Result<(),()>);
            });
            let result = runner.run();

            assert!(result.is_ok());
        }

        #[test]
        fn correctly_count_errors() {
            let runner = describe("a root", |ctx| {
                ctx.it("first is ok", || ());
                ctx.it("second is not", || false);
            });

            if let Err(res) = runner.run() {
                assert_eq!((1, 1),
                           (res.success_count, res.error_count));
            } else {
                assert!(false, "unreachable");
            }

        }

        #[test]
        fn tests_can_contains_asserts_that_panic() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let counter = &mut AtomicUsize::new(0);

            let runner = describe("A root", |ctx| {
                ctx.it("assert_eq fail", || {
                    assert_eq!(true, false);
                    Ok(()) as Result<(),()>
                });
                ctx.it("this should also be runned", || {
                    counter.fetch_add(1, Ordering::Relaxed);
                    Ok(()) as Result<(),()>
                })
            });
            let result = runner.run();

            // TODO refactor this to tuple
            assert!(result.is_err());
            assert_eq!(1, counter.load(Ordering::Relaxed));
        }

        #[test]
        fn can_count_the_tests() {
            let runner = describe("a root", |ctx| {
                ctx.it("first", || Ok(()) as Result<(),()>);
                ctx.it("second", || Ok(()) as Result<(),()>);
                ctx.it("third", || Ok(()) as Result<(),()>);
            });
            let results = runner.run();

            assert!(results.is_ok());
            if let Ok(report) = results {
                assert_eq!(3, report.total_tests);
            }
        }

        #[test]
        fn can_count_succes() {
            let runner = describe("a root", |ctx| {
                ctx.it("first", || Ok(()) as Result<(),()>);
                ctx.it("second", || Ok(()) as Result<(),()>);
                ctx.it("third", || Ok(()) as Result<(),()>);
            });
            let result = runner.run();

            assert!(result.is_ok());
            if let Ok(report) = result {
                assert_eq!(3, report.success_count);
            }
        }

        #[test]
        fn can_count_errors() {
            let runner = describe("a root", |ctx| {
                ctx.it("first", || Err(()) as Result<(),()>);
                ctx.it("second", || Err(()) as Result<(),()>);
                ctx.it("third", || Ok(()) as Result<(),()>);
            });
            let result = runner.run();

            assert!(result.is_err());
            if let Err(report) = result {
                assert_eq!(2, report.error_count);
            }
        }
    }
}
