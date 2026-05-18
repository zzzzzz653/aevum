//! Система обновления AABB для коллизий

use crate::world::World;
use crate::components::transform::Position;
use crate::components::physics::{RigidBody, CollisionShape};
use crate::components::collision::AABB;

/// Обновить AABB всех тел на основе их позиции и формы
pub fn update_aabb(world: &mut World) {
    for arch in world.archetypes_mut() {
        if !arch.has_component::<Position>() 
            || !arch.has_component::<RigidBody>() {
            continue;
        }
        
        let positions = arch.get_component_slice::<Position>();
        let rigid_bodies = arch.get_component_slice::<RigidBody>();
        
        // Добавляем компонент AABB если его нет
        if !arch.has_component::<AABB>() {
            for i in 0..arch.len() {
                let pos = [positions[i].x, positions[i].y, positions[i].z];
                let rb = &rigid_bodies[i];
                let aabb = compute_aabb(pos, &rb.shape);
                arch.add_component_by_index(i, aabb);
            }
        } else {
            let aabbs = arch.get_component_slice_mut::<AABB>();
            for i in 0..arch.len() {
                let pos = [positions[i].x, positions[i].y, positions[i].z];
                let rb = &rigid_bodies[i];
                aabbs[i] = compute_aabb(pos, &rb.shape);
            }
        }
    }
}

/// Вычислить AABB для данной позиции и формы
fn compute_aabb(position: [f32; 3], shape: &CollisionShape) -> AABB {
    match shape.tag {
        CollisionShape::SPHERE => {
            let r = shape.radius();
            AABB::from_center_half_extents(
                position,
                [r, r, r],
            )
        }
        CollisionShape::BOX => {
            let half_extents = shape.half_extents();
            AABB::from_center_half_extents(position, half_extents)
        }
        CollisionShape::CAPSULE => {
            // Capsule: radius + height
            let r = shape.data[0];
            let h = shape.data[1];
            AABB::from_center_half_extents(
                position,
                [r, h + r, r],
            )
        }
        _ => {
            // Default fallback
            AABB::from_center_half_extents(position, [1.0, 1.0, 1.0])
        }
    }
}
