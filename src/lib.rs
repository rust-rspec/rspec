
pub struct Context<'a> {
    pub test: Option<Box<FnMut() -> () + 'a>>
}

impl<'a> Context<'a> {
    pub fn describe<F>(&mut self, _name: &'a str, mut body: F)
        where F : 'a + FnMut(&mut Context) -> () {
        body(self)
    }

    pub fn it<F>(&mut self, _name: &'a str, mut body: F)
        where F : 'a + FnMut() -> () {

        self.test = Some(Box::new(body))
    }
}


pub fn describe<'a, 'b, F>(_block_name: &'b str, mut body: F) -> Runner<'a>
    where F : 'a + FnOnce(&mut Context<'a>) -> () {

    let mut c = Context { test: None };
    body(&mut c);
    Runner { describe: c }
}

pub struct Runner<'a> {
    describe: Context<'a>
}

impl<'a> Runner<'a> {
    pub fn run(&mut self) -> Result<(), ()> {
        if let Some(ref mut test_function) = self.describe.test {
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
    fn runner_efectively_run_tests() {
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
}
