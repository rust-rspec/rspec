//! Headers store the label and name of a Suite/Context/Example.

mod suite;
mod context;
mod example;

pub use header::suite::*;
pub use header::context::*;
pub use header::example::*;

/// A header with label and name of a [`Suite`](../block/struct.Suite.html)/[`Context`](../block/struct.Context.html)/[`Example`](../block/struct.Example.html).
pub trait Header {
    fn label(&self) -> &str;
    fn name(&self) -> &str;
}
