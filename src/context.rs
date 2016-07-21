use runner::*;

pub type BeforeFunction<'a> = FnMut() -> () + 'a + Send + Sync;
pub type AfterFunction<'a>  = BeforeFunction<'a>;
pub type TestFunction<'a>   = FnMut() -> TestResult + 'a + Send + Sync;
pub type TestResult         = Result<(), ()>;

pub enum Testable<'a>  {
    Test(Box<TestFunction<'a>>),
    Describe(Context<'a>)
}

#[derive(Default)]
pub struct Context<'a> {
    pub tests: Vec<Testable<'a>>,
    pub before_each: Vec<Box<BeforeFunction<'a>>>,
    pub after_each: Vec<Box<AfterFunction<'a>>>,
}

impl<'a> Context<'a> {
    pub fn describe<F>(&mut self, _name: &'a str, mut body: F)
        where F : 'a + Send + Sync + FnMut(&mut Context<'a>) -> () {

        let mut child = Context::default();
        body(&mut child);
        self.tests.push(Testable::Describe(child))
    }

    pub fn it<F>(&mut self, _name: &'a str, body: F)
        where F : 'a + Send + Sync + FnMut() -> TestResult {

        self.tests.push(Testable::Test(Box::new(body)))
    }

    pub fn before<F>(&mut self, body: F)
        where F : 'a + Send + Sync + FnMut() -> () {

        self.before_each.push(Box::new(body))
    }

    pub fn after<F>(&mut self, body: F)
        where F : 'a + Send + Sync + FnMut() -> () {

        self.after_each.push(Box::new(body))
    }
}

pub fn describe<'a, 'b, F>(_block_name: &'b str, body: F) -> Runner<'a>
    where F : 'a + FnOnce(&mut Context<'a>) -> () {

    let mut c = Context::default();
    body(&mut c);
    Runner::new(c)
}

// TODO: need refactoring
pub fn rdescribe<'a, 'b, F>(block_name: &'b str, body: F) -> ()
    where F : 'a + FnOnce(&mut Context<'a>) -> () {

    let mut runner = describe(block_name, body);
    runner.run().expect("run should be ok");
    let result = runner.result();
    assert!(result.is_ok(), "Tests ran with one mor more failures: {:?}", result)
}


#[cfg(test)]
mod tests {
    pub use super::*;
    pub use expectest::prelude::*;

    pub trait ToRes { fn to_res(self) -> Result<(), ()>; }
    impl ToRes for bool {
        fn to_res(self) -> Result<(), ()> {
            if self { Ok(()) } else { Err(()) }
        }
    }

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

        /*#[test]
        fn it_can_implicitely_returns_ok() {
            describe("a root", |ctx| {
                ctx.it("is ok", || {})
            })
        }*/
    }

    mod before {
        pub use super::*;

        #[test]
        fn can_be_called_inside_describe() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let ran_counter = &mut AtomicUsize::new(0);

            {
                let mut runner = describe("a root", |ctx| {
                    ctx.before(|| { ran_counter.fetch_add(1, Ordering::Relaxed); });
                    ctx.it("first", || { Ok(()) });
                    ctx.it("second", || { Ok(()) });
                    ctx.it("third", || { Ok(()) });
                });
                runner.run().unwrap();
            }

            expect!(ran_counter.load(Ordering::Relaxed)).to(be_equal_to(3));
        }

        #[test]
        fn it_is_only_applied_to_childs_describe() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let ran_counter = &mut AtomicUsize::new(0);

            rdescribe("root", |ctx| {
                ctx.it("shouldn't see the before hook", || (0 == ran_counter.load(Ordering::SeqCst)).to_res());
                ctx.describe("a sub-root", |ctx| {
                    ctx.before(|| { ran_counter.fetch_add(1, Ordering::SeqCst); });
                    ctx.it("should see the before hook", || (1 == ran_counter.load(Ordering::SeqCst)).to_res());
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
                    ctx.after(|| { ran_counter.fetch_add(1, Ordering::Relaxed); });
                    ctx.it("first", || { Ok(()) });
                    ctx.it("second", || { Ok(()) });
                    ctx.it("third", || { Ok(()) });
                });
                runner.run().unwrap();
            }

            expect!(ran_counter.load(Ordering::Relaxed)).to(be_equal_to(3));
        }

        #[test]
        fn it_is_not_like_before() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let ran_counter = &mut AtomicUsize::new(0);

            let report = {
                let mut runner = describe("a root", |ctx| {
                    ctx.after(|| { ran_counter.fetch_add(1, Ordering::SeqCst); });
                    ctx.it("first", || (0 == ran_counter.load(Ordering::SeqCst)).to_res());
                    ctx.it("second", || (1 == ran_counter.load(Ordering::SeqCst)).to_res());
                    ctx.it("third", || (2 == ran_counter.load(Ordering::SeqCst)).to_res());
                });
                runner.run().unwrap();
                runner.result()
            };

            expect!(report).to(be_ok());
        }
    }

    mod rdescribe {
        pub use super::*;

        #[test]
        fn it_implicitely_allocate_and_run_a_runner() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let ran_counter = &mut AtomicUsize::new(0);

            rdescribe("allocates a runner", |ctx| {
                ctx.before(|| { ran_counter.fetch_add(1, Ordering::SeqCst); });
                ctx.it("should be runned (1)", || (1 == ran_counter.load(Ordering::SeqCst)).to_res());
                ctx.it("should be runned (2)", || (2 == ran_counter.load(Ordering::SeqCst)).to_res());
                ctx.it("should be runned (3)", || (3 == ran_counter.load(Ordering::SeqCst)).to_res());
            })
        }

        #[test]
        #[should_panic]
        fn it_fails_when_run_fails() {
            rdescribe("a failed root", |ctx| {
                ctx.it("a ok test", || Ok(()));
                ctx.it("a failed test", || Err(()));
                ctx.it("a ok test", || Ok(()));
            })
        }
    }
}

