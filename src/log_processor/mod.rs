pub mod log_filter;
pub mod info_log_processor;
pub mod pass_log_processor;

pub trait LogProcessor {
    type Out;
    fn parse<C: AsRef<str>>(&self, content: C) -> Self::Out;
}