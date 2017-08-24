// derive_builder emits warnings otherwise:
#![allow(unused_mut)]

/// A Runner's configuration.
#[derive(Builder, Default)]
pub struct Configuration {
    /// Whether the runner executes tests in parallel
    #[builder(default="false")]
    pub parallel: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let config = Configuration::default();
        assert_eq!(config.parallel, false);
    }

    #[test]
    fn builder() {
        let config = ConfigurationBuilder::default().build().unwrap();
        assert_eq!(config.parallel, false);

        let config = ConfigurationBuilder::default().parallel(true).build().unwrap();
        assert_eq!(config.parallel, true);
    }
}
