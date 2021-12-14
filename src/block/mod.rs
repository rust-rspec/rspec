//! Blocks are used to build a tree structure of named tests and contextes.

pub mod context;
pub mod example;
pub mod suite;

pub use crate::block::context::*;
pub use crate::block::example::*;
pub use crate::block::suite::*;

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
