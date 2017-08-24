use std::io;
use std::sync::Mutex;
use std::ops::DerefMut;

use colored::*;

use header::Header;
use header::suite::SuiteHeader;
use header::context::ContextHeader;
use header::example::ExampleHeader;
use event_handler::EventHandler;
use report::{Report, BlockReport};
use report::suite::SuiteReport;
use report::context::ContextReport;
use report::example::ExampleReport;

struct FormatterState<T: io::Write = io::Stdout> {
    buffer: T,
    level: usize,
}

impl<T: io::Write> FormatterState<T> {
    pub fn new(buffer: T) -> Self {
        FormatterState {
            buffer: buffer,
            level: 0,
        }
    }
}

pub struct Formatter<T: io::Write = io::Stdout> {
    state: Mutex<FormatterState<T>>,
}

impl Default for Formatter<io::Stdout> {
    fn default() -> Self {
        Formatter::new(io::stdout())
    }
}

impl<T: io::Write> Formatter<T> {
    pub fn new(buffer: T) -> Self {
        let state = FormatterState::new(buffer);
        Formatter {
            state: Mutex::new(state),
        }
    }

    fn padding(depth: usize) -> String {
        "  ".repeat(depth)
    }

    fn access_state<F>(&self, mut f: F) -> io::Result<()>
    where
        F: FnMut(&mut FormatterState<T>) -> io::Result<()>
    {
        if let Ok(ref mut mutex_guard) = self.state.lock() {
            f(mutex_guard.deref_mut())?;
        } else {
            eprintln!("\n{}: failed to aquire lock on mutex.", "error".red().bold());
        }

        Ok(())
    }

    fn write_header<H>(&self, buffer: &mut T, indent: usize, header: &H) -> io::Result<()>
    where
        H: Header
    {
        let padding = Self::padding(indent);
        write!(buffer, "{}{} {:?}", padding, header.label(), header.name())?;

        Ok(())
    }

    fn write_suite_failures(&self, buffer: &mut T, indent: usize, report: &SuiteReport) -> io::Result<()> {
        if report.is_failure() {
            let _ = writeln!(buffer, "\nfailures:\n");
            self.write_header(buffer, indent, report.get_header())?;
            writeln!(buffer)?;
            let context_report = report.get_context();
            for block_report in context_report.get_blocks() {
                self.write_block_failures(buffer, indent + 1, block_report)?;
            }
        }

        Ok(())
    }

    fn write_block_failures(&self, buffer: &mut T, indent: usize, report: &BlockReport) -> io::Result<()> {
        if report.is_failure() {
            match report {
                &BlockReport::Context(ref header, ref report) => {
                    if let Some(header) = header.as_ref() {
                        self.write_header(buffer, indent, header)?;
                    }
                    self.write_context_failures(buffer, indent + 1, report)?;
                },
                &BlockReport::Example(ref header, ref report) => {
                    self.write_header(buffer, indent, header)?;
                    writeln!(buffer)?;
                    self.write_example_failure(buffer, indent + 1, report)?;
                },
            }
        }
        Ok(())
    }

    fn write_context_failures(&self, buffer: &mut T, indent: usize, report: &ContextReport) -> io::Result<()> {
        if report.is_failure() {
            writeln!(buffer)?;
            for block_report in report.get_blocks() {
                self.write_block_failures(buffer, indent + 1, block_report)?;
            }
        }

        Ok(())
    }

    fn write_example_failure(&self, buffer: &mut T, indent: usize, report: &ExampleReport) -> io::Result<()> {
        if let &ExampleReport::Failure(Some(ref reason)) = report {
            let padding = Self::padding(indent);
            writeln!(buffer, "{}{}", padding, reason)?;
        }
        Ok(())
    }

    fn write_suite_prefix(&self, buffer: &mut T) -> io::Result<()> {
        writeln!(buffer, "\ntests:\n")?;

        Ok(())
    }

    fn write_suite_suffix(&self, buffer: &mut T, report: &SuiteReport) -> io::Result<()> {
        let flag = self.report_flag(report);
        write!(buffer, "\ntest result: {}.", flag)?;
        writeln!(
            buffer,
            " {} passed; {} failed; {} ignored",
            report.get_passed(), report.get_failed(), report.get_ignored()
        )?;

        if report.is_failure() {
            writeln!(buffer, "\n{}: test failed", "error".red().bold())?;
        }

        Ok(())
    }

