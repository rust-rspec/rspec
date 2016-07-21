use events::{Event, EventHandler};
use formatter::formatter::Formatter;
use runner;
use std::io;

pub struct Simple<'a, Io: io::Write + 'a> {
    buf: &'a mut Io,
}

impl<'a, T: io::Write> Simple<'a, T> {
    fn new(buf: &mut T) -> Simple<T> {
        Simple { buf: buf }
    }

    fn write_summary(&mut self, result: runner::RunnerResult) -> Result<(), io::Error> {
        let (res, report) = match result {
            Ok(report) => ("ok", report),
            Err(report) => ("FAILED", report),
        };

        writeln!(self.buf,
                 "test result: {}. {} examples; {} passed; {} failed;",
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
            Event::StartRunner => writeln!(self.buf, "Running tests..."),
            Event::FinishedRunner(result) => self.write_summary(result),
            _ => Ok(()),
        };
    }
}
impl<'a, T: io::Write> Formatter for Simple<'a, T> {}

#[cfg(test)]
mod tests {
    pub use super::*;
    pub use formatter::formatter::Formatter;
    pub use events::{Event, EventHandler};
    pub use std::str;

    #[test]
    fn it_can_be_instanciated() {
        Simple::new(&mut vec![1u8]);
    }

    #[test]
    fn it_impl_formatter_trait() {
        let _: &Formatter = &Simple::new(&mut vec![1u8]) as &Formatter;
    }

    #[cfg(test)]
    mod event_start_runner {
        pub use super::*;

        #[test]
        fn it_display_that_tests_started() {
            let mut v = vec![];
            {
                let mut s = Simple::new(&mut v);
                s.trigger(&Event::StartRunner);
            }

            assert_eq!("Running tests...\n", str::from_utf8(&v).unwrap());
        }
    }

    #[cfg(test)]
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
                        let mut v = vec!();
                        {
                            let mut s = Simple::new(&mut v);
                            s.trigger(&Event::FinishedRunner(make_report($succ, $err)))
                        }

                        assert_eq!($msg, str::from_utf8(&v).unwrap())
                    }
                )+
            }
        }

        test_and_compare_output! {
            no_test_is_ok: (success: 0, errors: 0) =>
                "test result: ok. 0 examples; 0 passed; 0 failed;\n",
            one_test: (success: 1, errors: 0) =>
                "test result: ok. 1 examples; 1 passed; 0 failed;\n",
            multiple_ok: (success: 42, errors: 0) =>
                "test result: ok. 42 examples; 42 passed; 0 failed;\n",
            one_error: (success: 0, errors: 1) =>
              "test result: FAILED. 1 examples; 0 passed; 1 failed;\n",
            multiple_errors: (success: 0, errors: 37) =>
              "test result: FAILED. 37 examples; 0 passed; 37 failed;\n",
            one_of_each: (success: 1, errors: 1) =>
              "test result: FAILED. 2 examples; 1 passed; 1 failed;\n",
            multiple_of_each: (success: 12, errors: 21) =>
              "test result: FAILED. 33 examples; 12 passed; 21 failed;\n"
        }
    }
}
