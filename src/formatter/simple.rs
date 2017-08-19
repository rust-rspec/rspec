use std::io;
use std::mem;

use colored::*;

use events::{Event, EventHandler};
use formatter::formatter::Formatter;
use context_report::ContextReport;
use example_report::ExampleReport;

use suite::SuiteInfo;
use context::ContextInfo;
use example::ExampleInfo;

#[derive(Clone, Debug)]
enum ScopeInfo {
    Suite(SuiteInfo),
    Context(ContextInfo),
    Example(ExampleInfo),
}

pub struct Simple<T: io::Write> {
    buffer: T,
    name_stack: Vec<ScopeInfo>,
    failed: Vec<Vec<ScopeInfo>>,
}

impl<T: io::Write> Simple<T> {
    pub fn new(buffer: T) -> Simple<T> {
        Simple {
            buffer: buffer,
            name_stack: vec![],
            failed: vec![],
        }
    }

    fn padding(depth: usize) -> String {
        "  ".repeat(depth)
    }

    fn enter_suite(&mut self, info: &SuiteInfo) {
        self.name_stack.push(ScopeInfo::Suite(info.clone()));
        let _ = writeln!(self.buffer, "\nrunning tests");
        let label: &str = info.label.into();
        let _ = writeln!(self.buffer, "{} {:?}:", label, info.name);
    }

    fn exit_suite(&mut self, report: &ContextReport) {
        let _ = writeln!(self.buffer, "\nfailures:");
        let failed = mem::replace(&mut self.failed, vec![]);
        for scope_stack in failed {
            for (indent, scope) in scope_stack.into_iter().enumerate() {
                match scope {
                    ScopeInfo::Suite(info) => {
                        let padding = Self::padding(indent);
                        let label: &str = info.label.into();
                        let _ = writeln!(self.buffer, "{}{} {:?}:", padding, label, info.name);
                    }
                    ScopeInfo::Context(info) => {
                        let padding = Self::padding(indent);
                        let label: &str = info.label.into();
                        let _ = writeln!(self.buffer, "{}{} {:?}:", padding, label, info.name);
                    }
                    ScopeInfo::Example(info) => {
                        let padding = Self::padding(indent);
                        let label: &str = info.label.into();
                        if let Some(failure) = info.failure {
                            let _ = writeln!(self.buffer, "{}{} {:?}:", padding, label, info.name);
                            let padding = Self::padding(indent + 1);
                            let _ = writeln!(self.buffer, "{}{}", padding, failure);
                        } else {
                            let _ = writeln!(self.buffer, "{}{} {:?}", padding, label, info.name);
                        }
                    }
                }
            }
        }

        let label = if report.failed == 0 {
            "ok".green()
        } else {
            "FAILED".red()
        };
        let _ = write!(self.buffer, "\ntest result: {}.", label);
        let _ = write!(self.buffer, " {} passed", report.passed);
        let _ = write!(self.buffer, "; {} failed", report.failed);
        let _ = write!(self.buffer, "; {} ignored", report.ignored);
        let _ = write!(self.buffer, "; {} measured", report.measured);
        let _ = writeln!(self.buffer, "");

        if report.failed > 0 {
            let _ = writeln!(self.buffer, "\n{}: test failed", "error".red().bold());
        }
    }

    fn enter_context(&mut self, info: &ContextInfo) {
        self.name_stack.push(ScopeInfo::Context(info.clone()));

        let indent = self.name_stack.len() - 1;
        let _ = write!(self.buffer, "{}", Self::padding(indent));

        let label: &str = info.label.into();
        let _ = writeln!(self.buffer, "{} {:?}:", label, info.name);
    }

    fn exit_context(&mut self, _report: &ContextReport) {
        self.name_stack.pop();
    }

    fn enter_example(&mut self, info: &ExampleInfo) {
        self.name_stack.push(ScopeInfo::Example(info.clone()));

        let indent = self.name_stack.len() - 1;
        let _ = write!(self.buffer, "{}", Self::padding(indent));

        let label: &str = info.label.into();
        let _ = write!(self.buffer, "{} {:?}", label, info.name);
        let _ = write!(self.buffer, " ... ");
    }

    fn exit_example(&mut self, result: &ExampleReport) {
        if let &ExampleReport::Failure(ref failure) = result {
            if let Some(&mut ScopeInfo::Example(ref mut test_info)) = self.name_stack.last_mut() {
                test_info.failure = failure.message.clone();
            }
            if !self.name_stack.is_empty() {
                self.failed.push(self.name_stack.clone());
            }
        }
        let label = if result.is_ok() {
            "ok".green()
        } else {
            "FAILED".red()
        };
        let _ = writeln!(self.buffer, "{}", label);
        self.name_stack.pop();
    }
}

impl<T: io::Write> EventHandler for Simple<T>
    where T: Send + Sync
{
    fn handle(&mut self, event: &Event) {
        match *event {
            Event::EnterSuite(ref name) => {
                self.enter_suite(name);
            }
            Event::ExitSuite(ref report) => {
                self.exit_suite(report);
            }
            Event::EnterContext(ref name) => {
                self.enter_context(name);
            }
            Event::ExitContext(ref report) => {
                self.exit_context(report);
            }
            Event::EnterExample(ref name) => {
                self.enter_example(name);
            }
            Event::ExitExample(ref result) => {
                self.exit_example(result);
            }
        };
    }
}

