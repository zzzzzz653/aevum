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
        
        struct StateUpdate {
            index: usize,
            new_pos: [f32; 3],
            new_rot: [f32; 4],
        }
        let mut updates: Vec<StateUpdate> = Vec::new();

        // Шаг 1: Читаем данные и готовим новые значения
        {
            let positions = arch.get_component_slice::<Position>();
            let rotations = arch.get_component_slice::<Rotation>();
            let velocities = arch.get_component_slice::<Velocity>();
            let angular_velocities = arch.get_component_slice::<AngularVelocity>();
            let rigid_bodies = arch.get_component_slice::<RigidBody>();

            for i in 0..arch.len {
                let rb = &rigid_bodies[i];
                
                // Пропустить статические тела
                if rb.body_type == BodyType::Static {
                    continue;
                }
                
                // Обновить позицию: p = p + v * dt
                let vel = Vec3::new(velocities[i].x, velocities[i].y, velocities[i].z);
                let pos = &positions[i];
                let new_pos = [
                    pos.x + vel.x * dt,
                    pos.y + vel.y * dt,
                    pos.z + vel.z * dt,
                ];

                // Обновить вращение: q = q + 0.5 * w * q * dt
                let ang_vel = Vec3::new(angular_velocities[i].x, angular_velocities[i].y, angular_velocities[i].z);
                let rot = Quat::from_array([rotations[i].x, rotations[i].y, rotations[i].z, rotations[i].w]);
                
                let new_rot = if ang_vel.length_squared() > 0.0 {
                    let delta_quat = Quat::from_axis_angle(ang_vel.normalize(), ang_vel.length() * dt);
                    let normalized = (rot * delta_quat).normalize();
                    [normalized.x, normalized.y, normalized.z, normalized.w]
                } else {
                    [rot.x, rot.y, rot.z, rot.w]
                };

                updates.push(StateUpdate {
                    index: i,
                    new_pos,
                    new_rot,
                });
            }
        } // Освобождаем все ссылки на чтение

        // Шаг 2: Применяем изменения, получая мутабельный доступ
        {
            let positions = arch.get_component_slice_mut::<Position>();
            let rotations = arch.get_component_slice_mut::<Rotation>();

            for update in updates {
                positions[update.index].x = update.new_pos[0];
                positions[update.index].y = update.new_pos[1];
                positions[update.index].z = update.new_pos[2];
                rotations[update.index].x = update.new_rot[0];
                rotations[update.index].y = update.new_rot[1];
                rotations[update.index].z = update.new_rot[2];
                rotations[update.index].w = update.new_rot[3];
            }
        }
    }
}
