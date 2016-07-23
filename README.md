# rspec - a BDD test harness that works with stable Rust

[![Build Status](https://travis-ci.org/mackwic/rspec.svg?branch=master)](https://travis-ci.org/mackwic/rspec) [![Coverage Status](https://coveralls.io/repos/github/mackwic/rspec/badge.svg)](https://coveralls.io/github/mackwic/rspec) [![Crates.io](https://img.shields.io/crates/v/rspec.svg?maxAge=2592000)](https://crates.io/crates/rspec) [![Crates.io](https://img.shields.io/crates/l/rspec.svg?maxAge=2592000)]()

Do you like writing your tests in a tree of `describe`, and `it`, but also have
a hard requirement on stable rust ? This crate is for you !

If you don't know what is Rust, are confused by the terms BDD, TDD, or just want
a gently beginner introduction, [please go to the Beginner Section](#Beginners).

The last stable documentation is available for consultation at
[mackwic.github.io/rspec](https://mackwic.github.io/rspec).

## How to use

Add this in your `Cargo.toml`:

```toml
[dev_dependencies]
rspec = "1.0"
```

and add this to your `src/lib.rs` or `src/main.rs`:

```rust
#[cfg(test)]
extern crate rspec;
```

You can see complete examples in the [`examples/`](https://github.com/mackwic/rspec/tree/master/examples) directory.

You can now use rspec in your unit tests, this example will use the `cargo test`
runner:

```rust
fn add(x: u32, y: u32) -> u64 {
    x + y
}

#[test]
fn test_add() {
    rdescribe("add", |ctx| {
        ctx.describe("0 <= x + y <= u32::MAX", |ctx| {
            ctx.it("2 + 4 = 6", || {
                assert_eq!(6, add(2, 4)); Ok(())
            });

            ctx.it("4 + 4 = 8", || {
                assert_eq!(8, add(4, 4)); Ok(())
            });
        });

        ctx.it("is associative", || {
            assert_eq!(add(2, 1), add(1, 2));
            assert_eq!(add(4, 1), add(1, 4));
            assert_eq!(add(4, 5), add(5, 4));
            assert_eq!(add(12, 1), add(1, 12));
            Ok(())
        });
    });
}
```

You can also use rspec in integration tests, this example use the rspec runner:

```rust
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

```

Again, you can see complete examples in the [`examples/`](https://github.com/mackwic/rspec/tree/master/examples) directory.

The last stable documentation is available for consultation at
[mackwic.github.io/rspec](https://mackwic.github.io/rspec).


## Beginners

Welcome in the Rust community ! Here are some links which can hopefully help:

- [**Rust**](https://www.rust-lang.org) is a system programming language. Check the
  rust-lang link for a detailed description, you can install it [with rustup](https://www.rustup.rs/) !
- **Newcomers**: [Rust by Example](http://rustbyexample.com/) is a fantastic
  resource to get started and grasp most of the Rust semantic. Check it out !
- **Intermediate**: [The Rust Book](https://doc.rust-lang.org/book/) is the best
  resource to begin the journey to learn Rust.
- **Advanced**: [Learning Rust with too many linked lists](http://cglab.ca/~abeinges/blah/too-many-lists/book/) is a great book which explains by examples and with great details the system of ownership and how to use it. [The Rustonomicon](https://doc.rust-lang.org/nomicon/) Is another resoure helpful to understand how the compiler thinks and will help you converge quickly to a compilable code.

## Contributions

... are greatly welcome ! Contributions follow the standard Github workflow,
which is:

1. Fork this repository
2. Create a feature branch
3. Code and commit inside this branch. I have a personnal preference for small
   atomic commits, but that's not a hard rule.
4. Make sure you have written tests for your feature. If you don't know how to
   do that, push the PR with `[WIP]` in the title and we gladly help you.
5. When tests are ok, and you features/bug fixes are covered, we will review the
   code together, make it better together.
6. When everyone agrees that the code is right, and the tests pass, you will
   have the privilege to merge your PR and become a mighty Contributor.
   Congrats.

## Contributors

- Thomas Wickham: [@mackwic](https://github.com/mackwic)
- Pascal Hertleif [@killercup](https://github.com/killercup)

## Licence

Mozilla Public Licence 2.0. See the LICENCE file at the root of the repository.

In non legal terms it means that:
- if you fix a bug, you MUST give back the code of the fix (it's only fair, see
  the [Contributing Section](#Contributing)).
- if you change/extend the API, you MUST give back the code you changed in the
  files under MPL2. The [Contributing Section](#Contributing) can help there.
- you CAN'T sue the authors of this project for anything about this code
- appart from that, you can do almost whatever you want. See the LICENCE file
  for details.

This section DOES NOT REPLACE NOR COMPLETE the LICENCE files. The LICENCE file
is the only place where the licence of tis project is defined.


