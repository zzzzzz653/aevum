//! Интегратор физики — Semi-implicit Euler с substeps

use crate::components::transform::{Position, Rotation, Velocity, AngularVelocity};
use crate::components::physics::RigidBody;
use glam::{Vec3, Quat};

/// Конфигурация интегратора
#[derive(Debug, Clone)]
pub struct IntegratorConfig {
    pub dt_physics: f32,        // 0.01 (100 Гц)
    pub substeps: u32,          // 4-8 подшагов на тик
    pub solver_iterations: u32, // 5-10 XPBD итераций
    pub gravity: [f32; 3],      // (0, -9.81, 0)
    pub linear_damping: f32,    // 0.999
    pub angular_damping: f32,   // 0.995
    pub sleep_threshold: f32,   // 0.01
    pub sleep_frames: u32,      // 60
}

impl Default for IntegratorConfig {
    fn default() -> Self {
        Self {
            dt_physics: 0.01,
            substeps: 4,
            solver_iterations: 8,
            gravity: [0.0, -9.81, 0.0],
            linear_damping: 0.999,
            angular_damping: 0.995,
            sleep_threshold: 0.01,
            sleep_frames: 60,
        }
    }
}

/// Полу-неявный Эйлер интегрирование
pub struct Integrator {
    config: IntegratorConfig,
}

impl Integrator {
    pub fn new(config: IntegratorConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &IntegratorConfig {
        &self.config
    }

    /// Интегрировать одно тело на один substep
    pub fn integrate_body(
        &self,
        pos: &mut Position,
        rot: &mut Rotation,
        vel: &mut Velocity,
        ang_vel: &mut AngularVelocity,
        body: &RigidBody,
        force: &[f32; 3],
        torque: &[f32; 3],
        dt: f32,
    ) {
        if body.is_static() || body.sleeping {
            return;
        }

        let mut v = Vec3::from(*vel);
        let mut w = Vec3::from(*ang_vel);
        let mut p = Vec3::from(*pos);
        let mut q = Quat::from(*rot);

        let f = Vec3::from_array(*force);
        let t = Vec3::from_array(*torque);

        // Semi-implicit Euler
        // 1. Обновить скорость: v += (F / m + g) * dt
        let accel = f * body.inv_mass + Vec3::from(self.config.gravity);
        v += accel * dt;
        v *= self.config.linear_damping;

        // 2. Обновить угловую скорость: w += I^-1 * (t - w × Iw) * dt
        let inv_inertia_diag = Vec3::new(body.inv_inertia[0], body.inv_inertia[4], body.inv_inertia[8]);
        
        // Упрощённая модель (диагональная инерция)
        let angular_accel = t * inv_inertia_diag;
        w += angular_accel * dt;
        w *= self.config.angular_damping;

        // 3. Обновить позицию: p += v * dt
        p += v * dt;

        // 4. Обновить вращение: q += 0.5 * w * q * dt
        let dq = Quat::from_vec4(w.extend(0.0)) * q * 0.5 * dt;
        q = q + dq;
        q = q.normalize();

        *vel = Velocity::from(v);
        *ang_vel = AngularVelocity::from(w);
        *pos = Position::from(p);
        *rot = Rotation::from(q);
    }

    /// Полный шаг интеграции с substeps
    pub fn step(&self, bodies: &mut [PhysicsBodyView], dt: f32) {
        let substep_dt = dt / self.config.substeps as f32;
        
        for _ in 0..self.config.substeps {
            for body_view in bodies.iter_mut() {
                self.integrate_body(
                    &mut body_view.position,
                    &mut body_view.rotation,
                    &mut body_view.velocity,
                    &mut body_view.angular_velocity,
                    &body_view.rigid_body,
                    &body_view.force,
                    &body_view.torque,
                    substep_dt,
                );
            }
        }
    }
}

/// View на физические данные тела для интеграции
pub struct PhysicsBodyView<'a> {
    pub position: &'a mut Position,
    pub rotation: &'a mut Rotation,
    pub velocity: &'a mut Velocity,
    pub angular_velocity: &'a mut AngularVelocity,
    pub rigid_body: &'a RigidBody,
    pub force: &'a mut [f32; 3],
    pub torque: &'a mut [f32; 3],
}
