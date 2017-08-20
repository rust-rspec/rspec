
pub struct Configuration {
    pub parallel: bool,
}

impl Configuration {
    pub fn parallel(self, parallel: bool) -> Self {
        Configuration { parallel: parallel }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration { parallel: true }
    }
}
