//! Events are sent by the Runner to signal the progression in the test suite, with the results

use std::io;

use header::suite::SuiteHeader;
use header::context::ContextHeader;
use header::example::ExampleHeader;
use report::suite::SuiteReport;
use report::context::ContextReport;
use report::example::ExampleReport;

pub trait EventHandler: Send + Sync {
    fn enter_suite(&self, suite: &SuiteHeader) -> io::Result<()>;
    fn exit_suite(&self, suite: &SuiteHeader, report: &SuiteReport) -> io::Result<()>;
    fn enter_context(&self, context: &ContextHeader) -> io::Result<()>;
    fn exit_context(&self, context: &ContextHeader, _report: &ContextReport) -> io::Result<()>;
    fn enter_example(&self, example: &ExampleHeader) -> io::Result<()>;
    fn exit_example(&self, example: &ExampleHeader, report: &ExampleReport) -> io::Result<()>;
}
