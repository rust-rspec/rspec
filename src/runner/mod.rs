//! Runners are responsible for executing a test suite's examples.

mod configuration;
mod observer;

pub use runner::configuration::*;
pub use runner::observer::*;

use std::borrow::Borrow;
use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use std::panic;
#[cfg(not(test))]
use std::process;
use std::sync::{Arc, Mutex};

use time::PreciseTime;

use rayon::prelude::*;

use block::Block;
use block::Suite;
use block::Context;
use block::Example;
use report::{Report, BlockReport};
use report::ContextReport;
use report::SuiteReport;
use report::ExampleReport;
use visitor::TestSuiteVisitor;

/// Runner for executing a test suite's examples.
pub struct Runner {
    pub configuration: configuration::Configuration,
    observers: Vec<Arc<RunnerObserver>>,
    should_exit: Mutex<Cell<bool>>,
}

impl Runner {
    pub fn new(configuration: Configuration, observers: Vec<Arc<RunnerObserver>>) -> Runner {
        Runner {
            configuration: configuration,
            observers: observers,
            should_exit: Mutex::new(Cell::new(false)),
        }
    }
}

impl Runner {
    pub fn run<T>(&self, suite: &Suite<T>) -> SuiteReport
    where
        T: Clone + Send + Sync + ::std::fmt::Debug,
    {
        let mut environment = suite.environment.clone();
        self.prepare_before_run();
        let report = self.visit(suite, &mut environment);
        self.clean_after_run();
        if let Ok(mut mutex_guard) = self.should_exit.lock() {
            *mutex_guard.deref_mut().get_mut() |= report.is_failure();
        }
        report
    }

    fn broadcast<F>(&self, mut handler: F)
    where
        F: FnMut(&RunnerObserver),
    {
        for observer in &self.observers {
            handler(observer.borrow());
        }
    }

    fn wrap_all<T, U, F>(&self, context: &Context<T>, environment: &mut T, wrapped_block: F) -> U
    where
        F: Fn(&mut T) -> U,
    {
        for before_function in context.before_all.iter() {
            before_function(environment);
        }
        let result = wrapped_block(environment);
        for after_function in context.after_all.iter() {
            after_function(environment);
        }
        result
    }

    fn wrap_each<T, U, F>(&self, context: &Context<T>, environment: &mut T, wrapped_block: F) -> U
    where
        F: Fn(&mut T) -> U,
    {
        for before_function in context.before_each.iter() {
            before_function(environment);
        }
        let result = wrapped_block(environment);
        for after_function in context.after_each.iter() {
            after_function(environment);
        }
        result
    }

    fn evaluate_blocks_parallel<T>(&self, context: &Context<T>, environment: &T) -> Vec<BlockReport>
    where
        T: Clone + Send + Sync + ::std::fmt::Debug,
    {
        context
            .blocks
            .par_iter()
            .map(|block| self.evaluate_block(block, context, environment))
            .collect()
    }

    fn evaluate_blocks_serial<T>(&self, context: &Context<T>, environment: &T) -> Vec<BlockReport>
    where
        T: Clone + Send + Sync + ::std::fmt::Debug,
    {
        context
            .blocks
            .iter()
            .map(|block| self.evaluate_block(block, context, environment))
            .collect()
    }

    fn evaluate_block<T>(
        &self,
        block: &Block<T>,
        context: &Context<T>,
        environment: &T,
    ) -> BlockReport
    where
        T: Clone + Send + Sync + ::std::fmt::Debug,
    {
        let mut environment = environment.clone();
        self.wrap_each(context, &mut environment, |environment| {
            self.visit(block, environment)
        })
    }

    fn prepare_before_run(&self) {
        panic::set_hook(Box::new(|_panic_info| {
            // XXX panics already catched at the test call site, don't output the trace in stdout
        }));
    }

    fn clean_after_run(&self) {
        // XXX reset panic hook back to default hook:
        let _ = panic::take_hook();
    }
}

