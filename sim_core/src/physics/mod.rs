//! Физический движок

pub mod broadphase;
pub mod narrowphase;
pub mod constraints;
pub mod muscles;
pub mod integrator;
pub mod materials;
pub mod stability;
pub mod creature_builder;

pub use broadphase::{SpatialHashGrid, find_collision_pairs};
