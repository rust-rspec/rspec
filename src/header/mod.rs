pub mod suite;
pub mod context;
pub mod example;

pub trait Header {
    fn label(&self) -> &str;
    fn name(&self) -> &str;
}
