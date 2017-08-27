// derive_builder emits warnings otherwise:
#![allow(unused_mut)]

use std::default::Default;

/// A Runner's configuration.
#[derive(Builder)]
pub struct Configuration {
    /// Whether the runner executes tests in parallel
    #[builder(default = "true")]
    pub parallel: bool,
    /// Whether the runner exits the procees upon encountering failures
    #[builder(default = "true")]
    pub exit_on_failure: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        ConfigurationBuilder::default().build().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_with_builder() {
        let config = ConfigurationBuilder::default().build().unwrap();
        assert_eq!(config.parallel, true);
        assert_eq!(config.exit_on_failure, true);
    }

    #[test]
    fn default() {
        // arrange
        let expected = ConfigurationBuilder::default().build().unwrap();
        // act
        let config = Configuration::default();
        // assert
        assert_eq!(expected.parallel, config.parallel);
        assert_eq!(expected.exit_on_failure, config.exit_on_failure);
    }

    #[test]
    fn builder() {
        let config = ConfigurationBuilder::default().build().unwrap();
        assert_eq!(config.parallel, true);
        assert_eq!(config.exit_on_failure, true);

        let config = ConfigurationBuilder::default()
            .parallel(false)
            .build()
            .unwrap();
        assert_eq!(config.parallel, false);
        assert_eq!(config.exit_on_failure, true);

        let config = ConfigurationBuilder::default()
            .exit_on_failure(false)
            .build()
            .unwrap();
        assert_eq!(config.parallel, true);
        assert_eq!(config.exit_on_failure, false);
    }
}
