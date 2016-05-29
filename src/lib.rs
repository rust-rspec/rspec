
pub struct Context<'a> {
    pub tests: Vec<Box<FnMut() -> () + 'a>>
}

impl<'a> Context<'a> {
    pub fn describe<F>(&mut self, _name: &'a str, mut body: F)
        where F : 'a + FnMut(&mut Context<'a>) -> () {
        body(self)
    }

    pub fn it<F>(&mut self, _name: &'a str, mut body: F)
        where F : 'a + FnMut() -> () {

        self.tests.push(Box::new(body))
    }
}


pub fn describe<'a, 'b, F>(_block_name: &'b str, mut body: F) -> Runner<'a>
    where F : 'a + FnOnce(&mut Context<'a>) -> () {

    let mut c = Context { tests: vec!() };
    body(&mut c);
    Runner { describe: c }
}

pub struct Runner<'a> {
    describe: Context<'a>
}

impl<'a> Runner<'a> {
    pub fn run(&mut self) -> Result<(), ()> {

        for test_function in self.describe.tests.iter_mut() {
            test_function()
        }
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    pub use super::*;
    #[test]
    fn it_has_a_root_describe_functio() {
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
            ctx.it("is a test", || {})
        });
    }

    #[test]
    fn it_create_a_runner_that_can_be_runned() {
        let mut runner = describe("A root", |ctx| {
            ctx.it("is expected to run", || {
                assert_eq!(true, true)
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
                    *ran = true
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
                ctx.it("first run",  || { ran_counter.fetch_add(1, Ordering::Relaxed); });
                ctx.it("second run", || { ran_counter.fetch_add(1, Ordering::Relaxed); });
            });
            let _ = runner.run();
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
                    ctx.it("first run",  || { ran_counter.fetch_add(1, Ordering::Relaxed); });
                });
                ctx.describe("second describe", |ctx| {
                    ctx.it("second run",  || { ran_counter.fetch_add(1, Ordering::Relaxed); });
                });
                ctx.describe("third describe", |ctx| {
                    ctx.describe("fourth describe", |ctx| {
                        ctx.it("third run",  || { ran_counter.fetch_add(1, Ordering::Relaxed); });
                    })
                })
            });
            let _ = runner.run();
        }

        assert_eq!(3, ran_counter.load(Ordering::Relaxed))
    }

    /*
     * Test list:
     * - check that tests can call `assert_eq!`
     * - check that tests can return Err or Ok
     * - runner can count the tests
     * - runner can count the success and failures
     * - check that runner displays the tests names and their results
     * - check that we can use before in a describe
     * - check that we can use after in a describe
     * - check that after/before are run in all child contextes
     */

}