impl Default for Runner {
    /// Default Runner to help testing
    fn default() -> Self {
        Runner::new(Configuration::default(), vec!())
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        let should_exit = if let Ok(mutex_guard) = self.should_exit.lock() {
            mutex_guard.deref().get()
        } else {
            false
        };

        if self.configuration.exit_on_failure && should_exit {
            // XXX Cargo test failure returns 101.
            //
            // > "We use 101 as the standard failure exit code because it's something unique
            // > that the test runner can check for in run-fail tests (as opposed to something
            // > like 1, which everybody uses). I don't expect this behavior can ever change.
            // > This behavior probably dates to before 2013,
            // > all the way back to the creation of compiletest." – @brson
            #[cfg(not(test))]
            process::exit(101);
            #[cfg(test)]
            panic!("test suite failed !")
        }
    }
}


impl<T> TestSuiteVisitor<Suite<T>> for Runner
where
    T: Clone + Send + Sync + ::std::fmt::Debug,
{
    type Environment = T;
    type Output = SuiteReport;

    fn visit(&self, suite: &Suite<T>, environment: &mut Self::Environment) -> Self::Output {
        self.broadcast(|handler| handler.enter_suite(self, &suite.header));
        let report = SuiteReport::new(
            suite.header.clone(),
            self.visit(&suite.context, environment),
        );
        self.broadcast(|handler| handler.exit_suite(self, &suite.header, &report));
        report
    }
}

impl<T> TestSuiteVisitor<Block<T>> for Runner
where
    T: Clone + Send + Sync + ::std::fmt::Debug,
{
    type Environment = T;
    type Output = BlockReport;

    fn visit(&self, member: &Block<T>, environment: &mut Self::Environment) -> Self::Output {
        match member {
            &Block::Example(ref example) => {
                let header = example.header.clone();
                let report = self.visit(example, environment);
                BlockReport::Example(header, report)
            }
            &Block::Context(ref context) => {
                let header = context.header.clone();
                let report = self.visit(context, &mut environment.clone());
                BlockReport::Context(header, report)
            }
        }
    }
}

impl<T> TestSuiteVisitor<Context<T>> for Runner
where
    T: Clone + Send + Sync + ::std::fmt::Debug,
{
    type Environment = T;
    type Output = ContextReport;

    fn visit(&self, context: &Context<T>, environment: &mut Self::Environment) -> Self::Output {
        if let Some(ref header) = context.header {
            self.broadcast(|handler| handler.enter_context(self, &header));
        }
        let start_time = PreciseTime::now();
        let reports: Vec<_> =
            self.wrap_all(context, environment, |environment| if self.configuration
                .parallel
            {
                self.evaluate_blocks_parallel(context, environment)
            } else {
                self.evaluate_blocks_serial(context, environment)
            });
        let end_time = PreciseTime::now();
        let report = ContextReport::new(reports, start_time.to(end_time));
        if let Some(ref header) = context.header {
            self.broadcast(|handler| handler.exit_context(self, &header, &report));
        }
        report
    }
}

