//! Система интеграции - обновляет позиции и скорости

use crate::world::World;
use crate::components::transform::{Position, Rotation, Velocity, AngularVelocity};
use crate::components::physics::{RigidBody, BodyType};
use glam::{Vec3, Quat};

/// Интегрировать движение всех тел (semi-implicit Euler)
pub fn integrate_motion(world: &mut World, dt: f32) {
    for arch in world.archetypes_mut() {
        // Проверить наличие нужных компонентов
        if !arch.has_component::<Position>() 
            || !arch.has_component::<Rotation>()
            || !arch.has_component::<Velocity>()
            || !arch.has_component::<AngularVelocity>()
            || !arch.has_component::<RigidBody>() {
            continue;
        }
        
        let positions = arch.get_component_slice_mut::<Position>();
        let rotations = arch.get_component_slice_mut::<Rotation>();
        let velocities = arch.get_component_slice::<Velocity>();
        let angular_velocities = arch.get_component_slice::<AngularVelocity>();
        let rigid_bodies = arch.get_component_slice::<RigidBody>();
        
        for i in 0..arch.len() {
            let rb = &rigid_bodies[i];
            
            // Пропустить статические тела
            if rb.body_type == BodyType::Static {
                continue;
            }
            
            // Обновить позицию: p = p + v * dt
            let vel = Vec3::new(velocities[i].x, velocities[i].y, velocities[i].z);
            positions[i].x += vel.x * dt;
            positions[i].y += vel.y * dt;
            positions[i].z += vel.z * dt;
            
            // Обновить вращение: q = q + 0.5 * w * q * dt
            let ang_vel = Vec3::new(angular_velocities[i].x, angular_velocities[i].y, angular_velocities[i].z);
            let rot = Quat::from_array([rotations[i].x, rotations[i].y, rotations[i].z, rotations[i].w]);
            
            if ang_vel.length_squared() > 0.0 {
                let delta_rot = Quat::from_vec4(Vec3::new(
                    ang_vel.x * dt * 0.5,
                    ang_vel.y * dt * 0.5,
                    ang_vel.z * dt * 0.5,
                    0.0,
                )) * rot;
                let new_rot = (rot + delta_rot).normalize();
                rotations[i].x = new_rot.x;
                rotations[i].y = new_rot.y;
                rotations[i].z = new_rot.z;
                rotations[i].w = new_rot.w;
            }
        }
    }
}
