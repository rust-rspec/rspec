use context::Context;
use example::Example;

/// This enum is used to build a tree of named tests and contextes.
pub enum ContextMember<'a, T>
    where T: 'a
{
    Example(Example<'a, T>),
    Context(Context<'a, T>),
}

unsafe impl<'a, T> Send for ContextMember<'a, T> where T: 'a + Send {}
unsafe impl<'a, T> Sync for ContextMember<'a, T> where T: 'a + Sync {}
