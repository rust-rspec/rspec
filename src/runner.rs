use context::*;
use events;
use events::Event;

pub type RunnerResult = Result<TestReport, TestReport>;

pub struct Runner<'a> {
    describe: Context<'a>,
    handlers: Vec<&'a mut events::EventHandler>,
    report: Option<RunnerResult>
}

impl<'a> Runner<'a> {
    pub fn new<'b>(context: Context<'b>) -> Runner<'b> {
        Runner { describe: context, handlers: vec!(), report: None }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct TestReport {
    total_tests: u32,
    success_count: u32,
    error_count: u32
}

impl<'a> Runner<'a> {

    fn run_test<'inner1, 'inner2, 'outer, It1, It2>(test_fun: &mut Box<TestFunction<'outer>>,
                                    befores: &It1,
                                    afters:  &It2,
                                    ) -> Result<(), ()>
        where It1: Iterator<Item = &'inner1 mut Box<BeforeFunction<'outer>>>,
              It2: Iterator<Item = &'inner2 mut Box<AfterFunction<'outer>>>,
              'outer : 'inner1,
              'outer : 'inner2 {

        use std::panic::{catch_unwind, AssertUnwindSafe};

        catch_unwind(AssertUnwindSafe(|| {
            for ref mut before_function in befores.into_iter() { before_function() }
            let res = test_fun();
            for ref mut after_function in afters.into_iter() { after_function() }
            res
        })).unwrap_or(Err(()))
    }

    fn run_and_recurse<'inner1, 'inner2, 'inner3, 'inner4, 'outer, It1, It2>(
                                   report:    &'inner1 mut TestReport,
                                   child_ctx: &'inner2 mut Context<'outer>,
                                   befores:   &'inner3 It1,
                                   afters:    &'inner4 It2
                               )
                              -> Result<(), ()>
                              where It1 : Iterator<Item = &'inner2 mut Box<BeforeFunction<'outer>>>,
                                    It2 : Iterator<Item = &'inner3 mut Box<AfterFunction<'outer>>>,
                                    'outer : 'inner1,
                                    'outer : 'inner2,
                                    'outer : 'inner3,
                                    'outer : 'inner4 {

        let mut result = Ok(());
        let ref mut tests = child_ctx.tests;
        let befores = befores.chain(child_ctx.before_each.iter_mut());
        let afters  = child_ctx.after_each.iter_mut().chain(afters.into_iter());

        for test_function in tests.iter_mut() {

            let res = match test_function {
                &mut Testable::Test(ref mut test_function) => Runner::run_test(test_function, &befores, &afters),
                &mut Testable::Describe(ref mut desc) => Runner::run_and_recurse(
                    report,
                    desc,
                    &befores,
                    &afters,
                )
            };

            result = match result {
                Ok(()) => { report.success_count += 1; res },
                old @ _ => { report.error_count += 1; old }
            };

            report.total_tests += 1;
        }

        result
    }

    pub fn run<'b>(&'b mut self) -> Result<(), ()> where 'a : 'b {
        self.broadcast(Event::StartRunner);

        let mut report = TestReport::default();
        let befores_empty = vec!().iter_mut();
        let afters_empty  = vec!().iter_mut();
        let result = Runner::run_and_recurse(&mut report, &mut self.describe, &befores_empty, &afters_empty);
        let result = result.and(Ok(report)).or(Err(report));

        self.report = Some(result);
        self.broadcast(Event::FinishedRunner(result));
        Ok(())
    }

    pub fn result(&self) -> RunnerResult {
        self.report.unwrap_or(Ok(TestReport::default()))
    }

    pub fn add_event_handler<H: events::EventHandler>(&mut self, handler: &'a mut H) {
        self.handlers.push(handler)
    }

    fn broadcast(&mut self, event: events::Event) {
        for h in self.handlers.iter_mut() {
            h.trigger(event)
        }
    }
}


#[cfg(test)]
mod tests {
    pub use super::*;
    pub use context::*;
    pub use expectest::prelude::*;

    mod run {
        pub use super::*;

        #[test]
        fn it_create_a_runner_that_can_be_runned() {
            let mut runner = describe("A root", |ctx| {
                ctx.it("is expected to run", || {
                    assert_eq!(true, true);
                    Ok(())
                })
            });
            expect!(runner.run()).to(be_ok());
        }

        #[test]
        fn effectively_run_tests() {
            let ran = &mut false;

            {
                let mut runner = describe("A root", |ctx| {
                    ctx.it("is expected to run", || {
                        *ran = true;
                        Ok(())
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
                    ctx.it("first run",  || { ran_counter.fetch_add(1, Ordering::Relaxed); Ok(()) });
                    ctx.it("second run", || { ran_counter.fetch_add(1, Ordering::Relaxed); Ok(()) });
                });
                let _ = runner.run().unwrap();
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
                        ctx.it("first run",  || {
                            ran_counter.fetch_add(1, Ordering::Relaxed); Ok(())
                        });
                    });
                    ctx.describe("second describe", |ctx| {
                        ctx.it("second run",  || {
                            ran_counter.fetch_add(1, Ordering::Relaxed); Ok(())
                        });
                    });
                    ctx.describe("third describe", |ctx| {
                        ctx.describe("fourth describe", |ctx| {
                            ctx.it("third run",  || {
                                ran_counter.fetch_add(1, Ordering::Relaxed); Ok(())
                            });
                        })
                    })
                });
                let _ = runner.run().unwrap();
            }

