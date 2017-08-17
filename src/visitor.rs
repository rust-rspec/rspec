pub trait Visitor<T> {
    type Output;

    fn visit(&mut self, visitable: &T) -> Self::Output;
}
