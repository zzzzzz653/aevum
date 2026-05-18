//! Рендерер — визуализация симуляции через wgpu + egui

use crate::world::World;

/// Конфигурация рендерера
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            vsync: true,
        }
    }
}

/// Главный рендерер
pub struct Renderer<'a> {
    config: RenderConfig,
    instance: Option<wgpu::Instance>,
    surface: Option<wgpu::Surface<'a>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
}

impl<'a> Renderer<'a> {
    pub fn new(config: RenderConfig) -> Self {
        Self {
            config,
            instance: None,
            surface: None,
            device: None,
            queue: None,
            surface_config: None,
        }
    }

    /// Инициализировать wgpu
    pub async fn initialize(&'a mut self, window: &'a winit::window::Window) -> Result<(), Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)
            .map_err(|e| format!("Failed to create surface: {}", e))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to get adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Simulation Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: self.config.width,
            height: self.config.height,
            present_mode: if self.config.vsync {
                wgpu::PresentMode::Fifo
            } else {
                wgpu::PresentMode::Immediate
            },
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        self.instance = Some(instance);
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface_config = Some(config);

        Ok(())
    }

    /// Рендерить кадр
    pub fn render(&mut self, _world: &World) {
        let Some(device) = &self.device else { return };
        let Some(queue) = &self.queue else { return };
        let Some(surface) = &self.surface else { return };
        let Some(_config) = &self.surface_config else { return };

        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => return,
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.15,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // TODO: добавить отрисовку сфер здесь
            // Пока просто очищаем экран
        }

        queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }

    /// Получить устройства
    pub fn device(&self) -> Option<&wgpu::Device> {
        self.device.as_ref()
    }

    pub fn queue(&self) -> Option<&wgpu::Queue> {
        self.queue.as_ref()
    }
}

/// Отрисовать отладочную информацию
pub fn draw_debug(world: &World) {
    println!("Entities: {}", world.entity_count());
    for arch in world.archetypes() {
        println!("  Archetype {:?}: {} entities", arch.id, arch.len);
    }
}

