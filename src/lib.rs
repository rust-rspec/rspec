
#[derive(Debug, PartialEq, Eq)]
pub struct Context;

impl Context {
    pub fn describe<F>(&self, _name: &str, _body: F)
        where F: Fn(&Context) -> () {

    }

    pub fn it<F>(&self, _name: &str, _body: F)
        where F : Fn() -> () {

    }
}


pub fn describe<F>(_block_name: &str, _body: F) -> Runner
    where F : Fn(&Context) -> () {

    Runner {}
}

#[derive(Debug, PartialEq, Eq)]
pub struct Runner;

impl Runner {
    pub fn run(&self) -> Result<(), ()> {
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
        let runner = describe("A root", |ctx| {
            ctx.it("is expected to run", || {
                assert_eq!(true, true)
            })
        });
        assert_eq!(Ok(()), runner.run())
    }
}
