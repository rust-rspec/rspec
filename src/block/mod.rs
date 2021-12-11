//! Blocks are used to build a tree structure of named tests and contextes.

pub mod context;
pub mod example;
pub mod suite;

pub use block::context::*;
pub use block::example::*;
pub use block::suite::*;

/// Blocks are used to build a tree structure of named tests and contextes.
pub enum Block<T> {
    Context(Context<T>),
    Example(Example<T>),
}

impl<T> Block<T> {
    pub fn num_examples(&self) -> usize {
        match self {
            Block::Context(ref context) => context.num_examples(),
            Block::Example(_) => 1,
        }
    }
}
