#[macro_use(expect)]
extern crate expectest;
pub use expectest::prelude::*;

pub type TestResult = Result<(), ()>;

pub struct Context<'a> {
    pub tests: Vec<Box<FnMut() -> TestResult + 'a>>
}

impl<'a> Context<'a> {
    pub fn describe<F>(&mut self, _name: &'a str, mut body: F)
        where F : 'a + FnMut(&mut Context<'a>) -> () {
        body(self)
    }

    pub fn it<F>(&mut self, _name: &'a str, body: F)
        where F : 'a + FnMut() -> TestResult {

        self.tests.push(Box::new(body))
    }
}


pub fn describe<'a, 'b, F>(_block_name: &'b str, body: F) -> Runner<'a>
    where F : 'a + FnOnce(&mut Context<'a>) -> () {

    let mut c = Context { tests: vec!() };
    body(&mut c);
    Runner { describe: c, result: None }
}

pub struct Runner<'a> {
    describe: Context<'a>,
    result: Option<TestResult>
}

impl<'a> Runner<'a> {

    pub fn run(&mut self) -> Result<(), ()> {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        for test_function in self.describe.tests.iter_mut() {
            let res = match catch_unwind(AssertUnwindSafe(|| test_function())) {
                Ok(res) => res,
                _ => Err(())
            };

            self.result = match self.result {
                None => Some(res),
                Some(Ok(())) => Some(res),
                old @ _ => old
            }
        }
        Ok(())
    }

    pub fn result(&self) -> Result<(), ()> {
        self.result.unwrap_or(Ok(()))
    }
}



#[cfg(test)]
mod tests {
    pub use super::*;

    mod describe {
        pub use super::*;

        #[test]
        fn it_has_a_root_describe_function() {
            describe("A Test", |_ctx|{});
        }

        #[test]
        fn it_can_call_describe_inside_describe_body() {
            describe("A Root", |ctx| {
                ctx.describe("nested describe", |_ctx| {})
            });
        }

        #[test]
        fn it_can_call_it_inside_describe_body() {
            describe("A root", |ctx| {
                ctx.it("is a test", || { Ok(()) })
            });
        }
    }

    mod runner {
        pub use super::*;

        #[test]
        fn it_create_a_runner_that_can_be_runned() {
            let mut runner = describe("A root", |ctx| {
                ctx.it("is expected to run", || {
                    assert_eq!(true, true);
                    Ok(())
                })
            });
            assert_eq!(Ok(()), runner.run())
        }

        #[test]
        fn runner_effectively_run_tests() {
            let ran = &mut false;

            {
                let mut runner = describe("A root", |ctx| {
                    ctx.it("is expected to run", || {
                        *ran = true;
                        Ok(())
                    })
                });
                assert_eq!(Ok(()), runner.run());
            }

            assert_eq!(true, *ran)
        }

        #[test]
        fn runner_effectively_run_two_tests() {
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
        fn runner_effectively_run_two_tests_in_nested_describe() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let ran_counter = &mut AtomicUsize::new(0);

            {
                let mut runner = describe("A root", |ctx| {
                    ctx.describe("first describe", |ctx| {
                        ctx.it("first run",  || { ran_counter.fetch_add(1, Ordering::Relaxed); Ok(() )});
                    });
                    ctx.describe("second describe", |ctx| {
                        ctx.it("second run",  || { ran_counter.fetch_add(1, Ordering::Relaxed); Ok(()) });
                    });
                    ctx.describe("third describe", |ctx| {
                        ctx.describe("fourth describe", |ctx| {
                            ctx.it("third run",  || { ran_counter.fetch_add(1, Ordering::Relaxed);  Ok(()) });
                        })
                    })
                });
                let _ = runner.run().unwrap();
            }

            assert_eq!(3, ran_counter.load(Ordering::Relaxed))
        }

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
        fn result_should_be_ok_if_tests_are_ok() {
            let mut runner = describe("A root", |ctx| {
                ctx.it("should be ok", || { Ok(()) })
            });
            runner.run().unwrap();

            expect!(runner.result()).to(be_ok());
        }

        #[test]
        fn results_is_ok_if_no_tests_have_been_runned() {
            let mut runner = describe("A root", |_ctx| {});
            runner.run().unwrap();

            expect!(runner.result()).to(be_ok());
        }

        #[test]
        fn results_is_err_if_one_test_is_err() {
            let mut runner = describe("A root", |ctx| {
                ctx.it("an err", || { Err(()) });
                ctx.it("an ok", || { Ok(()) });
            });
            runner.run().unwrap();

            expect!(runner.result()).to(be_err());
        }

        #[test]
        fn results_is_ok_if_all_tests_are_ok() {
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
    }


    /*
     * Test list:
     * x check that tests can call `assert_eq!`
     * x check that tests can return Err or Ok
     * - runner can count the tests
     * - runner can count the success and failures
     * - check that runner displays the tests names and their results
     * - check that we can use before in a describe
     * - check that we can use after in a describe
     * - check that after/before are run in all child contextes
     */

}
