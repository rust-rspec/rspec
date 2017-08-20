use suite::{suite, describe, given};

mod describe {
    pub use super::*;

    macro_rules! test_suite_alias {
            ($suite: ident) => {
                $suite("suite (or alias)", (), |_| {});
            }
        }

    #[test]
    fn it_has_root_functions() {
        test_suite_alias!(suite);
        test_suite_alias!(describe);
        test_suite_alias!(given);
    }

    macro_rules! test_context_alias {
            ($suite: ident, $context: ident) => {
                $suite("suite (or alias)", (), |ctx| {
                    ctx.$context("context (or alias)", |_| {})
                });
            }
        }

    #[test]
    fn it_has_contextual_function_context() {
        test_context_alias!(suite, context);
        test_context_alias!(describe, context);
        test_context_alias!(given, context);
    }

    #[test]
    fn it_has_contexual_function_specify() {
        test_context_alias!(suite, specify);
        test_context_alias!(describe, specify);
        test_context_alias!(given, specify);
    }

    #[test]
    fn it_has_contexual_function_when() {
        test_context_alias!(suite, when);
        test_context_alias!(describe, when);
        test_context_alias!(given, when);
    }

    macro_rules! test_example_alias {
            ($suite: ident, $context: ident, $example: ident) => {
                $suite("suite (or alias)", (), |ctx| {
                    ctx.$context("context (or alias)", |ctx| {
                        ctx.$example("example (or alias)", |_| {

                        })
                    })
                });
            }
        }

    #[test]
    fn it_has_check_function_example() {
        test_example_alias!(suite, context, example);
        test_example_alias!(suite, specify, example);
        test_example_alias!(suite, when, example);

        test_example_alias!(describe, context, example);
        test_example_alias!(describe, specify, example);
        test_example_alias!(describe, when, example);

        test_example_alias!(given, context, example);
        test_example_alias!(given, specify, example);
        test_example_alias!(given, when, example);
    }

    #[test]
    fn it_has_check_function_it() {
        test_example_alias!(suite, context, it);
        test_example_alias!(suite, specify, it);
        test_example_alias!(suite, when, it);

        test_example_alias!(describe, context, it);
        test_example_alias!(describe, specify, it);
        test_example_alias!(describe, when, it);

        test_example_alias!(given, context, it);
        test_example_alias!(given, specify, it);
        test_example_alias!(given, when, it);
    }

    #[test]
    fn it_has_check_function_then() {
        test_example_alias!(suite, context, then);
        test_example_alias!(suite, specify, then);
        test_example_alias!(suite, when, then);

        test_example_alias!(describe, context, then);
        test_example_alias!(describe, specify, then);
        test_example_alias!(describe, when, then);

        test_example_alias!(given, context, then);
        test_example_alias!(given, specify, then);
        test_example_alias!(given, when, then);
    }
}
