extern crate rspec;

pub fn main() {
    // The easiest way to open a suite is by calling the `rspec::run(…)` function,
    // passing it the result of one of these functions:
    //
    // - `rspec::suite`,
    // - `rspec::describe`
    // - `rspec::given`
    //
    // which all behave the same and only differ in the label
    // that is printed the the est suite's log.
    //
    // One then passes the following arguments to aforementioned function:
    //
    // - a name (to add some more meaning to the runner's output)
    // - an initial value (to base the tests on)
    // - a closure (to provide the suite's test logic)
    rspec::run(&rspec::given("a value of zero", 0, |ctx| {
        ctx.then("it is zero", |value| {
            assert_eq!(*value, 0);
        });

        ctx.when("multiplying by itself", |ctx| {
            // Any time one wants to mutate the value being tested
            // one does so by calling `ctx.before(…)` (or `ctx.after(…)`),
            // which will execute the provided closure before any other
            // sub-context (e.g. `ctx.when(…)`) or example (e.g. `ctx.then(…)`)
            // is executed:
            ctx.before(|value| {
                *value *= *value;
            });

            ctx.then("it remains zero", |value| {
                assert_eq!(*value, 0);
            });
        });

        ctx.when("adding a value to it", |ctx| {
            ctx.before(|value| {
                *value += 42;
            });

            ctx.then("it becomes said value", |value| {
                assert_eq!(*value, 42);
            });
        });
    }));
}
