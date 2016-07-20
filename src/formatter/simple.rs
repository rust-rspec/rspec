use events::{Event, EventHandler};
use formatter::formatter::Formatter;
use std::io;

pub struct Simple<Io: io::Write> {
    buf: Io
}

impl<T : io::Write> Simple<T> {
    fn new(buf: T) -> Simple<T> {
        Simple { buf: buf }
    }
}

impl<T : io::Write> EventHandler for Simple<T> {
    fn trigger(&mut self, _event: Event) {
    }
}
impl<T : io::Write> Formatter for Simple<T> {}

#[cfg(test)]
mod tests {
    pub use super::*;
    pub use formatter::formatter::Formatter;

    #[test]
    fn it_can_be_instanciated() {
        Simple::new(vec!(1u8));
    }

    #[test]
    fn it_impl_formatter_trait() {
        let _ : &Formatter = &Simple::new(vec!(1u8)) as &Formatter;
    }

    #[cfg(test)]
    mod event_finished_runner {
        pub use super::*;
        use events::{Event, EventHandler};

        #[test]
        fn it_display_a_recap() {
            let mut s = Simple::new(vec!(1u8));
            s.trigger(Event::StartRunner)
        }
    }
}
