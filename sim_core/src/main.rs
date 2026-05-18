//! Точка входа — headless режим и визуализация

use sim_core::world::World;
use sim_core::scene::SceneSpawner;
use sim_core::systems::{apply_gravity, integrate_motion, clear_forces, update_aabb};
use sim_core::physics::{find_collision_pairs};
use glam::Vec3;

fn main() {
    println!("AEVUM Simulation Core");
    println!("=====================");
    
    // Создать мир
    let mut world = World::new();
    
    // Создать спавнер
    let mut spawner = SceneSpawner::new(&mut world);
    
    // Спавн пола
    spawner.spawn_ground(0.0);
    
    // Спавн тестовых сфер
    println!("Spawning 1000 spheres...");
    spawner.spawn_sphere_grid(1000, 2.5, 1.0, 1.0);
    
    println!("Total entities: {}", world.entity_count());
    
    // Главный цикл симуляции (headless тест)
    let gravity = Vec3::new(0.0, -9.81, 0.0);
    let dt = 0.01;
    
    println!("\nRunning simulation for 100 ticks...");
    for tick in 0..100 {
        // 1. Очистить силы
        clear_forces(&mut world);
        
        // 2. Применить гравитацию
        apply_gravity(&mut world, gravity);
        
        // 3. Интегрировать движение
        integrate_motion(&mut world, dt);
        
        // 4. Обновить AABB
        update_aabb(&mut world);
        
        // 5. Найти пары коллизий (для отладки)
        let pairs = find_collision_pairs(&world);
        
        if tick % 10 == 0 {
            println!("Tick {}: {} entities, {} collision pairs", 
                     tick, world.entity_count(), pairs.len());
        }
        
        world.step(dt);
    }
    
    println!("\nSimulation complete!");
    println!("Final entity count: {}", world.entity_count());
}
