pub mod trusted_generated;
mod elastic;
mod mongo;

pub use elastic::process_events_elastic;
pub use mongo::process_events_mongo;