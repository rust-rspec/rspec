use events::{Event, EventHandler};
use formatter::formatter::Formatter;
use runner;
use std::io;

pub struct Simple<'a, Io: io::Write + 'a> {
    buf: &'a mut Io,
    pub name_stack: Vec<String>,
    pub failures: Vec<String>,
}

impl<'a, T: io::Write> Simple<'a, T> {
    pub fn new(buf: &mut T) -> Simple<T> {
        Simple {
            buf: buf,
            name_stack: vec![],
            failures: vec![],
        }
    }

    fn failures_summary(&self) -> String {
        let res = String::with_capacity(100);
        let mut idx = 0;
        self.failures
            .iter()
            .map(|fail| {
                idx += 1;
                format!("  {}) {}\n", idx, fail)
            })
            .fold(res, |mut acc, elt| {
                acc.push_str(&elt);
                acc
            })
    }

    fn write_summary(&mut self, result: runner::RunnerResult) -> String {
        let (res, report) = match result {
            Ok(report) => ("ok", report),
            Err(report) => ("FAILED", report),
        };

        format!("\n\ntest result: {}. {} examples; {} passed; {} failed;",
                res,
                report.total_tests,
                report.success_count,
                report.error_count)
    }
}

impl<'a, T: io::Write> EventHandler for Simple<'a, T> {
    fn trigger(&mut self, event: &Event) {
        // FIXME: do something with the io::Error ?
        let _ = match *event {
            Event::StartRunner => writeln!(self.buf, "Running tests:\n"),
            Event::StartDescribe(ref name) |
            Event::StartTest(ref name) => {
                self.name_stack.push(name.clone());
                Ok(())
            }
            Event::EndTest(result) => {
                if !self.name_stack.is_empty() {
                    let failure_name = self.name_stack.join(" | ");
                    self.failures.push(failure_name);
                    self.name_stack.pop();
                }
                let chr = if result.is_ok() {
                    "."
                } else {
                    "F"
                };
                write!(self.buf, "{}", chr)
            }
            Event::FinishedRunner(result) => {
                let res = self.write_summary(result);
                writeln!(self.buf, "{}", res)
            }
            Event::EndDescribe => {
                self.name_stack.pop();
                Ok(())
            }
            // _ => Ok(()),
        };
    }
}
impl<'a, T: io::Write> Formatter for Simple<'a, T> {}

#[cfg(test)]
mod tests {
    pub use super::*;
    pub use formatter::formatter::Formatter;
    pub use events::{Event, EventHandler};
    pub use std::io;
    pub use std::str;

    #[test]
    fn it_can_be_instanciated() {
        Simple::new(&mut vec![1u8]);
    }

    #[test]
    fn it_impl_formatter_trait() {
        let _: &Formatter = &Simple::new(&mut vec![1u8]) as &Formatter;
    }

    mod event_start_runner {
        pub use super::*;

        #[test]
        fn it_display_that_tests_started() {
            let mut v = vec![];
            {
                let mut s = Simple::new(&mut v);
                s.trigger(&Event::StartRunner);
            }

            assert_eq!("Running tests:\n\n", str::from_utf8(&v).unwrap());
        }
    }

    mod event_finished_runner {
        pub use super::*;
        use runner::TestReport;

        fn make_report(succes: u32, errors: u32) -> Result<TestReport, TestReport> {
            let mut report = TestReport::default();
            report.success_count = succes;
            report.error_count = errors;
            report.total_tests = succes + errors;

            if errors != 0 {
                Err(report)
            } else {
                Ok(report)
            }
        }

        macro_rules! test_and_compare_output {
            ($(
                $test_name:ident : (success: $succ:expr, errors: $err:expr) => $msg:expr
            ),+) => {

                $(
                    #[test]
                    fn $test_name() {
                        let mut sink = io::sink();
                        let res = {
                            let mut s = Simple::new(&mut sink);
                            s.write_summary(make_report($succ, $err))
                        };

                        assert_eq!($msg, res)
                    }
                )+
            }
        }

        test_and_compare_output! {
            no_test_is_ok: (success: 0, errors: 0) =>
                "\n\ntest result: ok. 0 examples; 0 passed; 0 failed;",
            one_test: (success: 1, errors: 0) =>
                "\n\ntest result: ok. 1 examples; 1 passed; 0 failed;",
            multiple_ok: (success: 42, errors: 0) =>
                "\n\ntest result: ok. 42 examples; 42 passed; 0 failed;",
            one_error: (success: 0, errors: 1) =>
              "\n\ntest result: FAILED. 1 examples; 0 passed; 1 failed;",
            multiple_errors: (success: 0, errors: 37) =>
              "\n\ntest result: FAILED. 37 examples; 0 passed; 37 failed;",
            one_of_each: (success: 1, errors: 1) =>
              "\n\ntest result: FAILED. 2 examples; 1 passed; 1 failed;",
            multiple_of_each: (success: 12, errors: 21) =>
              "\n\ntest result: FAILED. 33 examples; 12 passed; 21 failed;"
        }
    }

    mod event_end_test {
        pub use super::*;

