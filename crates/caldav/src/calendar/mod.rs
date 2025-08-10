pub mod methods;
pub mod prop;
pub mod resource;
mod service;

pub use service::CalendarResourceService;

#[cfg(test)]
pub mod tests;
