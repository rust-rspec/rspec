// derive_builder emits warnings otherwise:
#![allow(unused_mut)]

/// A Runner's configuration.
#[derive(Builder)]
pub struct Configuration {
    /// Whether the runner executes tests in parallel
    #[builder(default="true")]
    pub parallel: bool,
    /// Whether the runner exits the procees upon encountering failures
    #[builder(default="true")]
    pub exit_on_failure: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let config = ConfigurationBuilder::default().build().unwrap();
        assert_eq!(config.parallel, true);
        assert_eq!(config.exit_on_failure, true);
    }

    #[test]
    fn builder() {
        let config = ConfigurationBuilder::default().build().unwrap();
        assert_eq!(config.parallel, true);
        assert_eq!(config.exit_on_failure, true);

        let config = ConfigurationBuilder::default().parallel(false).build().unwrap();
        assert_eq!(config.parallel, false);
        assert_eq!(config.exit_on_failure, true);

        let config = ConfigurationBuilder::default().exit_on_failure(false).build().unwrap();
        assert_eq!(config.parallel, true);
        assert_eq!(config.exit_on_failure, false);
    }
}
