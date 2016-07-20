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
    fn trigger(&mut self, _event: Event) {
        // ignore errors
        let _ = self.buf.write(b"Running tests...\n");
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
    mod event_finished_runner {
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
}
