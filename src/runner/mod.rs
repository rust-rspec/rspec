//! The Runner is where all the examples are actually executed.
//!
//! A Runner is instanciated by using [`context::describe`](../context/fn.describe.html) and
//! [`context::rdescribe`](../context/fn.rdescribe.html). You should not try to instanciate
//! a Runner directly.
//!
//! The main methods are `Runner::run` and `Runner::result`.

mod configuration;

pub use runner::configuration::Configuration;

use context::{Context, ContextMember, Example};
use events::{Event, EventHandler};
use rayon;
use rayon::prelude::*;
use report::context::ContextReport;
use report::suite::SuiteReport;
use report::example::ExampleReport;
use std::panic;
use std::sync::{Arc, Mutex};
use suite::Suite;
use visitor::Visitor;


pub struct Runner {
    configuration: configuration::Configuration,
    handlers: Vec<Arc<Mutex<EventHandler>>>,
}

impl Runner {
    pub fn new(configuration: Configuration, handlers: Vec<Arc<Mutex<EventHandler>>>) -> Runner {
        Runner {
            configuration: configuration,
            handlers: handlers,
        }
    }
}

impl Runner {

    pub fn run<T>(self, suite: (Suite<T>, T)) -> SuiteReport
    where
        T: Clone + Send + Sync + ::std::fmt::Debug,
    {
        let (suite, mut environment) = suite;
        self.prepare_before_run();
        let report = self.visit(&suite, &mut environment);
        self.clean_after_run();
        report
    }

    pub fn run_or_exit<T>(self, suite: (Suite<T>, T))
    where
        T: Clone + Send + Sync + ::std::fmt::Debug,
    {
        if self.run(suite).failed > 0 {
            // XXX Cargo test failure returns 101.
            //
            // > "We use 101 as the standard failure exit code because it's something unique
            // > that the test runner can check for in run-fail tests (as opposed to something
            // > like 1, which everybody uses). I don't expect this behavior can ever change.
            // > This behavior probably dates to before 2013,
            // > all the way back to the creation of compiletest." – @brson

            ::std::process::exit(101);
        }
    }

    fn broadcast(&self, event: Event) {
        for mutex in &self.handlers {
            if let Ok(ref mut handler) = mutex.lock() {
                handler.handle(&event);
            } else {
                println!("Error: lock failed");
            }
        }
    }

    fn prepare_before_run(&self) {
        panic::set_hook(Box::new(|_panic_info| {
            // XXX panics already catched at the test call site, don't output the trace in stdout
        }));
        let threads = if self.configuration.parallel { 0 } else { 1 }; // 0 is rayon default
        let rayon_config = rayon::Configuration::new().num_threads(threads);
        let _ = rayon::initialize(rayon_config);
    }

    fn clean_after_run(&self) {
        // XXX reset panic hook back to default hook:
        let _ = panic::take_hook();
    }
}

impl<T> Visitor<Suite<T>> for Runner
where
    T: Clone + Send + Sync + ::std::fmt::Debug,
{
    type Environment = T;
    type Output = ContextReport;

    fn visit(&self, suite: &Suite<T>, environment: &mut Self::Environment) -> Self::Output {
        self.broadcast(Event::EnterSuite(suite.info.clone()));
        let report = self.visit(&suite.context, environment);
        self.broadcast(Event::ExitSuite(report.clone()));
        report
    }
}

impl<T> Visitor<Context<T>> for Runner
where
    T: Clone + Send + Sync + ::std::fmt::Debug,
{
    type Environment = T;
    type Output = ContextReport;

    fn visit(&self, context: &Context<T>, environment: &mut Self::Environment) -> Self::Output {
        if let Some(ref info) = context.info {
            self.broadcast(Event::EnterContext(info.clone()));
        }
        for before_function in context.before_all.iter() {
            before_function(environment);
        }
        let report: ContextReport = context
            .members
            .par_iter()
            .map(|member| {
                let mut environment = environment.clone();
                for before_each_function in context.before_each.iter() {
                    before_each_function(&mut environment);
                }
                let report = match member {
                    &ContextMember::Example(ref example) => {
                        self.visit(example, &mut environment).into()
                    }
                    &ContextMember::Context(ref context) => {
                        self.visit(context, &mut environment.clone())
                    }
                };
                for after_each_function in context.after_each.iter() {
                    after_each_function(&mut environment);
                }
                report
            })
            .sum();
        for after_function in context.after_all.iter() {
            after_function(environment);
        }
        if let Some(_) = context.info {
            self.broadcast(Event::ExitContext(report.clone()));
        }
        report
    }
}

impl<T> Visitor<Example<T>> for Runner
where
    T: Clone + Send + Sync + ::std::fmt::Debug,
{
    type Environment = T;
    type Output = ExampleReport;

    fn visit(&self, example: &Example<T>, environment: &mut Self::Environment) -> Self::Output {
        self.broadcast(Event::EnterExample(example.info.clone()));
        let function = &example.function;
        let report = function(environment);
        self.broadcast(Event::ExitExample(report.clone()));
        report
    }
}
