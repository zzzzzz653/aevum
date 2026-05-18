//! Точка входа — headless режим и визуализация

use sim_core::world::World;
use sim_core::scene::SceneSpawner;
use sim_core::systems::{apply_gravity, integrate_motion, clear_forces, update_aabb};
use sim_core::physics::{find_collision_pairs};
use glam::Vec3;

#[cfg(feature = "visualizer")]
mod visualizer_app {
    use crate::world::World;
    use crate::scene::SceneSpawner;
    use crate::render::{Renderer, RenderConfig};
    use winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
        window::{Window, WindowId},
    };
    use std::sync::Arc;

    pub struct App<'a> {
        world: World,
        renderer: Option<Renderer<'a>>,
        window: Option<Arc<Window>>,
    }

    impl<'a> App<'a> {
        pub fn new() -> Self {
            let mut world = World::new();
            let mut spawner = SceneSpawner::new(&mut world);
            spawner.spawn_ground(0.0);
            spawner.spawn_sphere_grid(100, 0.5, 0.2, 1.0);
            println!("Total entities: {}", world.entity_count());
            
            Self {
                world,
                renderer: None,
                window: None,
            }
        }
    }

    impl<'a> ApplicationHandler for App<'a> {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.renderer.is_some() {
                return;
            }

            let window_attributes = Window::default_attributes()
                .with_title("AEVUM Simulation")
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            
            let mut renderer = Renderer::new(RenderConfig::default());
            
            let window_clone = window.clone();
            pollster::block_on(renderer.initialize(&window_clone)).unwrap();
            
            self.renderer = Some(renderer);
            self.window = Some(window);
        }

        fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    if let Some(renderer) = &mut self.renderer {
                        self.world.step(0.016);
                        renderer.render(&self.world);
                        self.window.as_ref().unwrap().request_redraw();
                    }
                }
                _ => {}
            }
        }
    }

    pub fn run_visualizer() {
        println!("AEVUM Simulation with Visualization");
        println!("====================================");
        
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        
        let mut app = App::new();
        event_loop.run_app(&mut app).unwrap();
        
        println!("Simulation closed");
    }
}

fn main() {
    #[cfg(feature = "visualizer")]
    {
        visualizer_app::run_visualizer();
    }
    
    #[cfg(not(feature = "visualizer"))]
    {
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
}