impl<T> TestSuiteVisitor<Example<T>> for Runner
where
    T: Clone + Send + Sync + ::std::fmt::Debug,
{
    type Environment = T;
    type Output = ExampleReport;

    fn visit(&self, example: &Example<T>, environment: &mut Self::Environment) -> Self::Output {
        self.broadcast(|handler| handler.enter_example(self, &example.header));
        let start_time = PreciseTime::now();
        let result = (example.function)(environment);
        let end_time = PreciseTime::now();
        let report = ExampleReport::new(result, start_time.to(end_time));
        self.broadcast(|handler| handler.exit_example(self, &example.header, &report));
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod runner {
        use super::*;

        #[test]
        fn it_can_be_instanciated() {
            // arrange
            let _ = Runner::new(Configuration::default(), vec!());
            // act
            // assert
        }

        mod broadcast {
            use super::*;

            use header::*;
            use std::sync::atomic::*;

            // XXX blank impl for stubbing
            impl RunnerObserver for () {}

            #[test]
            fn it_calls_the_closure() {
                // arrange
                let spy = Arc::new(());
                let runner = Runner::new(Configuration::default(), vec!(spy));
                let has_been_called = AtomicBool::new(false);
                // act
                runner.broadcast(|_| has_been_called.store(true, Ordering::SeqCst));
                // assert
                assert_eq!(true, has_been_called.load(Ordering::SeqCst));
            }

            #[test]
            fn it_calls_it_once_per_observer() {
                // arrange
                let spy1 = Arc::new(());
                let spy2 = Arc::new(());
                let runner = Runner::new(Configuration::default(), vec![spy1, spy2]);
                let call_times = AtomicUsize::new(0);
                // act
                runner.broadcast(|_| { call_times.fetch_add(1, Ordering::SeqCst); });
                // assert
                assert_eq!(2, call_times.load(Ordering::SeqCst))
            }

            struct ObserverStub {
                events: Mutex<Vec<(&'static str, SuiteHeader)>>,
            }
            impl ObserverStub {
                fn new() -> Self {
                    ObserverStub { events: Mutex::new(vec!()) }
                }
            }

            // XXX stub implem
            impl RunnerObserver for ObserverStub {
                fn enter_suite(&self, _runner: &Runner, header: &SuiteHeader) {
                    let mut vec = self.events.lock().unwrap();
                    (*vec).push(("enter_suite", header.clone()));
                }
            }

            #[test]
            fn it_gives_the_observer_as_callback_argument() {
                // arrange
                let spy1 = Arc::new(ObserverStub::new());
                let expected = SuiteHeader::new(SuiteLabel::Describe, "hello");
                let runner = Runner::new(Configuration::default(), vec![spy1.clone()]);
                // act
                runner.broadcast(|observer| observer.enter_suite(&runner, &expected.clone()));
                // assert
                let lock = spy1.events.lock().expect("no dangling threads");
                let res = (*lock).get(0).expect("to have been called once");
                assert_eq!(&("enter_suite", expected), res);
            }
        }

        mod wrap_each {
            use super::*;

            use std::sync::atomic::*;

            #[test]
            fn it_can_be_called() {
                // arrange
                let runner = Runner::default();
                // act
                runner.wrap_each(&Context::default(), &mut (), |_| {})
                // assert
            }

            #[test]
            fn it_calls_the_closure() {
                // arrange
                let runner = Runner::default();
                let has_been_called = AtomicBool::new(false);
                // act
                runner.wrap_each(&Context::default(), &mut (), |_| has_been_called.store(true, Ordering::SeqCst));
                // assert
                assert_eq!(true, has_been_called.load(Ordering::SeqCst));
            }

            #[test]
            fn it_calls_the_before_each_callbacks() {
                // arrange
                let runner = Runner::default();
                let has_been_called = Arc::new(AtomicBool::new(false));
                let closure_bool_handler = has_been_called.clone();
                let mut context = Context::default();
                // act
                context.before_each(move |_| closure_bool_handler.store(true, Ordering::SeqCst));
                runner.wrap_each(&context, &mut (), |_| ());
                // assert
                assert_eq!(true, has_been_called.load(Ordering::SeqCst));
            }

            #[test]
            fn it_calls_the_after_each_callbacks() {
                // arrange
                let runner = Runner::default();
                let has_been_called = Arc::new(AtomicBool::new(false));
                let closure_bool_handler = has_been_called.clone();
                let mut context = Context::default();
                // act
                context.after_each(move |_| closure_bool_handler.store(true, Ordering::SeqCst));
                runner.wrap_each(&context, &mut (), |_| ());
                // assert
                assert_eq!(true, has_been_called.load(Ordering::SeqCst));
            }

            #[test]
            fn it_calls_all_before_each_callbacks() {
                // arrange
                let runner = Runner::default();
                let call_counter = Arc::new(AtomicUsize::new(0));
                let closure_counter_handler1 = call_counter.clone();
                let closure_counter_handler2 = call_counter.clone();
                let mut context = Context::default();
                // act
                context.before_each(move |_| { closure_counter_handler1.fetch_add(1, Ordering::SeqCst); });
                context.before_each(move |_| { closure_counter_handler2.fetch_add(1, Ordering::SeqCst); });
                runner.wrap_each(&context, &mut (), |_| ());
                // assert
                assert_eq!(2, call_counter.load(Ordering::SeqCst));
            }

            #[test]
            fn it_calls_all_after_each_callbacks() {
                // arrange
                let runner = Runner::default();
                let call_counter = Arc::new(AtomicUsize::new(0));
                let closure_counter_handler1 = call_counter.clone();
                let closure_counter_handler2 = call_counter.clone();
                let mut context = Context::default();
                // act
                context.after_each(move |_| { closure_counter_handler1.fetch_add(1, Ordering::SeqCst); });
                context.after_each(move |_| { closure_counter_handler2.fetch_add(1, Ordering::SeqCst); });
                runner.wrap_each(&context, &mut (), |_| ());
                // assert
                assert_eq!(2, call_counter.load(Ordering::SeqCst));
            }

            #[test]
            fn it_calls_before_each_hook_before_the_main_closure() {
                // arrange
                let runner = Runner::default();
                let last_caller_id = Arc::new(AtomicUsize::new(0));
                let last_caller_handler1 = last_caller_id.clone();
                let last_caller_handler2 = last_caller_id.clone();
                let mut context = Context::default();
                // act
                context.before_each(move |_| { last_caller_handler1.store(1, Ordering::SeqCst); });
                runner.wrap_each(&context, &mut (), |_| { last_caller_handler2.store(2, Ordering::SeqCst); });
                // assert
                assert_eq!(2, last_caller_id.load(Ordering::SeqCst));
            }

            #[test]
            fn it_calls_after_each_hook_after_the_main_closure() {
                // arrange
                let runner = Runner::default();
                let last_caller_id = Arc::new(AtomicUsize::new(0));
                let last_caller_handler1 = last_caller_id.clone();
                let last_caller_handler2 = last_caller_id.clone();
                let mut context = Context::default();
                // act
                context.after_each(move |_| { last_caller_handler1.store(1, Ordering::SeqCst); });
                runner.wrap_each(&context, &mut (), |_| { last_caller_handler2.store(2, Ordering::SeqCst); });
                // assert
                assert_eq!(1, last_caller_id.load(Ordering::SeqCst));
            }
        }

        mod wrap_all {
            use super::*;

            use std::sync::atomic::*;

            #[test]
            fn it_can_be_called() {
                // arrange
                let runner = Runner::default();
                // act
                runner.wrap_all(&Context::default(), &mut (), |_| {})
                // assert
            }

            #[test]
            fn it_calls_the_closure() {
                // arrange
                let runner = Runner::default();
                let has_been_called = AtomicBool::new(false);
                // act
                runner.wrap_all(&Context::default(), &mut (), |_| has_been_called.store(true, Ordering::SeqCst));
                // assert
                assert_eq!(true, has_been_called.load(Ordering::SeqCst));
            }

            #[test]
            fn it_calls_the_before_all_callbacks() {
                // arrange
                let runner = Runner::default();
                let has_been_called = Arc::new(AtomicBool::new(false));
                let closure_bool_handler = has_been_called.clone();
                let mut context = Context::default();
                // act
                context.before_all(move |_| closure_bool_handler.store(true, Ordering::SeqCst));
                runner.wrap_all(&context, &mut (), |_| ());
                // assert
                assert_eq!(true, has_been_called.load(Ordering::SeqCst));
            }

            #[test]
            fn it_calls_the_after_all_callbacks() {
                // arrange
                let runner = Runner::default();
                let has_been_called = Arc::new(AtomicBool::new(false));
                let closure_bool_handler = has_been_called.clone();
                let mut context = Context::default();
                // act
                context.after_all(move |_| closure_bool_handler.store(true, Ordering::SeqCst));
                runner.wrap_all(&context, &mut (), |_| ());
                // assert
                assert_eq!(true, has_been_called.load(Ordering::SeqCst));
            }

            #[test]
            fn it_calls_all_before_all_callbacks() {
                // arrange
                let runner = Runner::default();
                let call_counter = Arc::new(AtomicUsize::new(0));
                let closure_counter_handler1 = call_counter.clone();
                let closure_counter_handler2 = call_counter.clone();
                let mut context = Context::default();
                // act
                context.before_all(move |_| { closure_counter_handler1.fetch_add(1, Ordering::SeqCst); });
                context.before_all(move |_| { closure_counter_handler2.fetch_add(1, Ordering::SeqCst); });
                runner.wrap_all(&context, &mut (), |_| ());
                // assert
                assert_eq!(2, call_counter.load(Ordering::SeqCst));
            }

            #[test]
            fn it_calls_all_after_all_callbacks() {
                // arrange
                let runner = Runner::default();
                let call_counter = Arc::new(AtomicUsize::new(0));
                let closure_counter_handler1 = call_counter.clone();
                let closure_counter_handler2 = call_counter.clone();
                let mut context = Context::default();
                // act
                context.after_all(move |_| { closure_counter_handler1.fetch_add(1, Ordering::SeqCst); });
                context.after_all(move |_| { closure_counter_handler2.fetch_add(1, Ordering::SeqCst); });
                runner.wrap_all(&context, &mut (), |_| ());
                // assert
                assert_eq!(2, call_counter.load(Ordering::SeqCst));
            }

            #[test]
            fn it_calls_before_all_hook_before_the_main_closure() {
                // arrange
                let runner = Runner::default();
                let last_caller_id = Arc::new(AtomicUsize::new(0));
                let last_caller_handler1 = last_caller_id.clone();
                let last_caller_handler2 = last_caller_id.clone();
                let mut context = Context::default();
                // act
                context.before_all(move |_| { last_caller_handler1.store(1, Ordering::SeqCst); });
                runner.wrap_all(&context, &mut (), |_| { last_caller_handler2.store(2, Ordering::SeqCst); });
                // assert
                assert_eq!(2, last_caller_id.load(Ordering::SeqCst));
            }

            #[test]
            fn it_calls_after_all_hook_after_the_main_closure() {
                // arrange
                let runner = Runner::default();
                let last_caller_id = Arc::new(AtomicUsize::new(0));
                let last_caller_handler1 = last_caller_id.clone();
                let last_caller_handler2 = last_caller_id.clone();
                let mut context = Context::default();
                // act
                context.after_all(move |_| { last_caller_handler1.store(1, Ordering::SeqCst); });
                runner.wrap_all(&context, &mut (), |_| { last_caller_handler2.store(2, Ordering::SeqCst); });
                // assert
                assert_eq!(1, last_caller_id.load(Ordering::SeqCst));
            }
        }
    }

    mod impl_drop_for_runner {
        use super::*;

        #[test]
        #[should_panic]
        fn it_should_abort() {
            // arrange
            let config = ConfigurationBuilder::default()
                .exit_on_failure(true)
                .build()
                .unwrap();
            // act
            {
                let runner = Runner::new(config, vec!());
                (*runner.should_exit.lock().unwrap()).set(true);
            }
            // assert
            // test should panic
        }
    }

    mod impl_visitor_example_for_runner {
        use super::*;

        use header::*;
        use report::*;
        use std::sync::atomic::*;

        #[derive(Default, Debug, Clone)]
        struct SpyObserver {
            enter_example: Arc<AtomicBool>,
            exit_example: Arc<AtomicBool>,
        }
        impl RunnerObserver for SpyObserver {
            fn enter_example(&self, _runner: &Runner, _header: &ExampleHeader) {
               self.enter_example.store(true, Ordering::SeqCst)
            }

            fn exit_example(&self, _runner: &Runner, _header: &ExampleHeader, _report: &ExampleReport) {
                self.exit_example.store(true, Ordering::SeqCst)
            }
        }

        #[test]
        fn it_can_be_called() {
            // arrange
            let runner = Runner::default();
            let example = Example::fixture_success();
            // act
            // assert
            runner.visit(&example, &mut ());
        }

        #[test]
        fn it_calls_observer_hooks() {
            // arrange
            let spy = Arc::new(SpyObserver::default());
            let runner = Runner::new(Configuration::default(), vec![spy.clone()]);
            let example = Example::fixture_success();
            // act
            runner.visit(&example, &mut ());
            // assert
            assert!(true == spy.enter_example.load(Ordering::SeqCst));
            assert!(true == spy.exit_example.load(Ordering::SeqCst))
        }

        #[test]
        fn it_gives_an_env_to_the_example() {
            // arrange
            let runner = Runner::default();
            let mut environment = Arc::new(AtomicBool::new(false));
            // act
            let example = Example::new(ExampleHeader::default(), |env : &Arc<AtomicBool>| {
                env.store(true, Ordering::SeqCst);
                ExampleResult::Success
            });
            runner.visit(&example, &mut environment);
            // assert
            assert_eq!(true, environment.load(Ordering::SeqCst));
        }
    }

    mod impl_visitor_block_for_runner {
        use super::*;

        #[test]
        fn it_can_be_called() {
            // arrange
            let runner = Runner::default();
            let block = Block::Example(Example::fixture_success());
            // act
            // assert
            runner.visit(&block, &mut ());
        }
    }

}
