//! sim_core — ядро симуляции AEVUM
//! 
//! Архитектура: Data-Oriented Design, GPU-first, deterministic-ready

pub mod world;
pub mod components;
pub mod physics;
pub mod systems;
pub mod scene;
pub mod render;
pub mod bridge;

// Остальные модули будут добавлены позже
// pub mod scheduler;
// pub mod environment;
// pub mod sensors;
// pub mod events;
// pub mod record;
// pub mod profiler;
// pub mod validate;

// Ре-экспорт основных типов
pub use world::{World, EntityId, ComponentMask};
pub use components::transform::{Position, Rotation, Velocity, AngularVelocity, Force, Torque};
pub use components::physics::{RigidBody, BodyType, CollisionShape};
pub use physics::integrator::IntegratorConfig;
pub use systems::{apply_gravity, integrate_motion, clear_forces};
pub use bridge::Simulation;
