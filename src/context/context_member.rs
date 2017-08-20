use context::Context;
use context::example::Example;

/// This enum is used to build a tree of named tests and contextes.
pub enum ContextMember<T> {
    Example(Example<T>),
    Context(Context<T>),
}

unsafe impl<T> Send for ContextMember<T>
where
    T: Send,
{
}
unsafe impl<T> Sync for ContextMember<T>
where
    T: Sync,
{
}
