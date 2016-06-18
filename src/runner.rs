use context::*;

pub struct Runner<'a> {
    describe: Context<'a>,
    report: Option<Result<TestReport, TestReport>>
}

impl<'a> Runner<'a> {
    pub fn new<'b>(context: Context<'b>) -> Runner<'b> {
        Runner { describe: context, report: None }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct TestReport {
    total_tests: u32,
    success_count: u32,
    error_count: u32
}

impl<'a> Runner<'a> {

    fn run_and_recurse(report: &mut TestReport, child_ctx: &mut Context) -> Result<(), ()> {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let mut result = Ok(());
        let ref mut before_functions = child_ctx.before_each;
        let ref mut after_functions = child_ctx.after_each;

        for test_function in child_ctx.tests.iter_mut() {
            let test_res = catch_unwind(AssertUnwindSafe(|| {
                for before_function in before_functions.iter_mut() { before_function() }
                let res = match test_function {
                    &mut Testable::Test(ref mut test_function) => test_function(),
                    &mut Testable::Describe(ref mut desc) => Runner::run_and_recurse(report, desc)
                };
                for after_function in after_functions.iter_mut() { after_function() }
                res
            }));
            // if test panicked, it means that it failed
            let test_res = test_res.unwrap_or(Err(()));

            result = match result {
                Ok(()) => { report.success_count += 1; test_res },
                old @ _ => { report.error_count += 1; old }
            };

            report.total_tests += 1;
        }

        result
    }

    pub fn run(&mut self) -> Result<(), ()> {
        let mut report = TestReport::default();
        let result = Runner::run_and_recurse(&mut report, &mut self.describe);

        self.report = Some(result.and(Ok(report)).or(Err(report)));
        Ok(())
    }

    pub fn result(&self) -> Result<TestReport, TestReport> {
        self.report.unwrap_or(Ok(TestReport::default()))
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
