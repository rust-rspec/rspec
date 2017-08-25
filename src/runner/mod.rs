//! The Runner is where all the examples are actually executed.
//!
//! A Runner is instanciated by using [`context::describe`](../context/fn.describe.html) and
//! [`context::rdescribe`](../context/fn.rdescribe.html). You should not try to instanciate
//! a Runner directly.
//!
//! The main methods are `Runner::run` and `Runner::result`.

pub mod configuration;

pub use runner::configuration::{Configuration, ConfigurationBuilder};

use std::fmt;
use std::panic;
use std::sync::{Arc, Mutex};
use std::borrow::Borrow;
use std::cell::Cell;
use std::process;
use std::ops::{Deref, DerefMut};

use colored::*;
use rayon::prelude::*;

use block::Block;
use block::suite::Suite;
use block::context::Context;
use block::example::Example;
use event_handler::EventHandler;
use report::BlockReport;
use report::context::ContextReport;
use report::suite::SuiteReport;
use report::example::ExampleReport;
use visitor::TestSuiteVisitor;

pub struct Runner {
    configuration: configuration::Configuration,
    handlers: Vec<Arc<EventHandler>>,
    should_exit: Mutex<Cell<bool>>,
}

impl Runner {
    pub fn new(configuration: Configuration, handlers: Vec<Arc<EventHandler>>) -> Runner {
        Runner {
            configuration: configuration,
            handlers: handlers,
            should_exit: Mutex::new(Cell::new(false)),
        }
    }
}

impl Runner {
    pub fn run<T>(&self, suite: (Suite<T>, T)) -> SuiteReport
    where
        T: Clone + Send + Sync + ::std::fmt::Debug,
    {
        let (suite, mut environment) = suite;
        self.prepare_before_run();
        let report = self.visit(&suite, &mut environment);
        self.clean_after_run();
        if let Ok(mut mutex_guard) = self.should_exit.lock() {
            *mutex_guard.deref_mut().get_mut() |= report.is_failure();
        }
        report
    }

    fn broadcast<F, U, V>(&self, mut f: F)
    where
        F: FnMut(&EventHandler) -> Result<U, V>,
        U: fmt::Debug,
        V: fmt::Debug
    {
        for event_handler in &self.handlers {
            if let Err(error) = f(event_handler.borrow()) {
                eprintln!("\n{}: {:?}", "error".red().bold(), error);
            }
        }
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

impl Drop for Runner {
    fn drop(&mut self) {
        let should_exit = if let Ok(mutex_guard) = self.should_exit.lock() {
            mutex_guard.deref().get()
        } else { false };

        if self.configuration.exit_on_failure && should_exit {
            // XXX Cargo test failure returns 101.
            //
            // > "We use 101 as the standard failure exit code because it's something unique
            // > that the test runner can check for in run-fail tests (as opposed to something
            // > like 1, which everybody uses). I don't expect this behavior can ever change.
            // > This behavior probably dates to before 2013,
            // > all the way back to the creation of compiletest." – @brson

            process::exit(101);
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
        self.broadcast(|handler| handler.enter_suite(&suite.header));
        let report = SuiteReport::new(suite.header.clone(), self.visit(&suite.context, environment));
        self.broadcast(|handler| handler.exit_suite(&suite.header, &report));
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
            self.broadcast(|handler| handler.enter_context(&header));
        }
        for before_function in context.before_all.iter() {
            before_function(environment);
        }
        let process_member = |runner: &Runner, member: &Block<T>, mut environment: T| {
                for before_each_function in context.before_each.iter() {
                    before_each_function(&mut environment);
                }
            let report = runner.visit(member, &mut environment);
                for after_each_function in context.after_each.iter() {
                    after_each_function(&mut environment);
                }
                report
        };
        let reports: Vec<_> = if self.configuration.parallel {
            context
                .blocks
                .par_iter()
                .map(|member| {
                    process_member(self, member, environment.clone())
            }).collect()
        } else {
            context
                .blocks
                .iter()
                .map(|member| {
                    process_member(self, member, environment.clone())
                })
                .collect()
        };
        for after_function in context.after_all.iter() {
            after_function(environment);
        }
        let report = ContextReport::new(reports);
        if let Some(ref header) = context.header {
            self.broadcast(|handler| handler.exit_context(&header, &report));
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
        self.broadcast(|handler| handler.enter_example(&example.header));
        let function = &example.function;
        let report = function(environment);
        self.broadcast(|handler| handler.exit_example(&example.header, &report));
        report
    }
}
