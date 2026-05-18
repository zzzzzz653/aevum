//! ECS ядро — Structure of Arrays (SoA) хранение компонентов

mod archetype;
mod entity;
mod world;
mod query;
mod component_mask;

pub use archetype::{Archetype, ArchetypeId};
pub use entity::EntityId;
pub use world::World;
pub use query::Query;
pub use component_mask::ComponentMask;
