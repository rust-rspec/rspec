# rspec - a BDD test harness that works with stable Rust

[![Build Status](https://travis-ci.org/mackwic/rspec.svg?branch=master)](https://travis-ci.org/mackwic/rspec) [![Coverage Status](https://coveralls.io/repos/github/mackwic/rspec/badge.svg)](https://coveralls.io/github/mackwic/rspec) [![Crates.io](https://img.shields.io/crates/v/rspec.svg?maxAge=2592000)](https://crates.io/crates/rspec) [![Crates.io](https://img.shields.io/crates/l/rspec.svg?maxAge=2592000)](https://github.com/mackwic/rspec/blob/master/LICENSE)

Do you like writing your tests in a tree of `describe`, and `it`, but also have
a hard requirement on stable rust? This crate is for you!

If you don't know what is Rust, are confused by the terms BDD, TDD, or just want
a gently beginner introduction, [please go to the Beginner Section](#beginners).

The last stable documentation is available for consultation at
[mackwic.github.io/rspec](https://mackwic.github.io/rspec).

**All rspec releases are garanteed to compile against the latest stable rust and
are tested on all rust versions from the 1.9**.

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

You can also use rspec in integration tests, this example uses the rspec runner:

```rust
extern crate rspec;
use rspec::context::describe;
use std::io;

pub fn main() {
    let stdout = &mut io::stdout();
    let mut formatter = rspec::formatter::Simple::new(stdout);
    let mut runner = describe("rspec is a classic BDD testing", |ctx| {

        ctx.it("can define tests", || true);

        ctx.describe("rspec use results for tests results", |ctx| {

            ctx.it("passed if the return is_ok()", || Ok(()) as Result<(),()>);

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

```

Again, you can see complete examples in the [`examples/`](https://github.com/mackwic/rspec/tree/master/examples) directory.

The last stable documentation is available for consultation at
[mackwic.github.io/rspec](https://mackwic.github.io/rspec).

## Contributions

... are greatly welcome! Contributions follow the standard Github workflow,
which is:

1. Fork this repository
2. Create a feature branch
3. Code and commit inside this branch. I have a personnal preference for small
   atomic commits, but that's not a hard rule.
4. Make sure you have written tests for your feature. If you don't know how to
   do that, push the PR with `[WIP]` in the title and we'll gladly help you.
5. When tests are ok, and you features/bug fixes are covered, we will review the
   code together, make it better together.
6. When everyone agrees that the code is right, and the tests pass, you will
   have the privilege to merge your PR and become a mighty Contributor.
   Congrats.

Take a look at [the issues](https://github.com/mackwic/rspec/issues) if you want
to help without knowing how. Some issues are mentored!

## Contributors

- Thomas WICKHAM [@mackwic](https://github.com/mackwic)
- Pascal HERTLEIF [@killercup](https://github.com/killercup)
- Matthias BOURG [@pol0nium](https://github.com/pol0nium)

## Beginners

#### About Rust

Welcome in the Rust community! Here are some links which can hopefully help:

- [**Rust is a system programming language**](https://www.rust-lang.org). Check the
  rust-lang link for a detailed description, you can install it [with rustup](https://www.rustup.rs/).
- **Newcomers**: [Rust by Example](http://rustbyexample.com/) is a fantastic
  resource to get started and grasp most of the Rust semantic. Check it out!
- **Intermediate**: [The Rust Book](https://doc.rust-lang.org/book/) is the best
  resource to begin the journey to learn Rust.
- **Advanced**: [Learning Rust with too many linked lists](http://cglab.ca/~abeinges/blah/too-many-lists/book/) is a great book which explains by examples and with great details the system of ownership and how to use it. [The Rustonomicon](https://doc.rust-lang.org/nomicon/) Is another resoure helpful to understand how the compiler thinks and will help you converge quickly to a compilable code.

#### About TDD

TDD, short for Tests Driven Development, is a methodology of development where
your code is always working, where refactoring is easy, and you know exactly
what piece of code do _and what it doesn't_.

With TDD, legacy code is limited, your build is always green and you don't have
regresssions.

This is a wonderful and magical land, a well-keeped secret where only the best
of the best are admitted, people who don't compromise on quality because they
know the cost of the absence of quality. People who know that over-quality is
a non-sense.

Here are some useful links:

- [Uncle Bob's 3 rules of
  TDD](http://butunclebob.com/ArticleS.UncleBob.TheThreeRulesOfTdd)
- [The Art of Agile: Test Driven
  Development](http://www.jamesshore.com/Agile-Book/test_driven_development.html)
- TDD also enable [simple and emergeant
  designs](http://www.jamesshore.com/Agile-Book/simple_design.html) by reducing
  the number of decisions you have to take at each step.

#### About BDD

BDD, short for Behavior Driven Development, is a variation on TDD; some would
say that BDD is simply TDD but refined.

BDD states that tests are not the center of the methodology. They are one of the
most usefull tool available, but we should not look at them in too high regard.
What matters is the contract they seal, the _described behavior_.

Thus, there is enough tests when the behavior of the `struct` is sufficiently
described. Thinking in term of behavior has two benefits:

- When doing TDD, it helps to make incremential steps. Just write examples of
  how to use the functions, and make this example pass, then go to the next one.
  Your tests will naturally have one clear intent, and so will be easy to debug
  / rely on when refactoring.   
  This is the describe/it approach, which this crate hopes to fill.

- By describing behavior, we are doing an _analysis_ of our program. This
  analysis can be very useful! Say... an User Story, for example. Given the
  formalism _As a X, When Y, I want Z_, you can assign scenarii describing the
  high-level behavior of your units. The Gherkin formalism is often employed for
  this, it use a _Given X, When Y, Then Z_ structure.   
  This project does not aim to help on high-level BDD, see [the cucumber for
  Rust port](https://github.com/acmcarther/cucumbe://github.com/acmcarther/cucumber)
  for that.

The foundation of BDD [is well explained here](https://dannorth.net/introducing-bdd/)
and [also here](http://blog.daveastels.com.s3-website-us-west-2.amazonaws.com/2014/09/29/a-new-look-at-test-driven-development.html).

BDD written with the Gherkin formalism can really gain from a layer of DDD
(Domain Driven Development), [but this is another
story...](https://msdn.microsoft.com/en-us/magazine/dd419654.aspx).

## Licence

Mozilla Public Licence 2.0. See the LICENCE file at the root of the repository.

In non legal terms it means that:
- if you fix a bug, you MUST give back the code of the fix (it's only fair, see
  the [Contributing Section](#contributions)).
- if you change/extend the API, you MUST give back the code you changed in the
  files under MPL2. The [Contributing Section](#contributions) can help there.
- you CAN'T sue the authors of this project for anything about this code
- appart from that, you can do almost whatever you want. See the LICENCE file
  for details.

This section DOES NOT REPLACE NOR COMPLETE the LICENCE files. The LICENCE file
is the only place where the licence of tis project is defined.