impl<T: io::Write> Formatter for Simple<T>
    where T: Send + Sync
{}

// #[cfg(test)]
// mod tests {
//     pub use super::*;
//     pub use formatter::formatter::Formatter;
//     pub use events::{Event, EventHandler};
//     pub use example_report::*;
//     pub use std::io;
//     pub use std::str;
//
//     #[test]
//     fn it_can_be_instanciated() {
//         Simple::new(&mut vec![1u8]);
//     }
//
//     #[test]
//     fn it_impl_formatter_trait() {
//         let _: &Formatter = &Simple::new(&mut vec![1u8]) as &Formatter;
//     }
//
//     mod event_start_runner {
//         pub use super::*;
//
//         #[test]
//         fn it_display_that_examples_started() {
//             let mut v = vec![];
//             {
//                 let mut s = Simple::new(&mut v);
//                 s.handle(&Event::EnterSuite);
//             }
//
//             assert_eq!("\nrunning tests", str::from_utf8(&v).unwrap());
//         }
//     }
//
//     mod event_finished_runner {
//         pub use super::*;
//         use runner::ContextReport;
//
//         macro_rules! test_and_compare_output {
//             ($(
//                 $test_name:ident : (passed: $succ:expr, failed: $fail:expr) => $msg:expr
//             ),+) => {
//
//                 $(
//                     #[test]
//                     fn $test_name() {
//                         let mut sink = io::sink();
//                         let res = {
//                             let mut s = Simple::new(&mut sink);
//                             s.write_summary(ContextReport {
//                                 passed: $succ,
//                                 failed: $fail,
//                                 ignored: 0,
//                                 measured: 0,
//                             })
//                         };
//
//                         assert_eq!($msg, res)
//                     }
//                 )+
//             }
//         }
//
//         test_and_compare_output! {
//             no_example_is_ok: (passed: 0, failed: 0) =>
//                 "\n\ntest result: ok. 0 examples; 0 passed; 0 failed;",
//             one_example: (passed: 1, failed: 0) =>
//                 "\n\ntest result: ok. 1 examples; 1 passed; 0 failed;",
//             multiple_ok: (passed: 42, failed: 0) =>
//                 "\n\ntest result: ok. 42 examples; 42 passed; 0 failed;",
//             one_error: (passed: 0, failed: 1) =>
//               "\n\ntest result: FAILED. 1 examples; 0 passed; 1 failed;",
//             multiple_errors: (passed: 0, failed: 37) =>
//               "\n\ntest result: FAILED. 37 examples; 0 passed; 37 failed;",
//             one_of_each: (passed: 1, failed: 1) =>
//               "\n\ntest result: FAILED. 2 examples; 1 passed; 1 failed;",
//             multiple_of_each: (passed: 12, failed: 21) =>
//               "\n\ntest result: FAILED. 33 examples; 12 passed; 21 failed;"
//         }
//     }
//
//     mod event_end_example {
//         pub use super::*;
//
//         #[test]
//         fn it_displays_a_dot_when_success() {
//             let mut v = vec![];
//             {
//                 let mut s = Simple::new(&mut v);
//                 s.handle(&Event::ExitExample(SUCCESS_RES))
//             }
//
//             assert_eq!(".", str::from_utf8(&v).unwrap());
//         }
//
//         #[test]
//         #[allow(non_snake_case)]
//         fn it_displays_a_F_when_error() {
//             let mut v = vec![];
//             {
//                 let mut s = Simple::new(&mut v);
//                 s.handle(&Event::ExitExample(FAILED_RES))
//             }
//
//             assert_eq!("F", str::from_utf8(&v).unwrap());
//         }
//     }
//
//     mod event_start_end_describe {
//         pub use super::*;
//
//         #[test]
//         fn start_describe_event_push_the_name_stack() {
//             let mut sink = &mut io::sink();
//             let mut s = Simple::new(&mut sink);
//
//             s.handle(&Event::EnterContext(String::from("Hey !")));
//             assert_eq!(vec![String::from("Hey !")], s.name_stack);
//
//             s.handle(&Event::EnterContext(String::from("Ho !")));
//             assert_eq!(vec![String::from("Hey !"), String::from("Ho !")],
//                        s.name_stack)
//         }
//
//         #[test]
//         fn end_describe_event_pop_the_name_stack() {
//             let mut sink = &mut io::sink();
//             let mut s = Simple::new(&mut sink);
//
//             s.handle(&Event::EnterContext(String::from("Hey !")));
//             s.handle(&Event::EnterContext(String::from("Ho !")));
//
//             s.handle(&Event::ExitContext);
//             assert_eq!(vec![String::from("Hey !")], s.name_stack);
//
//             s.handle(&Event::ExitContext);
//             assert_eq!(0, s.name_stack.len());
//         }
//     }
//
//     mod failures_pretty_printing {
//         use super::*;
//
//         #[test]
//         fn it_register_failures() {
//             let mut sink = &mut io::sink();
//             let mut s = Simple::new(&mut sink);
//             s.handle(&Event::EnterExample("hola".into()));
//             s.handle(&Event::ExitExample(FAILED_RES));
//             assert_eq!(1, s.failed.len());
//         }
//
//         #[test]
//         fn it_keep_track_of_the_failure_name() {
//             let mut sink = &mut io::sink();
//             let mut s = Simple::new(&mut sink);
//             s.handle(&Event::EnterExample("hola".into()));
//             s.handle(&Event::ExitExample(FAILED_RES));
//             assert_eq!(Some(&"hola".into()), s.failed.get(0));
//         }
//
//         #[test]
//         fn it_has_a_nice_diplay_for_describes() {
//             let mut sink = &mut io::sink();
//             let mut s = Simple::new(&mut sink);
//             s.handle(&Event::EnterContext("hola".into()));
//             s.handle(&Event::EnterExample("holé".into()));
//             s.handle(&Event::ExitExample(FAILED_RES));
//             assert_eq!(Some(&"hola | holé".into()), s.failed.get(0));
//
//             s.handle(&Event::EnterContext("ohééé".into()));
//             s.handle(&Event::EnterExample("holé".into()));
//             s.handle(&Event::ExitExample(FAILED_RES));
//             assert_eq!(Some(&"hola | ohééé | holé".into()), s.failed.get(1));
//         }
//
//         #[test]
//         fn it_works_with_multiple_describes() {
//             let mut sink = &mut io::sink();
//             let mut s = Simple::new(&mut sink);
//             s.handle(&Event::EnterContext("hola".into()));
//             s.handle(&Event::EnterExample("holé".into()));
//             s.handle(&Event::ExitExample(FAILED_RES));
//
//             s.handle(&Event::ExitContext);
//             s.handle(&Event::EnterContext("ok".into()));
//             s.handle(&Event::EnterExample("cacao".into()));
//             s.handle(&Event::ExitExample(FAILED_RES));
//             assert_eq!(Some(&"ok | cacao".into()), s.failed.get(1));
//         }
//
//         #[test]
//         fn it_doesnt_includes_success() {
//             let mut sink = &mut io::sink();
//             let mut s = Simple::new(&mut sink);
//             s.handle(&Event::EnterContext("hola".into()));
//             s.handle(&Event::EnterExample("holé".into()));
//             s.handle(&Event::ExitExample(SUCCESS_RES));
//
//             assert_eq!(None, s.failed.get(0));
//         }
//
//         #[test]
//         fn is_doesnt_keep_examples_in_name_stack() {
//             let mut sink = &mut io::sink();
//             let mut s = Simple::new(&mut sink);
//             s.handle(&Event::EnterContext("hola".into()));
//             s.handle(&Event::EnterExample("holé".into()));
//             s.handle(&Event::ExitExample(SUCCESS_RES));
//             s.handle(&Event::EnterExample("holé".into()));
//             s.handle(&Event::ExitExample(FAILED_RES));
//
//             // not "hola | holé | holé"
//             assert_eq!(Some(&"hola | holé".into()), s.failed.get(0));
//         }
//
//         #[test]
//         fn format_all_failures_one_error() {
//             let mut sink = &mut io::sink();
//             let res = {
//                 let mut s = Simple::new(&mut sink);
//                 s.handle(&Event::EnterContext("hola".into()));
//                 s.handle(&Event::EnterExample("holé".into()));
//                 s.handle(&Event::ExitExample(FAILED_RES));
//                 s.failures_summary()
//             };
//
//             assert_eq!("  1) hola | holé\n", res);
//         }
//
//         #[test]
//         fn format_all_failures() {
//             let mut sink = &mut io::sink();
//             let res = {
//                 let mut s = Simple::new(&mut sink);
//                 s.handle(&Event::EnterContext("hola".into()));
//                 s.handle(&Event::EnterExample("holé".into()));
//                 s.handle(&Event::ExitExample(FAILED_RES));
//                 s.handle(&Event::EnterExample("hola".into()));
//                 s.handle(&Event::ExitExample(FAILED_RES));
//                 s.failures_summary()
//             };
//
//             assert_eq!("  1) hola | holé\n  2) hola | hola\n", res);
//
//             let res = {
//                 let mut s = Simple::new(&mut sink);
//                 s.handle(&Event::EnterContext("hola".into()));
//                 s.handle(&Event::EnterExample("holé".into()));
//                 s.handle(&Event::ExitExample(FAILED_RES));
//                 s.handle(&Event::ExitContext);
//                 s.handle(&Event::EnterContext("second".into()));
//                 s.handle(&Event::EnterContext("third".into()));
//                 s.handle(&Event::EnterExample("hola".into()));
//                 s.handle(&Event::ExitExample(FAILED_RES));
//                 s.failures_summary()
//             };
//
//             assert_eq!("  1) hola | holé\n  2) second | third | hola\n", res);
//         }
//     }
// }
