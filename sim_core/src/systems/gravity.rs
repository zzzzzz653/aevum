//! Система гравитации
//! Применяет силу гравитации ко всем динамическим телам

use crate::world::{World, Query};
use crate::components::physics::{RigidBody, BodyType};
use crate::components::transform::{Force, Velocity};
use glam::Vec3;

/// Применить гравитацию ко всем динамическим телам
pub fn apply_gravity(world: &mut World, gravity: Vec3) {
    let mut query = Query::default().with::<RigidBody>().with::<Velocity>().with::<Force>();
    
    for arch in world.archetypes() {
        // Проверить что архетип содержит нужные компоненты
        if !query.matches_archetype(arch) {
            continue;
        }
        
        let rigid_bodies = arch.get_component_slice::<RigidBody>();
        let forces = arch.get_component_slice_mut::<Force>();
        
        for i in 0..arch.len() {
            let rb = &rigid_bodies[i];
            
            // Применять только к динамическим телам
            if rb.body_type == BodyType::Dynamic && rb.mass > 0.0 {
                let force = &mut forces[i];
                force.x += gravity.x * rb.mass;
                force.y += gravity.y * rb.mass;
                force.z += gravity.z * rb.mass;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;
    use crate::components::transform::{Position, Rotation, AngularVelocity};
    use crate::components::physics::CollisionShape;
    
    #[test]
    fn test_gravity_applied() {
        let mut world = World::new();
        
        // Создать тело с необходимыми компонентами
        let entity = world.spawn();
        {
            let arch = world.archetypes_mut().next().unwrap();
            arch.add_component(entity, Position::from(Vec3::ZERO));
            arch.add_component(entity, Rotation::identity());
            arch.add_component(entity, Velocity::default());
            arch.add_component(entity, AngularVelocity::default());
            arch.add_component(entity, Force::zero());
            arch.add_component(entity, RigidBody::new(
                BodyType::Dynamic,
                CollisionShape::sphere(1.0),
                10.0,
            ));
        }
        
        apply_gravity(&mut world, Vec3::new(0.0, -9.81, 0.0));
        
        // Проверить что сила применена
        let arch = world.archetypes().next().unwrap();
        let forces = arch.get_component_slice::<Force>();
        assert!((forces[0].y + 98.1).abs() < 0.01, "Gravity force should be applied");
    }
}