    fn report_flag<R>(&self, report: &R) -> ColoredString
    where
        R: Report
    {
        if report.is_success() {
            "ok".green()
        } else {
            "FAILED".red()
        }
    }
}

impl<T: io::Write> EventHandler for Formatter<T>
where
    T: Send + Sync,
{
    fn enter_suite(&self, suite: &SuiteHeader) -> io::Result<()> {
        self.access_state(|state| {
            state.level += 1;
            let indentation = state.level - 1;
            self.write_suite_prefix(&mut state.buffer)?;
            self.write_header(&mut state.buffer, indentation, suite)?;
            writeln!(state.buffer)?;

            Ok(())
        })
    }

    fn exit_suite(&self, _suite: &SuiteHeader, report: &SuiteReport) -> io::Result<()> {
        self.access_state(|state| {
            self.write_suite_failures(&mut state.buffer, 0, report)?;
            self.write_suite_suffix(&mut state.buffer, report)?;

            state.level -= 1;

            Ok(())
        })
    }

    fn enter_context(&self, context: &ContextHeader) -> io::Result<()> {
        self.access_state(|state| {
            state.level += 1;
            let indentation = state.level - 1;

            self.write_header(&mut state.buffer, indentation, context)?;
            writeln!(state.buffer)?;

            Ok(())
        })
    }

    fn exit_context(&self, _context: &ContextHeader, _report: &ContextReport) -> io::Result<()> {
        self.access_state(|state| {
            state.level -= 1;

            Ok(())
        })
    }

    fn enter_example(&self, example: &ExampleHeader) -> io::Result<()> {
        self.access_state(|state| {
            state.level += 1;
            let indentation = state.level - 1;
            self.write_header(&mut state.buffer, indentation, example)?;
            write!(state.buffer, " ... ")?;

            Ok(())
        })
    }

    fn exit_example(&self, _example: &ExampleHeader, report: &ExampleReport) -> io::Result<()> {
        self.access_state(|state| {
            writeln!(state.buffer, "{}", self.report_flag(report))?;
            state.level -= 1;

            Ok(())
        })
    }
}

// #[cfg(test)]
// mod tests {
//     pub use super::*;
//     pub use event_handler::{Event, EventHandler};
//     pub use example_report::*;
//     pub use std::io;
//     pub use std::str;
//
//     #[test]
//     fn it_can_be_instanciated() {
//         Formatter::new(&mut vec![1u8]);
//     }
//
//     #[test]
//     fn it_impl_formatter_trait() {
//         let _: &Formatter = &Formatter::new(&mut vec![1u8]) as &Formatter;
//     }
//
//     mod event_start_runner {
//         pub use super::*;
//
//         #[test]
//         fn it_display_that_examples_started() {
//             let mut v = vec![];
//             {
//                 let mut s = Formatter::new(&mut v);
//                 s.handle(&Event::EnterSuite);
//             }
//
//             assert_eq!("\ntests", str::from_utf8(&v).unwrap());
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
//                             let mut s = Formatter::new(&mut sink);
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
//                 let mut s = Formatter::new(&mut v);
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
//                 let mut s = Formatter::new(&mut v);
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
//             let mut s = Formatter::new(&mut sink);
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
//             let mut s = Formatter::new(&mut sink);
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
//             let mut s = Formatter::new(&mut sink);
//             s.handle(&Event::EnterExample("hola".into()));
//             s.handle(&Event::ExitExample(FAILED_RES));
//             assert_eq!(1, s.failed.len());
//         }
//
//         #[test]
//         fn it_keep_track_of_the_failure_name() {
//             let mut sink = &mut io::sink();
//             let mut s = Formatter::new(&mut sink);
//             s.handle(&Event::EnterExample("hola".into()));
//             s.handle(&Event::ExitExample(FAILED_RES));
//             assert_eq!(Some(&"hola".into()), s.failed.get(0));
//         }
//
//         #[test]
//         fn it_has_a_nice_diplay_for_describes() {
//             let mut sink = &mut io::sink();
//             let mut s = Formatter::new(&mut sink);
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
//             let mut s = Formatter::new(&mut sink);
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
//             let mut s = Formatter::new(&mut sink);
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
//             let mut s = Formatter::new(&mut sink);
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
//                 let mut s = Formatter::new(&mut sink);
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
//                 let mut s = Formatter::new(&mut sink);
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
//                 let mut s = Formatter::new(&mut sink);
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