            assert_eq!(3, ran_counter.load(Ordering::Relaxed))
        }

        #[test]
        fn it_runs_befores_correct_number_of_time() {
            use std::sync::atomic::AtomicUsize;
            use std::sync::atomic::Ordering::SeqCst;
            let ran_counter = &mut AtomicUsize::new(0);

            rdescribe("a lot of before incoming (implicitely)", |ctx| {
                ctx.before(|| { ran_counter.fetch_add(1, SeqCst); });

                ctx.it("== 1", || { assert_eq!(1, ran_counter.load(SeqCst)); Ok(()) });
                ctx.it("== 2", || { assert_eq!(2, ran_counter.load(SeqCst)); Ok(()) });

                ctx.describe("flat with depth 1", |ctx| {
                    ctx.it("== 6", || { assert_eq!(6, ran_counter.load(SeqCst)); Ok(()) });
                    ctx.it("== 7", || { assert_eq!(7, ran_counter.load(SeqCst)); Ok(()) });
                    ctx.it("== 8", || { assert_eq!(8, ran_counter.load(SeqCst)); Ok(()) });
                    ctx.it("== 9", || { assert_eq!(9, ran_counter.load(SeqCst)); Ok(()) });
                });

                ctx.describe("depth 1", |ctx| {
                    ctx.it("== 3", || { assert_eq!(3, ran_counter.load(SeqCst)); Ok(()) });
                    ctx.describe("depth 2", |ctx| {
                        ctx.it("== 4", || { assert_eq!(4, ran_counter.load(SeqCst)); Ok(()) });
                        ctx.describe("depth 3", |ctx| {
                            ctx.it("== 5", || { assert_eq!(4, ran_counter.load(SeqCst)); Ok(()) });
                        });
                    });
                });

            });
        }

        mod events {
            pub use super::*;
            pub use events::*;
            pub use events::Event::*;

            #[derive(Default)]
            struct StubEventHandler  {
                pub events: Vec<Event>,
            }

            impl EventHandler for StubEventHandler {
                fn trigger(&mut self, event: Event) {
                    self.events.push(event)
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
                    let mut runner = describe("one good test", |ctx| {
                        ctx.it("is a good test", || Ok(()))
                    });
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
        }
    }

    mod results {
        pub use super::*;

        #[test]
        fn tests_can_fail_with_an_error_result() {
            let mut runner = describe("A root", |ctx| {
                ctx.it("should fail", || {
                    Err(())
                })
            });
            runner.run().unwrap();

            expect!(runner.result()).to(be_err());
        }

        #[test]
        fn should_be_ok_if_tests_are_ok() {
            let mut runner = describe("A root", |ctx| {
                ctx.it("should be ok", || { Ok(()) })
            });
            runner.run().unwrap();

            expect!(runner.result()).to(be_ok());
        }

        #[test]
        fn is_ok_if_no_tests_have_been_runned() {
            let mut runner = describe("A root", |_ctx| {});
            runner.run().unwrap();

            expect!(runner.result()).to(be_ok());
        }

        #[test]
        fn is_err_if_one_test_is_err() {
            let mut runner = describe("A root", |ctx| {
                ctx.it("an err", || { Err(()) });
                ctx.it("an ok", || { Ok(()) });
            });
            runner.run().unwrap();

            expect!(runner.result()).to(be_err());
        }

        #[test]
        fn is_ok_if_all_tests_are_ok() {
            let mut runner = describe("A root", |ctx| {
                ctx.it("ok 1", || { Ok(()) });
                ctx.it("ok 2", || { Ok(()) });
                ctx.it("ok 3", || { Ok(()) });
                ctx.it("ok 4", || { Ok(()) });
            });
            runner.run().unwrap();

            expect!(runner.result()).to(be_ok());
        }

        #[test]
        fn tests_can_contains_asserts_that_panic() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let counter = &mut AtomicUsize::new(0);

            let mut runner = describe("A root", |ctx| {
                ctx.it("assert_eq fail", || { assert_eq!(true, false); Ok(()) });
                ctx.it("this should also be runned", || { counter.fetch_add(1, Ordering::Relaxed); Ok(()) })
            });
            runner.run().unwrap();

            expect!(runner.result()).to(be_err());
            expect!(counter.load(Ordering::Relaxed)).to(be_equal_to(1));
        }

        #[test]
        fn can_count_the_tests() {
            let mut runner = describe("a root", |ctx| {
                ctx.it("first", || { Ok(()) });
                ctx.it("second", || { Ok(()) });
                ctx.it("third", || { Ok(()) });
            });
            runner.run().unwrap();
            let result = runner.result();

            expect!(result).to(be_ok());
            if let Ok(report) = result {
                expect!(report.total_tests).to(be_equal_to(3));
            }
        }

        #[test]
        fn can_count_succes() {
            let mut runner = describe("a root", |ctx| {
                ctx.it("first", || { Ok(()) });
                ctx.it("second", || { Ok(()) });
                ctx.it("third", || { Ok(()) });
            });
            runner.run().unwrap();
            let result = runner.result();

            expect!(result).to(be_ok());
            if let Ok(report) = result {
                expect!(report.success_count).to(be_equal_to(3));
            }
        }

        #[test]
        fn can_count_errors() {
            let mut runner = describe("a root", |ctx| {
                ctx.it("first", || { Err(()) });
                ctx.it("second", || { Err(()) });
                ctx.it("third", || { Ok(()) });
            });
            runner.run().unwrap();
            let result = runner.result();

            expect!(result).to(be_err());
            if let Err(report) = result {
                expect!(report.error_count).to(be_equal_to(2));
            }
        }
    }
}
