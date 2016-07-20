use events::{Event, EventHandler};
use formatter::formatter::Formatter;
use std::io;

pub struct Simple<'a, Io: io::Write + 'a> {
    buf: &'a mut Io
}

impl<'a, T : io::Write> Simple<'a, T> {
    fn new<'b>(buf: &'b mut T) -> Simple<'b, T> {
        Simple { buf: buf }
    }
}

impl<'a, T : io::Write> EventHandler for Simple<'a, T> {
    fn trigger(&mut self, event: Event) {
        let _ = match event {
            Event::StartRunner => writeln!(self.buf, "Running tests..."),
            Event::FinishedRunner(result) => writeln!(
                self.buf,
                "test result: ok. {0} examples; {0} passed; 0 failed;",
                result.unwrap().total_tests
            )
        };
    }
}
impl<'a, T : io::Write> Formatter for Simple<'a, T> {}

#[cfg(test)]
mod tests {
    pub use super::*;
    pub use formatter::formatter::Formatter;

    #[test]
    fn it_can_be_instanciated() {
        Simple::new(&mut vec!(1u8));
    }

    #[test]
    fn it_impl_formatter_trait() {
        let _ : &Formatter = &Simple::new(&mut vec!(1u8)) as &Formatter;
    }

    #[cfg(test)]
    mod event_start_runner {
        pub use super::*;
        use events::{Event, EventHandler};
        use std::str;

        #[test]
        fn it_display_that_tests_started() {
            let mut v = vec!();
            {
                let mut s = Simple::new(&mut v);
                s.trigger(Event::StartRunner);
            }

            assert_eq!("Running tests...\n", str::from_utf8(&v).unwrap());
        }
    }

    #[cfg(test)]
    mod event_finished_runner {
        pub use super::*;
        use events::{Event, EventHandler};
        use runner::TestReport;
        use std::str;

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

        #[test]
        fn one_test() {
            let mut v = vec!();
            {
                let mut s = Simple::new(&mut v);
                s.trigger(Event::FinishedRunner(make_report(1, 0)))
            }

            assert_eq!("test result: ok. 1 examples; 1 passed; 0 failed;\n",
                       str::from_utf8(&v).unwrap())
        }

        #[test]
        fn multiple_ok() {
            let mut v = vec!();
            {
                let mut s = Simple::new(&mut v);
                s.trigger(Event::FinishedRunner(make_report(42, 0)))
            }

            assert_eq!("test result: ok. 42 examples; 42 passed; 0 failed;\n",
                       str::from_utf8(&v).unwrap())
        }
    }
}
