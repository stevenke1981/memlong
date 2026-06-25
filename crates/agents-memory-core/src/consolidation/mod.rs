pub mod decay;
pub mod dedup;
pub mod engine;
pub mod entity;

pub use decay::{calculate_retention, initial_stability, reinforce_stability};
pub use engine::ConsolidationEngine;
