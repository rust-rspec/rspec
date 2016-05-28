
pub struct Context<'a> {
    pub test: Option<Box<Fn() -> () + 'a>>
}

impl<'a> Context<'a> {
    pub fn describe<F>(&mut self, _name: &str, _body: F)
        where F: Fn(&mut Context) -> () {

    }

    pub fn it<F>(&mut self, _name: &str, body: F)
        where F : 'a + Fn() -> () {

        self.test = Some(Box::new(body))
    }
}


pub fn describe<F>(_block_name: &str, _body: F) -> Runner
    where F : Fn(&mut Context) -> () {

    let c = Context { test: None };
    Runner { describe: c }
}

pub struct Runner<'a> {
    describe: Context<'a>
}

impl<'a> Runner<'a> {
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
