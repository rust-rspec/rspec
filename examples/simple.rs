extern crate rspec;

use std::io;
use std::sync::{Arc, Mutex};
use std::collections::BTreeSet;

pub fn main() {
    let simple = rspec::formatter::Simple::new(io::stdout());
    let formatter = Arc::new(Mutex::new(simple));

    #[derive(Clone, Debug)]
    struct Environment {
        set: BTreeSet<usize>,
        len_before: usize,
    }

    let environment = Environment {
        set: BTreeSet::new(),
        len_before: 0,
    };

    let mut runner = rspec::given("a BTreeSet", environment, |ctx| {
        ctx.when("not having added any items", |ctx| {
            ctx.then("it is empty", |env| {
                assert!(env.set.is_empty())
            });
        });

        ctx.when("adding an new item", |ctx| {
            ctx.before_all(|env| {
                env.len_before = env.set.len();
                env.set.insert(42);
            });

            ctx.then("it is not empty any more", |env| {
                // println!("\n👉  it is not empty any more");
                // use std::time::Duration;
                // use std::thread;
                // thread::sleep(Duration::from_millis(4000));
                // println!("it is not empty any more  👈");

                assert!(!env.set.is_empty());
            });

            ctx.then("its len increases by 1", move |env| {
                // println!("\n👉  its len increases by 1");
                // use std::time::Duration;
                // use std::thread;
                // thread::sleep(Duration::from_millis(4000));
                // println!("its len increases by 1  👈");

                assert_eq!(env.set.len(), env.len_before + 1);
            });

            ctx.when("adding it again", |ctx| {
                ctx.before_all(|env| {
                    env.len_before = env.set.len();
                    env.set.insert(42);
                });

                ctx.then("its len remains the same", move |env| {
                    assert_eq!(env.set.len(), env.len_before);
                });
            });
        });

        ctx.when("returning to outer context", |ctx| {
            ctx.then("it is still empty", |env| {
                assert!(env.set.is_empty())
            });
        });

        ctx.then("panic!(…) fails", move |_env| {
            panic!("Some reason for failure.")
        });
    });

    runner.add_event_handler(formatter);
    runner.run_or_exit();
}
