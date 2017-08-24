use block::example::Example;

pub struct Failure<'a, T>
where
    T: 'a
{
    example: &'a Example<T>,
    reason: String
}
