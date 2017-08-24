pub mod suite;
pub mod context;
pub mod example;

use block::context::Context;
use block::example::Example;

/// This enum is used to build a tree of named tests and contextes.
pub enum Block<T> {
    Context(Context<T>),
    Example(Example<T>),
}

impl<T> Block<T> {
    pub fn num_examples(&self) -> usize {
        match self {
            &Block::Context(ref context) => context.num_examples(),
            &Block::Example(_) => 1,
        }
    }
}

unsafe impl<T> Send for Block<T>
where
    T: Send,
{
}
unsafe impl<T> Sync for Block<T>
where
    T: Sync,
{
}
