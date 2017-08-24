extern crate rspec;

use std::io;
use std::sync::Arc;
use std::collections::BTreeSet;

pub fn main() {
    let formatter = Arc::new(rspec::Formatter::new(io::stdout()));
    let configuration = rspec::Configuration::default();
    let runner = rspec::Runner::new(configuration, vec![formatter]);

    #[derive(Clone, Debug)]
    struct Environment {
        set: BTreeSet<usize>,
    }

    let environment = Environment {
        set: BTreeSet::new(),
    };

    runner.run(rspec::suite("a BTreeSet", environment.clone(), |ctx| {
        ctx.context("not having added any items", |ctx| {
            ctx.example("it is empty", |env| assert!(env.set.is_empty()));
        });
    }));

    runner.run(rspec::describe("a BTreeSet", environment.clone(), |ctx| {
        ctx.specify("not having added any items", |ctx| {
            ctx.it("it is empty", |env| assert!(env.set.is_empty()));
        });
    }));

    runner.run(rspec::given("a BTreeSet", environment.clone(), |ctx| {
        ctx.when("not having added any items", |ctx| {
            ctx.then("it is empty", |env| assert!(env.set.is_empty()));
        });
    })).or_exit();
}
