//! Python мост через PyO3

use pyo3::prelude::*;
use crate::world::World;
use crate::scene::SceneSpawner;
use crate::systems::{apply_gravity, integrate_motion, clear_forces, update_aabb};
use crate::physics::find_collision_pairs;
use glam::Vec3;

/// Симуляция мира, доступная из Python
#[pyclass]
pub struct Simulation {
    world: World,
    gravity: Vec3,
    dt: f32,
}

#[pymethods]
impl Simulation {
    #[new]
    fn new() -> Self {
        Self {
            world: World::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            dt: 0.01,
        }
    }
    
    /// Спавн пола
    fn spawn_ground(&mut self, y: f32) {
        let mut spawner = SceneSpawner::new(&mut self.world);
        spawner.spawn_ground(y);
    }
    
    /// Спавн сфер в сетке
    fn spawn_sphere_grid(&mut self, count: u32, spacing: f32, start_x: f32, start_y: f32) {
        let mut spawner = SceneSpawner::new(&mut self.world);
        spawner.spawn_sphere_grid(count, spacing, start_x, start_y);
    }
    
    /// Получить количество сущностей
    fn entity_count(&self) -> usize {
        self.world.entity_count()
    }
    
    /// Шаг симуляции
    fn step(&mut self) {
        clear_forces(&mut self.world);
        apply_gravity(&mut self.world, self.gravity);
        integrate_motion(&mut self.world, self.dt);
        update_aabb(&mut self.world);
        let _pairs = find_collision_pairs(&self.world);
        self.world.step(self.dt);
    }
    
    /// Запустить несколько шагов
    fn run(&mut self, steps: u32) {
        for _ in 0..steps {
            self.step();
        }
    }
    
    /// Установить гравитацию
    fn set_gravity(&mut self, x: f32, y: f32, z: f32) {
        self.gravity = Vec3::new(x, y, z);
    }
    
    /// Установить шаг времени
    fn set_dt(&mut self, dt: f32) {
        self.dt = dt;
    }
}

/// Инициализировать Python модуль
#[pymodule]
fn sim_bridge(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Simulation>()?;
    Ok(())
}
