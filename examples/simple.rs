
extern crate rspec;
use rspec::context::describe;
use std::io;

pub fn main() {
    let stdout = &mut io::stdout();
    let mut formatter = rspec::formatter::simple::Simple::new(stdout);
    let mut runner = describe("rspec is a classic BDD testing", |ctx| {

        ctx.it("can define tests", || Ok(()));

        ctx.describe("rspec use results for tests results", |ctx| {

            ctx.it("passed if the return is_ok()", || Ok(()));

            ctx.it("failed if the return is_err()", || Err(()));

            ctx.it("is executed so you can use dynamic values", || {
                if (42 % 37 + 2) > 3 { Ok(()) } else { Err(()) }
            })
        });

        ctx.describe("rspec also supports asserts", |ctx| {

            ctx.it("is a simple test", || {
                assert_eq!(true, false);
                Ok(()) // don't forget to return a Result
            });

            ctx.it("can also pass", || {
                assert_eq!(true, true);
                Ok(())
            });
        });
    });
    runner.add_event_handler(&mut formatter);
    runner.run().unwrap();
}
