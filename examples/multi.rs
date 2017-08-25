extern crate rspec;

use std::io;
use std::sync::Arc;
use std::collections::BTreeSet;

// An example of a single runner running multiple test suites in succession.

pub fn main() {
    let formatter = Arc::new(rspec::Formatter::new(io::stdout()));
    let configuration = rspec::ConfigurationBuilder::default().build().unwrap();
    let runner = rspec::Runner::new(configuration, vec![formatter]);

    #[derive(Clone, Debug)]
    struct Environment {
        set: BTreeSet<usize>,
    }

    let environment = Environment {
        set: BTreeSet::new(),
    };

    // A test suite using the `suite`, `context`, `example` syntax family:
    runner.run(rspec::suite("a BTreeSet", environment.clone(), |ctx| {
        ctx.context("not having added any items", |ctx| {
            ctx.example("it is empty", |env| assert!(env.set.is_empty()));
        });
    }));

    // A test suite using the `describe`, `specify`, `it` syntax family:
    runner.run(rspec::describe("a BTreeSet", environment.clone(), |ctx| {
        ctx.specify("not having added any items", |ctx| {
            ctx.it("it is empty", |env| assert!(env.set.is_empty()));
        });
    }));

    // A test suite using the `given`, `when`, `then` syntax family:
    runner.run(rspec::given("a BTreeSet", environment.clone(), |ctx| {
        ctx.when("not having added any items", |ctx| {
            ctx.then("it is empty", |env| assert!(env.set.is_empty()));
        });
    }));
}
