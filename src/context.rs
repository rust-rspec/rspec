use runner::*;

pub type TestResult = Result<(), ()>;

pub struct Context<'a> {
    pub tests: Vec<Box<FnMut() -> TestResult + 'a>>,
    pub before_each: Vec<Box<FnMut() -> () + 'a>>,
    pub after_each: Vec<Box<FnMut() -> () + 'a>>,
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

    pub fn before<F>(&mut self, body: F)
        where F : 'a + FnMut() -> () {

        self.before_each.push(Box::new(body))
    }

    pub fn after<F>(&mut self, body: F)
        where F : 'a + FnMut() -> () {

        self.after_each.push(Box::new(body))
    }
}

pub fn describe<'a, 'b, F>(_block_name: &'b str, body: F) -> Runner<'a>
    where F : 'a + FnOnce(&mut Context<'a>) -> () {

    let mut c = Context { tests: vec!(), before_each: vec!(), after_each: vec!() };
    body(&mut c);
    Runner::new(c)
}


#[cfg(test)]
mod tests {
    pub use super::*;
    pub use expectest::prelude::*;

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
                    ctx.it("first", || {
                        if 0 == ran_counter.load(Ordering::SeqCst) { Ok(()) } else { Err(()) }
                    });
                    ctx.it("second", || {
                        if 1 == ran_counter.load(Ordering::SeqCst) { Ok(()) } else { Err(()) }
                    });
                    ctx.it("third", || {
                        if 2 == ran_counter.load(Ordering::SeqCst) { Ok(()) } else { Err(()) }
                    });
                });
                runner.run().unwrap();
                runner.result()
            };

            expect!(report).to(be_ok());
        }
    }
}

