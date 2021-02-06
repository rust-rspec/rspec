pub(crate) trait TestSuiteVisitor<T> {
    type Environment;
    type Output;

    fn visit(&self, visitable: &T, environment: &mut Self::Environment) -> Self::Output;
}
