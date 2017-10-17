extern crate rspec;

use std::collections::BTreeSet;

pub fn main() {
    #[derive(Clone, Debug)]
    struct Environment {
        set: BTreeSet<usize>,
        len_before: usize,
    }

    let environment = Environment {
        set: BTreeSet::new(),
        len_before: 0,
    };

    rspec::run(&rspec::given("a BTreeSet", environment, |ctx| {
        ctx.when("not having added any items", |ctx| {
            ctx.then("it is empty", |env| assert!(env.set.is_empty()));
        });

        ctx.when("adding an new item", |ctx| {
            ctx.before_all(|env| {
                env.len_before = env.set.len();
                env.set.insert(42);
            });

            ctx.then("it is not empty any more", |env| {
                assert!(!env.set.is_empty());
            });

            ctx.then("its len increases by 1", |env| {
                assert_eq!(env.set.len(), env.len_before + 1);
            });

            ctx.when("adding it again", |ctx| {
                ctx.before_all(|env| {
                    env.len_before = env.set.len();
                    env.set.insert(42);
                });

                ctx.then("its len remains the same", |env| {
                    assert_eq!(env.set.len(), env.len_before);
                });
            });
        });

        ctx.when("returning to outer context", |ctx| {
            ctx.then("it is still empty", |env| assert!(env.set.is_empty()));
        });

        ctx.then("panic!(…) fails", |_env| {
            panic!("Some reason for failure.")
        });
    }));
}
