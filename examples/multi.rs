extern crate rspec;

use std::io;
use std::sync::Arc;

// An example of a single runner running multiple semantically equivalent,
// yet syntactically different test suites in succession:

pub fn main() {
    let logger = Arc::new(rspec::Logger::new(io::stdout()));
    let configuration = rspec::ConfigurationBuilder::default().build().unwrap();
    let runner = rspec::Runner::new(configuration, vec![logger]);

    // A test suite using the `suite`, `context`, `example` syntax family:
    runner.run(&rspec::suite("an value of ten", 10, |ctx| {
        ctx.context("adding 5 to it", |ctx| {
            ctx.example("results in fifteen", |num| {
                assert_eq!(*num, 15);
            });
        });
    }));

    // A test suite using the `describe`, `specify`, `it` syntax family:
    runner.run(&rspec::describe("an value of ten", 10, |ctx| {
        ctx.specify("adding 5 to it", |ctx| {
            ctx.it("results in fifteen", |num| {
                assert_eq!(*num, 15);
            });
        });
    }));

    // A test suite using the `given`, `when`, `then` syntax family:
    runner.run(&rspec::given("an value of ten", 10, |ctx| {
        ctx.when("adding 5 to it", |ctx| {
            ctx.then("results in fifteen", |num| {
                assert_eq!(*num, 15);
            });
        });
    }));
}
