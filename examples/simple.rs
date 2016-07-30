
extern crate rspec;
use rspec::context::describe;
use std::io;

pub fn main() {
    let stdout = &mut io::stdout();
    let mut formatter = rspec::formatter::simple::Simple::new(stdout);
    let mut runner = describe("rspec is a classic BDD testing", |ctx| {

        ctx.it("can define tests", || true);

        ctx.describe("rspec use results for tests results", |ctx| {

            ctx.it("passed if the return is_ok()", || Err(()) as Result<(),()>);

            ctx.it("failed if the return is_err()", || Err(()) as Result<(),()>);
        });

        ctx.describe("rspec can use bools", |ctx| {

            ctx.it("should pass if true", || true);

            ctx.it("should fail if false", || false);

            ctx.it("is convenient for comparisons", || {
                (42 % 37 + 2) > 3
            })
        });

        ctx.describe("rspec can use units", |ctx| {

            ctx.it("should pass if the return is ()", || {});

            ctx.it("is convenient for asserts", || assert_eq!(1, 1));
        });
    });
    runner.add_event_handler(&mut formatter);
    runner.run().unwrap();
}
