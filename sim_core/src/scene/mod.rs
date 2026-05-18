//! Scene модуль — спавн объектов и управление сценой

use crate::world::World;
use crate::components::transform::{Position, Rotation, Velocity, AngularVelocity, Force};
use crate::components::physics::{RigidBody, BodyType, CollisionShape};
use crate::components::collision::AABB;
use glam::Vec3;

/// Спавнер для создания объектов на сцене
pub struct SceneSpawner<'a> {
    world: &'a mut World,
}

impl<'a> SceneSpawner<'a> {
    pub fn new(world: &'a mut World) -> Self {
        Self { world }
    }

    /// Спавн сферы с физикой
    pub fn spawn_sphere(&mut self, position: Vec3, radius: f32, mass: f32) -> crate::EntityId {
        let entity = self.world.spawn();
        
        // Найти архетип с нужными компонентами или создать новый
        self.add_physics_components(entity, position, radius, mass);
        
        entity
    }

    fn add_physics_components(&mut self, entity: crate::EntityId, position: Vec3, radius: f32, mass: f32) {
        for arch in self.world.archetypes_mut() {
            if arch.entities.contains(&entity) {
                arch.add_component(entity, Position::from(position));
                arch.add_component(entity, Rotation::identity());
                arch.add_component(entity, Velocity::default());
                arch.add_component(entity, AngularVelocity::default());
                arch.add_component(entity, Force::zero());
                arch.add_component(entity, RigidBody::new(
                    BodyType::Dynamic,
                    CollisionShape::sphere(radius),
                    mass,
                ));
                arch.add_component(entity, AABB::from_center_half_extents(
                    [position.x, position.y, position.z],
                    [radius, radius, radius],
                ));
                return;
            }
        }
    }

    /// Спавн множества сфер для теста производительности
    pub fn spawn_sphere_grid(&mut self, count: usize, spacing: f32, start_x: f32, start_y: f32) {
        let radius = start_x;  // используем start_x как радиус
        let mass = start_y;    // используем start_y как массу
        let mut spawned = 0;
        let size = (count as f32).cbrt() as usize + 1;
        
        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    if spawned >= count {
                        return;
                    }
                    
                    let pos = Vec3::new(
                        x as f32 * spacing,
                        y as f32 * spacing + 10.0,
                        z as f32 * spacing,
                    );
                    
                    self.spawn_sphere(pos, radius, mass);
                    spawned += 1;
                }
            }
        }
    }

    /// Спавн пола (статичная плоскость)
    pub fn spawn_ground(&mut self, y_height: f32) -> crate::EntityId {
        let entity = self.world.spawn();
        
        for arch in self.world.archetypes_mut() {
            if arch.entities.contains(&entity) {
                arch.add_component(entity, Position::from(Vec3::new(0.0, y_height, 0.0)));
                arch.add_component(entity, Rotation::identity());
                // Пол не нуждается в скорости/силах
                arch.add_component(entity, RigidBody::new(
                    BodyType::Static,
                    CollisionShape::box_shape([100.0, 0.1, 100.0]),
                    0.0,
                ));
                arch.add_component(entity, AABB::from_center_half_extents(
                    [0.0, y_height, 0.0],
                    [100.0, 0.1, 100.0],
                ));
                return entity;
            }
        }
        
        entity
    }
}

