use std::io;
use std::ops::DerefMut;
use std::sync::Mutex;

use time::Duration;

use colored::*;

use header::{ContextHeader, ExampleHeader, SuiteHeader};
use report::{BlockReport, ContextReport, ExampleReport, ExampleResult, Report, SuiteReport};
use runner::{Runner, RunnerObserver};

#[derive(new)]
struct SerialLoggerState<T: io::Write = io::Stdout> {
    buffer: T,
    #[new(value = "0")]
    level: usize,
}

/// Preferred logger for serial test suite execution
/// (see [`Configuration.parallel`](struct.Configuration.html#fields)).
pub struct SerialLogger<T: io::Write = io::Stdout> {
    state: Mutex<SerialLoggerState<T>>,
}

impl Default for SerialLogger<io::Stdout> {
    fn default() -> Self {
        SerialLogger::new(io::stdout())
    }
}

impl<T: io::Write> SerialLogger<T> {
    pub fn new(buffer: T) -> Self {
        let state = SerialLoggerState::new(buffer);
        SerialLogger {
            state: Mutex::new(state),
        }
    }

    fn padding(depth: usize) -> String {
        "  ".repeat(depth)
    }

    fn access_state<F>(&self, mut accessor: F)
    where
        F: FnMut(&mut SerialLoggerState<T>) -> io::Result<()>,
    {
        if let Ok(ref mut mutex_guard) = self.state.lock() {
            let result = accessor(mutex_guard.deref_mut());
            if let Err(error) = result {
                // TODO: better error handling
                eprintln!("\n{}: {:?}", "error".red().bold(), error);
            }
        } else {
            // TODO: better error handling
            eprintln!(
                "\n{}: failed to aquire lock on mutex.",
                "error".red().bold()
            );
        }
    }

    fn write_suite_failures(
        &self,
        buffer: &mut T,
        indent: usize,
        report: &SuiteReport,
    ) -> io::Result<()> {
        if report.is_failure() {
            let _ = writeln!(buffer, "\nfailures:\n");
            writeln!(buffer, "{}{}", Self::padding(indent), report.get_header())?;
            let context_report = report.get_context();
            for block_report in context_report.get_blocks() {
                self.write_block_failures(buffer, indent + 1, block_report)?;
            }
        }

        Ok(())
    }

    fn write_block_failures(
        &self,
        buffer: &mut T,
        indent: usize,
        report: &BlockReport,
    ) -> io::Result<()> {
        if report.is_failure() {
            match report {
                BlockReport::Context(ref header, ref report) => {
                    if let Some(header) = header.as_ref() {
                        write!(buffer, "{}{}", Self::padding(indent), header)?;
                    }
                    self.write_context_failures(buffer, indent + 1, report)?;
                }
                BlockReport::Example(ref header, ref report) => {
                    writeln!(buffer, "{}{}", Self::padding(indent), header)?;
                    self.write_example_failure(buffer, indent + 1, report)?;
                }
            }
        }
        Ok(())
    }

    fn write_context_failures(
        &self,
        buffer: &mut T,
        indent: usize,
        report: &ContextReport,
    ) -> io::Result<()> {
        if report.is_failure() {
            writeln!(buffer)?;
            for block_report in report.get_blocks() {
                self.write_block_failures(buffer, indent + 1, block_report)?;
            }
        }

        Ok(())
    }

    fn write_example_failure(
        &self,
        buffer: &mut T,
        indent: usize,
        report: &ExampleReport,
    ) -> io::Result<()> {
        if let ExampleResult::Failure(Some(ref reason)) = report.get_result() {
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
        self.write_duration(buffer, report.get_duration())?;

        write!(buffer, "\ntest result: {}.", self.report_flag(report))?;

        writeln!(
            buffer,
            " {} passed; {} failed; {} ignored",
            report.get_passed(),
            report.get_failed(),
            report.get_ignored()
        )?;

        if report.is_failure() {
            writeln!(buffer, "\n{}: test failed", "error".red().bold())?;
        }

        Ok(())
    }

    fn write_duration(&self, buffer: &mut T, duration: Duration) -> io::Result<()> {
        let millisecond = 1;
        let second = 1000 * millisecond;
        let minute = 60 * second;
        let hour = 60 * minute;

        let remainder = duration.whole_milliseconds();

        let hours = remainder / hour;
        let remainder = remainder % hour;

        let minutes = remainder / minute;
        let remainder = remainder % minute;

        let seconds = remainder / second;
        let remainder = remainder % second;

        let milliseconds = remainder / millisecond;
        match (hours, minutes, seconds, milliseconds) {
            (0, 0, s, ms) => writeln!(buffer, "\nduration: {}.{:03}s.", s, ms),
            (0, m, s, ms) => writeln!(buffer, "\nduration: {}m {}.{:03}s.", m, s, ms),
            (h, m, s, ms) => writeln!(buffer, "\nduration: {}h {}m {}.{:03}s.", h, m, s, ms),
        }
    }

    fn report_flag<R>(&self, report: &R) -> ColoredString
    where
        R: Report,
    {
        if report.is_success() {
            "ok".green()
        } else {
            "FAILED".red()
        }
    }
}

impl<T: io::Write> RunnerObserver for SerialLogger<T>
where
    T: Send + Sync,
{
    fn enter_suite(&self, _runner: &Runner, header: &SuiteHeader) {
        self.access_state(|state| {
            state.level += 1;
            self.write_suite_prefix(&mut state.buffer)?;
            writeln!(state.buffer, "{}{}", Self::padding(state.level - 1), header)?;

            Ok(())
        });
    }

    fn exit_suite(&self, _runner: &Runner, _header: &SuiteHeader, report: &SuiteReport) {
        self.access_state(|state| {
            self.write_suite_failures(&mut state.buffer, 0, report)?;
            self.write_suite_suffix(&mut state.buffer, report)?;

            state.level -= 1;

            Ok(())
        });
    }

    fn enter_context(&self, _runner: &Runner, header: &ContextHeader) {
        self.access_state(|state| {
            state.level += 1;
            writeln!(state.buffer, "{}{}", Self::padding(state.level - 1), header)?;

            Ok(())
        });
    }

    fn exit_context(&self, _runner: &Runner, _header: &ContextHeader, _report: &ContextReport) {
        self.access_state(|state| {
            state.level -= 1;

            Ok(())
        });
    }

    fn enter_example(&self, _runner: &Runner, header: &ExampleHeader) {
        self.access_state(|state| {
            state.level += 1;
            write!(
                state.buffer,
                "{}{} ... ",
                Self::padding(state.level - 1),
                header
            )?;

            Ok(())
        });
    }

    fn exit_example(&self, _runner: &Runner, _header: &ExampleHeader, report: &ExampleReport) {
        self.access_state(|state| {
            writeln!(state.buffer, "{}", self.report_flag(report))?;
            state.level -= 1;

            Ok(())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod padding {
        use super::*;

        #[test]
        fn it_padds() {
            // arrange
            let expected = vec![("", 0), ("  ", 1), ("    ", 2), ("      ", 3)];
            for (expected_res, given_depth) in expected {
                // act
                let res = SerialLogger::<Vec<u8>>::padding(given_depth);
                // assert
                assert_eq!(String::from(expected_res), res)
            }
        }
    }
}
