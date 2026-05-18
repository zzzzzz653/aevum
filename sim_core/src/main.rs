//! Точка входа — headless режим и визуализация

use sim_core::Simulation;

#[cfg(feature = "visualizer")]
mod visualizer_app {
    use sim_core::Simulation;
    use sim_core::render::{Renderer, RenderConfig};
    use winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
        window::{Window, WindowId},
    };
    use std::sync::Arc;

    pub struct App<'a> {
        sim: Simulation,
        renderer: Option<Renderer<'a>>,
        window: Option<Arc<Window>>,
    }

    impl<'a> App<'a> {
        pub fn new() -> Self {
            let mut sim = Simulation::new();
            sim.add_floor(0.0);
            sim.add_sphere_grid(100, 2.0, 0.0, 5.0, 1.0);
            println!("Total entities: {}", sim.entity_count());
            
            Self {
                sim,
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
                        self.sim.step(0.016);
                        renderer.render(self.sim.get_world());
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
        println!("Running native sim_core benchmark...");
        let mut sim = Simulation::new();

        // Создадим немного объектов для теста
        sim.add_floor(0.0);
        sim.add_sphere_grid(10, 2.0, 0.0, 5.0, 1.0);
        
        let steps = 100;
        let start = std::time::Instant::now();
        for _ in 0..steps {
            sim.step(0.016); // примерно 60 FPS
        }
        let duration = start.elapsed();
        
        println!("Entities: {}", sim.entity_count());
        println!("Completed {} steps in {:.2?} (avg: {:.2?}/step)", steps, duration, duration / steps as u32);
    }
}
