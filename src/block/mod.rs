//! Blocks are used to build a tree structure of named tests and contextes.

mod context;
mod example;
mod suite;

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

unsafe impl<T> Send for Block<T> where T: Send {}
unsafe impl<T> Sync for Block<T> where T: Sync {}