        #[test]
        fn it_displays_a_dot_when_success() {
            let mut v = vec![];
            {
                let mut s = Simple::new(&mut v);
                s.trigger(&Event::EndTest(Ok(())));
            }

            assert_eq!(".", str::from_utf8(&v).unwrap());
        }

        #[test]
        #[allow(non_snake_case)]
        fn it_displays_a_F_when_error() {
            let mut v = vec![];
            {
                let mut s = Simple::new(&mut v);
                s.trigger(&Event::EndTest(Err(())));
            }

            assert_eq!("F", str::from_utf8(&v).unwrap());
        }
    }

    mod event_start_end_describe {
        pub use super::*;

        #[test]
        fn start_describe_event_push_the_name_stack() {
            let mut sink = &mut io::sink();
            let mut s = Simple::new(&mut sink);

            s.trigger(&Event::StartDescribe(String::from("Hey !")));
            assert_eq!(vec![String::from("Hey !")], s.name_stack);

            s.trigger(&Event::StartDescribe(String::from("Ho !")));
            assert_eq!(vec![String::from("Hey !"), String::from("Ho !")],
                       s.name_stack)
        }

        #[test]
        fn end_describe_event_pop_the_name_stack() {
            let mut sink = &mut io::sink();
            let mut s = Simple::new(&mut sink);

            s.trigger(&Event::StartDescribe(String::from("Hey !")));
            s.trigger(&Event::StartDescribe(String::from("Ho !")));

            s.trigger(&Event::EndDescribe);
            assert_eq!(vec![String::from("Hey !")], s.name_stack);

            s.trigger(&Event::EndDescribe);
            assert_eq!(0, s.name_stack.len());
        }
    }

    mod failures_pretty_printing {
        use super::*;

        #[test]
        fn it_register_failures() {
            let mut sink = &mut io::sink();
            let mut s = Simple::new(&mut sink);
            s.trigger(&Event::StartTest("hola".into()));
            s.trigger(&Event::EndTest(Err(())));
            assert_eq!(1, s.failures.len());
        }

        #[test]
        fn it_keep_track_of_the_failure_name() {
            let mut sink = &mut io::sink();
            let mut s = Simple::new(&mut sink);
            s.trigger(&Event::StartTest("hola".into()));
            s.trigger(&Event::EndTest(Err(())));
            assert_eq!(Some(&"hola".into()), s.failures.get(0));
        }

        #[test]
        fn it_has_a_nice_diplay_for_describes() {
            let mut sink = &mut io::sink();
            let mut s = Simple::new(&mut sink);
            s.trigger(&Event::StartDescribe("hola".into()));
            s.trigger(&Event::StartTest("holé".into()));
            s.trigger(&Event::EndTest(Err(())));
            assert_eq!(Some(&"hola | holé".into()), s.failures.get(0));

            s.trigger(&Event::StartDescribe("ohééé".into()));
            s.trigger(&Event::StartTest("holé".into()));
            s.trigger(&Event::EndTest(Err(())));
            assert_eq!(Some(&"hola | ohééé | holé".into()), s.failures.get(1));
        }

        #[test]
        fn it_works_with_multiple_describes() {
            let mut sink = &mut io::sink();
            let mut s = Simple::new(&mut sink);
            s.trigger(&Event::StartDescribe("hola".into()));
            s.trigger(&Event::StartTest("holé".into()));
            s.trigger(&Event::EndTest(Err(())));

            s.trigger(&Event::EndDescribe);
            s.trigger(&Event::StartDescribe("ok".into()));
            s.trigger(&Event::StartTest("cacao".into()));
            s.trigger(&Event::EndTest(Err(())));
            assert_eq!(Some(&"ok | cacao".into()), s.failures.get(1));
        }

        #[test]
        fn format_all_failures_one_error() {
            let mut buf = vec![];
            let res = {
                let mut s = Simple::new(&mut buf);
                s.trigger(&Event::StartDescribe("hola".into()));
                s.trigger(&Event::StartTest("holé".into()));
                s.trigger(&Event::EndTest(Err(())));
                s.failures_summary()
            };

            assert_eq!("  1) hola | holé\n", res);
        }

        #[test]
        fn format_all_failures() {
            let mut buf = vec![];
            let res = {
                let mut s = Simple::new(&mut buf);
                s.trigger(&Event::StartDescribe("hola".into()));
                s.trigger(&Event::StartTest("holé".into()));
                s.trigger(&Event::EndTest(Err(())));
                s.trigger(&Event::StartTest("hola".into()));
                s.trigger(&Event::EndTest(Err(())));
                s.failures_summary()
            };

            assert_eq!("  1) hola | holé\n  2) hola | hola\n", res);

            let res = {
                let mut s = Simple::new(&mut buf);
                s.trigger(&Event::StartDescribe("hola".into()));
                s.trigger(&Event::StartTest("holé".into()));
                s.trigger(&Event::EndTest(Err(())));
                s.trigger(&Event::EndDescribe);
                s.trigger(&Event::StartDescribe("second".into()));
                s.trigger(&Event::StartDescribe("third".into()));
                s.trigger(&Event::StartTest("hola".into()));
                s.trigger(&Event::EndTest(Err(())));
                s.failures_summary()
            };

            assert_eq!("  1) hola | holé\n  2) second | third | hola\n", res);
        }
    }
}
