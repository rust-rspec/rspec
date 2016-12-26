extern crate rspec;

use std::io;
use std::collections::BTreeSet;

pub fn main() {
    let stdout = &mut io::stdout();
    let mut formatter = rspec::formatter::Simple::new(stdout);

    #[derive(Clone, Debug)]
    struct Environment {
        set: BTreeSet<usize>,
    }

    let environment = Environment {
        set: BTreeSet::new(),
    };

    let mut runner = rspec::given("a BTreeSet", environment, |ctx| {
        ctx.when("not having added any items", |ctx| {
            ctx.then("it is empty", |env| {
                assert!(env.set.is_empty());
            });

            ctx.then("its len is zero", |env| {
                assert_eq!(env.set.len(), 0);
            });
        });

        ctx.then("panic!(…) fails", move |_env| {
            panic!("Some reason for failure.")
        });
    });

    runner.add_event_handler(&mut formatter);
    runner.run_or_exit();
}
