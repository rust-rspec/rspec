//! The Runner is where all the examples are actually executed.
//!
//! A Runner is instanciated by using [`context::describe`](../context/fn.describe.html) and
//! [`context::rdescribe`](../context/fn.rdescribe.html). You should not try to instanciate
//! a Runner directly.
//!
//! The main methods are `Runner::run` and `Runner::result`.

use std::mem;
use std::panic;

use context::*;
use events::{self, Event};
use example_result::ExampleResult;

pub trait Visitor<T> {
    fn visit(&mut self, visitable: &T);
    // fn enter(&mut self, visitable: &T);
    // fn exit(&mut self, visitable: &T);
}

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

#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct TestReport {
    pub passed: u32,
    pub failed: u32,
    pub ignored: u32,
    pub measured: u32,
}

impl TestReport {
    pub fn new(passed: u32, failed: u32) -> Self {
        TestReport {
            passed: passed,
            failed: failed,
            ignored: 0,
            measured: 0,
        }
    }

    pub fn add<T>(&mut self, report: T)
        where T: Into<TestReport>
    {
        let report: TestReport = report.into();
        self.passed += report.passed;
        self.failed += report.failed;
        self.ignored += report.ignored;
        self.measured += report.measured;
    }
}

impl From<ExampleResult> for TestReport {
    fn from(result: ExampleResult) -> Self {
        let (passed, failed) = if result.is_ok() { (1, 0) } else { (0, 1) };
        TestReport {
            passed: passed,
            failed: failed,
            ignored: 0,
            measured: 0,
        }
    }
}

pub struct Runner<'a, T>
    where T: 'a
{
    suite: Option<Suite<'a, T>>,
    report: TestReport,
    handlers: Handlers<'a>,
}

