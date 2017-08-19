//! The Runner is where all the examples are actually executed.
//!
//! A Runner is instanciated by using [`context::describe`](../context/fn.describe.html) and
//! [`context::rdescribe`](../context/fn.rdescribe.html). You should not try to instanciate
//! a Runner directly.
//!
//! The main methods are `Runner::run` and `Runner::result`.

use std::mem;
use std::panic;
use std::sync::{Arc, Mutex};

use rayon;
use rayon::prelude::*;

use context_member::ContextMember;
use example_report::ExampleReport;
use context_report::ContextReport;
use visitor::Visitor;
use events::{Event, EventHandler};

use suite::Suite;
use context::Context;
use example::Example;

pub struct Configuration {
    parallel: bool
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            parallel: true
        }
    }
}

pub struct Runner<'a, T>
    where T: 'a
{
    suite: Option<Suite<'a, T>>,
    environment: T,
    handlers: Vec<Arc<Mutex<EventHandler>>>,
}

impl<'a, T> Runner<'a, T> {
    pub fn new(suite: Suite<'a, T>, environment: T) -> Runner<'a, T> {
        Runner {
            suite: Some(suite),
            environment: environment,
            handlers: vec![],
        }
    }
}

impl<'a, T> Runner<'a, T>
    where T: 'a + Clone + Send + Sync + ::std::fmt::Debug
{
    pub fn run(self) -> ContextReport {
        self.run_with(&Configuration::default())
    }

    pub fn run_with(mut self, config: &Configuration) -> ContextReport {
        let suite = mem::replace(&mut self.suite, None).expect("Expected context");
        panic::set_hook(Box::new(|_panic_info| {
            // silently swallows panics
        }));
        let mut environment = self.environment.clone();
        let threads = if config.parallel { 0 } else  { 1 };
        let _ = rayon::initialize(rayon::Configuration::new().num_threads(threads));
        let report = self.visit(&suite, &mut environment);
        let _ = panic::take_hook();
        report
    }

    pub fn run_or_exit(self) {
        self.run_or_exit_with(&Configuration::default())
    }

    pub fn run_or_exit_with(self, config: &Configuration) {
        if self.run_with(config).failed > 0 {
            ::std::process::exit(101);
        }
    }

    pub fn add_event_handler(&mut self, handler: Arc<Mutex<EventHandler>>) {
        self.handlers.push(handler)
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
}

impl<'a, T> Visitor<Suite<'a, T>> for Runner<'a, T>
    where T: 'a + Clone + Send + Sync + ::std::fmt::Debug
{
    type Environment = T;
    type Output = ContextReport;

    fn visit(&self, suite: &Suite<'a, T>, environment: &mut Self::Environment) -> Self::Output {
        self.broadcast(Event::EnterSuite(suite.info.clone()));
        let report = self.visit(&suite.context, environment);
        self.broadcast(Event::ExitSuite(report.clone()));
        report
    }
}

impl<'a, T> Visitor<Context<'a, T>> for Runner<'a, T>
    where T: 'a + Clone + Send + Sync + ::std::fmt::Debug
{
    type Environment = T;
    type Output = ContextReport;

    fn visit(&self, context: &Context<'a, T>, environment: &mut Self::Environment) -> Self::Output {
        if let Some(ref info) = context.info {
            self.broadcast(Event::EnterContext(info.clone()));
        }
        for function in context.before_all.iter() {
            function(environment);
        }
        let report: ContextReport = context.members.par_iter().map(|member| {
            let mut environment = environment.clone();
            for function in context.before_each.iter() {
                function(&mut environment);
            }
            let report = match member {
                &ContextMember::Example(ref example) => {
                    self.visit(example, &mut environment).into()
                }
                &ContextMember::Context(ref context) => {
                    self.visit(context, &mut environment.clone())
                }
            };
            for function in context.after_each.iter() {
                function(&mut environment);
            }
            report
        }).sum();
        for function in context.after_all.iter() {
            function(environment);
        }
        if let Some(_) = context.info {
            self.broadcast(Event::ExitContext(report.clone()));
        }
        report
    }
}

impl<'a, T> Visitor<Example<'a, T>> for Runner<'a, T>
    where T: 'a + Clone + Send + Sync + ::std::fmt::Debug
{
    type Environment = T;
    type Output = ExampleReport;

    fn visit(&self, example: &Example<'a, T>, environment: &mut Self::Environment) -> Self::Output {
        self.broadcast(Event::EnterExample(example.info.clone()));
        let function = &example.function;
        let report = function(environment);
        self.broadcast(Event::ExitExample(report.clone()));
        report
    }
}
