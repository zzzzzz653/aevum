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
        
        // Добавляем компонент AABB если его нет
        if !arch.has_component::<AABB>() {
            let mut new_aabbs: Vec<AABB> = Vec::new();
            
            // Шаг 1: Вычисляем новые AABB на основе позиций
            {
                let positions = arch.get_component_slice::<Position>();
                let rigid_bodies = arch.get_component_slice::<RigidBody>();
                for i in 0..arch.len {
                    let pos = [positions[i].x, positions[i].y, positions[i].z];
                    let rb = &rigid_bodies[i];
                    let aabb = compute_aabb(pos, &rb.shape);
                    new_aabbs.push(aabb);
                }
            } // Чтение завершено

            // Шаг 2: Применяем новые AABB
            for i in 0..arch.len {
                arch.add_component_by_index(i, new_aabbs[i]);
            }
        } else {
            let mut new_aabbs: Vec<AABB> = Vec::new();
            
            // Шаг 1: Вычисляем новые AABB на основе позиций
            {
                let positions = arch.get_component_slice::<Position>();
                let rigid_bodies = arch.get_component_slice::<RigidBody>();
                for i in 0..arch.len {
                    let pos = [positions[i].x, positions[i].y, positions[i].z];
                    let rb = &rigid_bodies[i];
                    new_aabbs.push(compute_aabb(pos, &rb.shape));
                }
            } // Чтение завершено

            // Шаг 2: Применяем новые AABB
            {
                let aabbs = arch.get_component_slice_mut::<AABB>();
                for i in 0..arch.len {
                    aabbs[i] = new_aabbs[i];
                }
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