impl<'a, T> Runner<'a, T> {
    pub fn new(suite: Suite<'a, T>) -> Runner<'a, T> {
        Runner {
            suite: Some(suite),
            report: TestReport::default(),
            handlers: Handlers::default(),
        }
    }
}

impl<'a, T> Runner<'a, T>
    where T: 'a + Clone + ::std::fmt::Debug
{
    pub fn run(mut self) -> TestReport {
        let mut suite = mem::replace(&mut self.suite, None).expect("Expected context");
        panic::set_hook(Box::new(|_panic_info| {
            // silently swallows panics
        }));
        let result = suite.accept(&mut self);
        let _ = panic::take_hook();
        result
    }

    pub fn run_or_exit(self) {
        if self.run().failed > 0 {
            ::std::process::exit(101);
        }
    }

    pub fn add_event_handler<H: events::EventHandler>(&mut self, handler: &'a mut H) {
        self.handlers.handlers.push(handler)
    }

    pub fn broadcast(&mut self, event: Event) {
        self.handlers.broadcast(&event)
    }
}

// #[cfg(test)]
// mod tests {
//     pub use super::*;
//     pub use context::*;
//     pub use example_result::*;
//
//     mod run {
//         pub use super::*;
//
//         #[test]
//         fn it_create_a_runner_that_can_be_runned() {
//             let runner = describe("A root", (), |ctx, _| {
//                 ctx.it("is expected to run", || {
//                     assert_eq!(true, true);
//                     Ok(()) as Result<(),()>
//                 })
//             });
//             assert!(runner.run().is_ok());
//         }
//
//         #[test]
//         fn effectively_run_tests() {
//             let ran = &mut false;
//
//             {
//                 let runner = describe("A root", (), |ctx, _| {
//                     ctx.it("is expected to run", || {
//                         *ran = true;
//                         Ok(()) as Result<(),()>
//                     })
//                 });
//                 runner.run().unwrap();
//             }
//
//             assert_eq!(true, *ran)
//         }
//
//         #[test]
//         fn effectively_run_two_tests() {
//             use std::sync::atomic::{AtomicUsize, Ordering};
//             let ran_counter = &mut AtomicUsize::new(0);
//
//             {
//                 let runner = describe("A root", (), |ctx, _| {
//                     ctx.it("first run", || {
//                         ran_counter.fetch_add(1, Ordering::Relaxed);
//                         Ok(()) as Result<(),()>
//                     });
//                     ctx.it("second run", || {
//                         ran_counter.fetch_add(1, Ordering::Relaxed);
//                         Ok(()) as Result<(),()>
//                     });
//                 });
//                 runner.run().unwrap();
//             }
//
//             assert_eq!(2, ran_counter.load(Ordering::Relaxed))
//         }
//
//         #[test]
//         fn effectively_run_two_tests_in_nested_describe() {
//             use std::sync::atomic::{AtomicUsize, Ordering};
//             let ran_counter = &mut AtomicUsize::new(0);
//
//             {
//                 let runner = describe("A root", (), |ctx, _| {
//                     ctx.describe("first describe", (), |ctx, _| {
//                         ctx.it("first run", || {
//                             ran_counter.fetch_add(1, Ordering::Relaxed);
//                             Ok(()) as Result<(),()>
//                         });
//                     });
//                     ctx.describe("second describe", (), |ctx, _| {
//                         ctx.it("second run", || {
//                             ran_counter.fetch_add(1, Ordering::Relaxed);
//                             Ok(()) as Result<(),()>
//                         });
//                     });
//                     ctx.describe("third describe", (), |ctx, _| {
//                         ctx.describe("fourth describe", (), |ctx, _| {
//                             ctx.it("third run", || {
//                                 ran_counter.fetch_add(1, Ordering::Relaxed);
//                                 Ok(()) as Result<(),()>
//                             });
//                         })
//                     })
//                 });
//                 runner.run().unwrap();
//             }
//
//             assert_eq!(3, ran_counter.load(Ordering::Relaxed))
//         }
//
//         mod events {
//             pub use super::*;
//             pub use events::*;
//             pub use events::Event::*;
//
//             #[derive(Default)]
//             struct StubEventHandler {
//                 pub events: Vec<Event>,
//             }
//
//             impl EventHandler for StubEventHandler {
//                 fn trigger(&mut self, event: &Event) {
//                     self.events.push(event.clone())
//                 }
//             }
//
//             #[test]
//             fn start_runner_event_is_sent() {
//                 let mut handler = StubEventHandler::default();
//                 {
//                     let mut runner = describe("empty but should run anyway", (), |_, _| {});
//                     runner.add_event_handler(&mut handler);
//
//                     runner.run().unwrap();
//                 }
//
//                 assert_eq!(Some(&EnterSuite), handler.events.get(0))
//             }
//
//             #[test]
//             fn no_event_when_no_run() {
//                 let mut handler = StubEventHandler::default();
//
//                 {
//                     let mut runner = describe("empty but should run anyway", (), |_, _| {});
//                     runner.add_event_handler(&mut handler);
//                 }
//
//                 assert_eq!(None, handler.events.get(0))
//             }
//
//             #[test]
//             fn finished_runner_event_is_last_event_sent() {
//                 let mut handler = StubEventHandler::default();
//                 {
//                     let mut runner = describe("empty but should run anyway", (), |_, _| {});
//                     runner.add_event_handler(&mut handler);
//
//                     runner.run().unwrap();
//                 }
//
//                 if let Some(&ExitSuite(_)) = handler.events.last() {
//                     assert!(true);
//                 } else {
//                     assert!(false, "ExitSuite event not sent at last")
//                 }
//             }
//
//             #[test]
//             fn finished_runner_event_has_correct_test_report() {
//                 let mut handler = StubEventHandler::default();
//                 {
//                     let mut runner = describe("one good test",
//                                               |ctx| ctx.it("is a good test", || Ok(()) as Result<(),()>));
//                     runner.add_event_handler(&mut handler);
//
//                     runner.run().unwrap();
//                 }
//
//                 if let Some(&ExitSuite(Ok(report))) = handler.events.last() {
//                     let expected_report = TestReport {
//                         total_tests: 1,
//                         success_count: 1,
//                         error_count: 0,
//                     };
//                     assert_eq!(expected_report, report)
//                 } else {
//                     assert!(false, "ExitSuite send bad TestReport")
//                 }
//             }
//
//             #[test]
//             fn start_test_is_broadcasted() {
//                 let mut handler = StubEventHandler::default();
//
//                 {
//                     let mut runner = describe("root", (), |ctx, _| {
//                         ctx.it("should run with an event", || Ok(()) as Result<(),()>);
//                     });
//                     runner.add_event_handler(&mut handler);
//                     runner.run().unwrap();
//                 }
//
//                 assert_eq!(Some(&EnterTest(String::from("should run with an event"))),
//                            handler.events.get(1))
//             }
//
//             #[test]
//             fn end_test_is_broadcasted() {
//                 let mut handler = StubEventHandler::default();
//
//                 {
//                     let mut runner = describe("root", (), |ctx, _| {
//                         ctx.it("should run with an event", || Ok(()) as Result<(),()>);
//                     });
//                     runner.add_event_handler(&mut handler);
//                     runner.run().unwrap();
//                 }
//
//                 assert_eq!(Some(&ExitTest(SUCCESS_RES)), handler.events.get(2));
//             }
//
//             #[test]
//             fn start_describe_is_broadcasted() {
//                 let mut handler = StubEventHandler::default();
//
//                 {
//                     let mut runner = describe("root, no hook", (), |ctx, _| {
//                         ctx.describe("this has a hook", (), |_, _| {});
//                     });
//                     runner.add_event_handler(&mut handler);
//                     runner.run().unwrap();
//                 }
//
//                 assert_eq!(Some(&EnterContext(String::from("this has a hook"))),
//                            handler.events.get(1));
//             }
//
//             #[test]
//             fn end_describe_is_broadcasted() {
//                 let mut handler = StubEventHandler::default();
//
//                 {
//                     let mut runner = describe("root, no hook", (), |ctx, _| {
//                         ctx.describe("this has a hook", (), |_, _| {});
//                     });
//                     runner.add_event_handler(&mut handler);
//                     runner.run().unwrap();
//                 }
//
//                 assert_eq!(Some(&ExitContext), handler.events.get(2));
//             }
//         }
//     }
//
//     mod results {
//         pub use super::*;
//
//         #[test]
//         fn tests_can_fail_with_an_error_result() {
//             let runner = describe("A root", (), |ctx, _| ctx.it("should fail", || Err(()) as Result<(),()>));
//             let result = runner.run();
//
//             assert!(result.is_err());
//         }
//
//         #[test]
//         fn should_be_ok_if_tests_are_ok() {
//             let runner = describe("A root", (), |ctx, _| ctx.it("should be ok", || Ok(()) as Result<(),()>));
//             let result = runner.run();
//
//             assert!(result.is_ok());
//         }
//
//         #[test]
//         fn is_ok_if_no_tests_have_been_runned() {
//             let runner = describe("A root", |_ctx| {});
//             let result = runner.run();
//
//             assert!(result.is_ok());
//         }
//
//         #[test]
//         fn is_err_if_one_test_is_err() {
//             let runner = describe("A root", (), |ctx, _| {
//                 ctx.it("an err", || Err(()) as Result<(),()>);
//                 ctx.it("an ok", || Ok(()) as Result<(),()>);
//             });
//             let result = runner.run();
//
//             assert!(result.is_err());
//         }
//
//         #[test]
//         fn is_ok_if_all_are_ok() {
//             let runner = describe("A root", (), |ctx, _| {
//                 ctx.it("ok 1", || Ok(()) as Result<(),()>);
//                 ctx.it("ok 2", || Ok(()) as Result<(),()>);
//                 ctx.it("ok 3", || Ok(()) as Result<(),()>);
//                 ctx.it("ok 4", || Ok(()) as Result<(),()>);
//             });
//             let result = runner.run();
//
//             assert!(result.is_ok());
//         }
//
//         #[test]
//         fn correctly_count_errors() {
//             let runner = describe("a root", (), |ctx, _| {
//                 ctx.it("first is ok", || ());
//                 ctx.it("second is not", || false);
//             });
//
//             if let Err(res) = runner.run() {
//                 assert_eq!((1, 1),
//                            (res.success_count, res.error_count));
//             } else {
//                 assert!(false, "unreachable");
//             }
//
//         }
//
//         #[test]
//         fn tests_can_contains_asserts_that_panic() {
//             use std::sync::atomic::{AtomicUsize, Ordering};
//             let counter = &mut AtomicUsize::new(0);
//
//             let runner = describe("A root", (), |ctx, _| {
//                 ctx.it("assert_eq fail", || {
//                     assert_eq!(true, false);
//                     Ok(()) as Result<(),()>
//                 });
//                 ctx.it("this should also be runned", || {
//                     counter.fetch_add(1, Ordering::Relaxed);
//                     Ok(()) as Result<(),()>
//                 })
//             });
//             let result = runner.run();
//
//             // TODO refactor this to tuple
//             assert!(result.is_err());
//             assert_eq!(1, counter.load(Ordering::Relaxed));
//         }
//
//         #[test]
//         fn can_count_the_tests() {
//             let runner = describe("a root", (), |ctx, _| {
//                 ctx.it("first", || Ok(()) as Result<(),()>);
//                 ctx.it("second", || Ok(()) as Result<(),()>);
//                 ctx.it("third", || Ok(()) as Result<(),()>);
//             });
//             let results = runner.run();
//
//             assert!(results.is_ok());
//             if let Ok(report) = results {
//                 assert_eq!(3, report.total_tests);
//             }
//         }
//
//         #[test]
//         fn can_count_succes() {
//             let runner = describe("a root", (), |ctx, _| {
//                 ctx.it("first", || Ok(()) as Result<(),()>);
//                 ctx.it("second", || Ok(()) as Result<(),()>);
//                 ctx.it("third", || Ok(()) as Result<(),()>);
//             });
//             let result = runner.run();
//
//             assert!(result.is_ok());
//             if let Ok(report) = result {
//                 assert_eq!(3, report.success_count);
//             }
//         }
//
//         #[test]
//         fn can_count_errors() {
//             let runner = describe("a root", (), |ctx, _| {
//                 ctx.it("first", || Err(()) as Result<(),()>);
//                 ctx.it("second", || Err(()) as Result<(),()>);
//                 ctx.it("third", || Ok(()) as Result<(),()>);
//             });
//             let result = runner.run();
//
//             assert!(result.is_err());
//             if let Err(report) = result {
//                 assert_eq!(2, report.error_count);
//             }
//         }
//     }
// }
