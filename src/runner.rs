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

use std::any::Any;

impl<'a> Runner<'a> {

    fn normalize_result(res: Box<Any>) -> Result<(), ()> {

        if let Some(res) = res.downcast_ref::<Result<(), ()>>() {
            return *res
        }

        if let Some(_) = res.downcast_ref::<()>() {
            return Ok(())
        }

        if let Some(res) = res.downcast_ref::<bool>() {
            return if *res { Ok(()) } else { Err(()) }
        }

        // TODO: put a log here
        Err(())
    }

    #[cfg_attr(feature="clippy", allow(redundant_closure))]
    fn run_test(test_name: &str,
                test_function: &mut Box<TestFunction>,
                handlers: &mut Handlers)
                -> TestResult {

        use std::panic::{catch_unwind, AssertUnwindSafe};

        handlers.broadcast(&Event::StartTest(String::from(test_name)));
        let res = catch_unwind(AssertUnwindSafe(|| test_function()));

        let res = match res {
            Ok(res) => Runner::normalize_result(res),
            // if test panicked, it means that it failed
            Err(_) => Err(())
        };
        handlers.broadcast(&Event::EndTest(res));
        res
    }

    fn run_and_recurse(report: &mut TestReport,
                       child_ctx: &mut Context,
                       handlers: &mut Handlers)
                       -> TestResult {
        let mut result = Ok(());
        let before_functions = &mut child_ctx.before_each;
        let after_functions = &mut child_ctx.after_each;

        for test_function in &mut child_ctx.tests {
            let test_res = {
                for before_function in before_functions.iter_mut() {
                    before_function()
                }
                let res = match *test_function {
                    Testable::Test(ref name, ref mut test_function) => {
                        Runner::run_test(name, test_function, handlers)
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

            result = match result {
                Ok(()) => {
                    report.success_count += 1;
                    test_res
                }
                old => {
                    report.error_count += 1;
                    old
                }
            };

            report.total_tests += 1;
        }

        result
    }

    pub fn run(&mut self) -> Result<(), ()> {
        self.handlers.broadcast(&Event::StartRunner);

        let mut report = TestReport::default();
        let result = Runner::run_and_recurse(&mut report, &mut self.describe, &mut self.handlers);
        let result = result.and(Ok(report)).or_else(|_| Err(report));

        self.report = Some(result);
        self.handlers.broadcast(&Event::FinishedRunner(result));
        Ok(())
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

    mod run {
        pub use super::*;

        #[test]
        fn it_create_a_runner_that_can_be_runned() {
            let mut runner = describe("A root", |ctx| {
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
                let mut runner = describe("A root", |ctx| {
                    ctx.it("is expected to run", || {
                        *ran = true;
                        Ok(()) as Result<(),()>
                    })
                });
                runner.run().unwrap()
            }

            assert_eq!(true, *ran)
        }

        #[test]
        fn effectively_run_two_tests() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let ran_counter = &mut AtomicUsize::new(0);

            {
                let mut runner = describe("A root", |ctx| {
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
                let mut runner = describe("A root", |ctx| {
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

                assert_eq!(Some(&EndTest(Ok(()) as Result<(),()>)), handler.events.get(2));
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
            let mut runner = describe("A root", |ctx| ctx.it("should fail", || Err(()) as Result<(),()>));
            runner.run().unwrap();

            assert!(runner.result().is_err());
        }

        #[test]
        fn should_be_ok_if_tests_are_ok() {
            let mut runner = describe("A root", |ctx| ctx.it("should be ok", || Ok(()) as Result<(),()>));
            runner.run().unwrap();

            assert!(runner.result().is_ok());
        }

        #[test]
        fn is_ok_if_no_tests_have_been_runned() {
            let mut runner = describe("A root", |_ctx| {});
            runner.run().unwrap();

            assert!(runner.result().is_ok());
        }

        #[test]
        fn is_err_if_one_test_is_err() {
            let mut runner = describe("A root", |ctx| {
                ctx.it("an err", || Err(()) as Result<(),()>);
                ctx.it("an ok", || Ok(()) as Result<(),()>);
            });
            runner.run().unwrap();

            assert!(runner.result().is_err());
        }

        #[test]
        fn is_ok_if_all_tests_are_ok() {
            let mut runner = describe("A root", |ctx| {
                ctx.it("ok 1", || Ok(()) as Result<(),()>);
                ctx.it("ok 2", || Ok(()) as Result<(),()>);
                ctx.it("ok 3", || Ok(()) as Result<(),()>);
                ctx.it("ok 4", || Ok(()) as Result<(),()>);
            });
            runner.run().unwrap();

            assert!(runner.result().is_ok());
        }

        #[test]
        fn tests_can_contains_asserts_that_panic() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let counter = &mut AtomicUsize::new(0);

            let mut runner = describe("A root", |ctx| {
                ctx.it("assert_eq fail", || {
                    assert_eq!(true, false);
                    Ok(()) as Result<(),()>
                });
                ctx.it("this should also be runned", || {
                    counter.fetch_add(1, Ordering::Relaxed);
                    Ok(()) as Result<(),()>
                })
            });
            runner.run().unwrap();

            assert!(runner.result().is_err());
            assert_eq!(1, counter.load(Ordering::Relaxed));
        }

        #[test]
        fn can_count_the_tests() {
            let mut runner = describe("a root", |ctx| {
                ctx.it("first", || Ok(()) as Result<(),()>);
                ctx.it("second", || Ok(()) as Result<(),()>);
                ctx.it("third", || Ok(()) as Result<(),()>);
            });
            runner.run().unwrap();
            let result = runner.result();

            assert!(result.is_ok());
            if let Ok(report) = result {
                assert_eq!(3, report.total_tests);
            }
        }

        #[test]
        fn can_count_succes() {
            let mut runner = describe("a root", |ctx| {
                ctx.it("first", || Ok(()) as Result<(),()>);
                ctx.it("second", || Ok(()) as Result<(),()>);
                ctx.it("third", || Ok(()) as Result<(),()>);
            });
            runner.run().unwrap();
            let result = runner.result();

            assert!(result.is_ok());
            if let Ok(report) = result {
                assert_eq!(3, report.success_count);
            }
        }

        #[test]
        fn can_count_errors() {
            let mut runner = describe("a root", |ctx| {
                ctx.it("first", || Err(()) as Result<(),()>);
                ctx.it("second", || Err(()) as Result<(),()>);
                ctx.it("third", || Ok(()) as Result<(),()>);
            });
            runner.run().unwrap();
            let result = runner.result();

            assert!(result.is_err());
            if let Err(report) = result {
                assert_eq!(2, report.error_count);
            }
        }
    }
}
