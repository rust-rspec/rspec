//! The Runner is where all the examples are actually executed.
//!
//! A Runner is instanciated by using [`context::describe`](../context/fn.describe.html) and
//! [`context::rdescribe`](../context/fn.rdescribe.html). You should not try to instanciate
//! a Runner directly.
//!
//! The main methods are `Runner::run` and `Runner::result`.

use std::mem;
use std::panic;

use example_report::ExampleReport;
use context_report::ContextReport;
use visitor::Visitor;
use events::{Event, EventHandler};

use suite::Suite;
use context::Context;
use example::Example;
use context::ContextMember;

/// Handlers is a separate struct which only holds the registered handlers.
/// This is useful to Runner so that its recursive call doesn't have to keep a refernce to `self`
#[derive(Default)]
struct Handlers<'a> {
    handlers: Vec<&'a mut EventHandler>,
}

impl<'a> Handlers<'a> {
    fn broadcast(&mut self, event: &Event) {
        for h in &mut self.handlers {
            h.trigger(event)
        }
    }
}

pub struct Runner<'a, T>
    where T: 'a
{
    suite: Option<Suite<'a, T>>,
    environments: Vec<T>,
    report: ContextReport,
    handlers: Handlers<'a>,
}

impl<'a, T> Runner<'a, T> {
    pub fn new(suite: Suite<'a, T>, environment: T) -> Runner<'a, T> {
        Runner {
            suite: Some(suite),
            environments: vec![environment],
            report: ContextReport::default(),
            handlers: Handlers::default(),
        }
    }
}

impl<'a, T> Runner<'a, T>
    where T: 'a + Clone + ::std::fmt::Debug
{
    pub fn run(mut self) -> ContextReport {
        let suite = mem::replace(&mut self.suite, None).expect("Expected context");
        panic::set_hook(Box::new(|_panic_info| {
            // silently swallows panics
        }));
        self.visit(&suite);
        let _ = panic::take_hook();
        self.report
    }

    pub fn run_or_exit(self) {
        if self.run().failed > 0 {
            ::std::process::exit(101);
        }
    }

    pub fn add_event_handler<H: EventHandler>(&mut self, handler: &'a mut H) {
        self.handlers.handlers.push(handler)
    }

    pub fn broadcast(&mut self, event: Event) {
        self.handlers.broadcast(&event)
    }

    pub fn push_environment(&mut self, environment: T) {
        self.environments.push(environment);
    }

    pub fn pop_environment(&mut self) -> Option<T> {
        self.environments.pop()
    }

    pub fn get_environment(&self) -> &T {
        let index = self.environments.len() - 1;
        &self.environments[index]
    }

    pub(crate) fn get_environment_mut(&mut self) -> &mut T {
        let index = self.environments.len() - 1;
        &mut self.environments[index]
    }
}

impl<'a, T> Visitor<Suite<'a, T>> for Runner<'a, T>
    where T: 'a + Clone + ::std::fmt::Debug
{
    type Output = ContextReport;

    fn visit(&mut self, suite: &Suite<'a, T>) -> Self::Output {
        self.broadcast(Event::EnterSuite(suite.info.clone()));
        let report = self.visit(&suite.context);
        self.broadcast(Event::ExitSuite(report.clone()));
        report
    }
}

impl<'a, T> Visitor<Context<'a, T>> for Runner<'a, T>
    where T: 'a + Clone + ::std::fmt::Debug
{
    type Output = ContextReport;

    fn visit(&mut self, context: &Context<'a, T>) -> Self::Output {
        let mut report = ContextReport::default();
        if let Some(ref info) = context.info {
            self.broadcast(Event::EnterContext(info.clone()));
        }
        if let Some(environment) = self.environments.last_mut() {
            for function in context.before_all.iter() {
                function(environment);
            }
        }
        for member in context.members.iter() {
            let environment = self.environments.last().unwrap().clone();
            self.push_environment(environment);
            if let Some(environment) = self.environments.last_mut() {
                for function in context.before_each.iter() {
                    function(environment);
                }
            }
            report.add(self.visit(member));
            if let Some(environment) = self.environments.last_mut() {
                for function in context.after_each.iter() {
                    function(environment);
                }
            }
            self.pop_environment();
        }
        if let Some(environment) = self.environments.last_mut() {
            for function in context.after_all.iter() {
                function(environment);
            }
        }
        if let Some(_) = context.info {
            self.broadcast(Event::ExitContext(report.clone()));
        }
        report
    }
}

impl<'a, T> Visitor<ContextMember<'a, T>> for Runner<'a, T>
    where T: 'a + Clone + ::std::fmt::Debug
{
    type Output = ContextReport;

    fn visit(&mut self, member: &ContextMember<'a, T>) -> Self::Output {
        match member {
            &ContextMember::Example(ref example) => {
                self.visit(example).into()
            }
            &ContextMember::Context(ref context) => {
                self.visit(context)
            }
        }
    }
}

impl<'a, T> Visitor<Example<'a, T>> for Runner<'a, T>
    where T: 'a + Clone + ::std::fmt::Debug
{
    type Output = ExampleReport;

    fn visit(&mut self, example: &Example<'a, T>) -> Self::Output {
        self.broadcast(Event::EnterExample(example.info.clone()));
        let function = &example.function;
        let report = function(&self.get_environment());
        self.broadcast(Event::ExitExample(report.clone()));
        report
    }
}
