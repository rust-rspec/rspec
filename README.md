# rspec - a BDD test harness that works with stable Rust

[![Build Status](https://travis-ci.org/rust-rspec/rspec.svg?branch=master)](https://travis-ci.org/rust-rspec/rspec) [![Coverage Status](https://coveralls.io/repos/github/rust-rspec/rspec/badge.svg)](https://coveralls.io/github/rust-rspec/rspec) [![Crates.io](https://img.shields.io/crates/v/rspec.svg?maxAge=2592000)](https://crates.io/crates/rspec) [![Crates.io](https://img.shields.io/crates/l/rspec.svg?maxAge=2592000)](https://github.com/rust-rspec/rspec/blob/master/LICENSE)

When you like BDD, and all the nested `describe/context/it` way of testing, but
you also like when your code compiles every day 👌.

If you don't know what is Rust, are confused by the terms BDD, TDD, or just want
a gently beginner introduction, [please go to the Beginner Section](#beginners).

The last stable documentation is available for consultation at
[docs.rs/rspec](https://docs.rs/rspec).

**All rspec releases are garanteed to compile against the latest stable rust and
are tested on all rust versions from the 1.19**.

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

You can see complete examples in the [`examples/`](https://github.com/rust-rspec/rspec/tree/master/examples) directory.


```rust
extern crate rspec;

pub fn main() {
    // Use a local struct to provide the test contexts with an environment.
    // The environment will contain the subject that is to be tested
    // along with any additional data you might need during the test run:
    #[derive(Clone, Default, Debug)]
    struct Environment {
        // ...
    }

    // `rspec::run(…)` is a convenience wrapper that takes care of setting up
    // a runner, logger, configuration and running the test suite for you.
    // If you want more direct control, you can manually set up those things, too.
    rspec::run(&rspec::describe("rspec, a BDD testing framework", Environment::default(), |ctx| {
        // `describe`, or any of its equivalents, opens the root context
        // of your test suite. Within you can then either define test examples:
        ctx.it("can define top-level tests", |_| true);

        // or make use of sub-contexts to add some structure to your test suite:
        ctx.specify("contexts give your tests structure and reduce redundancy", |ctx| {
            ctx.before(|_| {
                // Executed once, before any of the contexts/examples is entered.
            });

            ctx.after(|_| {
                // Executed once, after all of the contexts/examples have been exited.
            });

            ctx.specify("rspec can handle results", |ctx| {
                ctx.it("passes if the return is_ok()", |_| Ok(()) as Result<(),()>);
                ctx.it("failes if the return is_err()", |_| Err(()) as Result<(),()>);
            });

            ctx.specify("rspec can handle bools", |ctx| {
                ctx.it("should pass if true", |_| true);
                ctx.it("should fail if false", |_| false);
                ctx.it("is convenient for comparisons", |_| (42 % 37 + 2) > 3);
            });

            ctx.specify("rspec can handle units", |ctx| {
                ctx.it("should pass if the return is ()", |_| {});
            });

            ctx.specify("rspec can handle panics", |ctx| {
                ctx.it("is convenient for asserts", |_| assert_eq!(1, 1));
            });
        });
    })); // exits the process with a failure code if one of the tests failed.
}
```

### Suites, Contexts & Examples

rspec provides three variants for each of the structural elements:

|           | Variant A  | Variant B  | Variant C  |
|-----------|------------|------------|------------|
| Suites:   | `suite`    | `describe` | `given`    |
| Contexts: | `context`  | `specify`  | `when`     |
| Examples: | `example`  | `it`       | `then`     |

**Note:** While the intended use is to stick to a single variant per test suite
it is possible to freely mix structural elements across variants.

#### Variant A: `suite`, `context` & `example`

```rust
runner.run(&rspec::suite("opens a suite", /* environment */, |ctx| {
    ctx.context("opens a context", |ctx| {
        ctx.example("opens an example", |env| /* test condition */ );
    });
}));
```

#### Variant B: `describe`, `specify` & `it`

```rust
runner.run(&rspec::describe("opens a suite", /* environment */, |ctx| {
    ctx.specify("opens a context", |ctx| {
        ctx.it("opens an example", |env| /* test condition */ );
    });
}));
```

#### Variant C: `given`, `when` & `then`

```rust
runner.run(&rspec::given("opens a suite", /* environment */, |ctx| {
    ctx.when("opens a context", |ctx| {
        ctx.then("opens an example", |env| /* test condition */ );
    });
}));
```

### Before & After

|         | All                   | Each          |
|---------|-----------------------|---------------|
| Before: | `before`/`before_all` | `before_each` |
| After:  | `after` /`after_all`  | `after_each`  |

#### All

The "All" variants of before and after blocks are executed once upon
entering (or exiting, respectively) the given context.

#### Each

`before_each` and `after_each` blocks are executed once before each of the
given context's sub-contexts or examples.

### More Examples

Again, you can see complete examples in the [`examples/`](https://github.com/rust-rspec/rspec/tree/master/examples) directory.

## Documentation

The last stable documentation is available for consultation at
[https://docs.rs/rspec](https://docs.rs/rspec).

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

Take a look at [the issues](https://github.com/rust-rspec/rspec/issues) if you want
to help without knowing how. Some issues are mentored!

## Contributors

- Thomas WICKHAM [@mackwic](https://github.com/mackwic)
- Pascal HERTLEIF [@killercup](https://github.com/killercup)
- Matthias BOURG [@pol0nium](https://github.com/pol0nium)
- Vincent ESCHE [@regexident](https://github.com/regexident)

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
is the only place where the licence of this project is defined.
