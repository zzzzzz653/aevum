//! Рендерер — визуализация симуляции через wgpu + egui

use crate::world::World;
use crate::components::transform::Position;
use crate::components::physics::CollisionShape;
use glam::{Vec3, Mat4};
use bytemuck::{Pod, Zeroable};

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

/// Vertex для сферы (инстансинг)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
}

/// Instance данные для каждой сферы
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct InstanceData {
    pub model: [[f32; 4]; 4],
    pub color: [f32; 4],
}

/// Uniform буфер для камеры
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

/// Главный рендерер
pub struct Renderer<'a> {
    config: RenderConfig,
    instance: Option<wgpu::Instance>,
    surface: Option<wgpu::Surface<'a>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    
    // Ресурсы для отрисовки сфер
    sphere_vertex_buffer: Option<wgpu::Buffer>,
    sphere_index_buffer: Option<wgpu::Buffer>,
    instance_buffer: Option<wgpu::Buffer>,
    camera_uniform_buffer: Option<wgpu::Buffer>,
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group: Option<wgpu::BindGroup>,
    index_count: u32,
    instance_count: u32,
    
    // Камера
    camera_position: Vec3,
    camera_target: Vec3,
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
            sphere_vertex_buffer: None,
            sphere_index_buffer: None,
            instance_buffer: None,
            camera_uniform_buffer: None,
            pipeline: None,
            bind_group: None,
            index_count: 0,
            instance_count: 0,
            camera_position: Vec3::new(0.0, 10.0, 20.0),
            camera_target: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    /// Создать геометрию сферы (UV-сфера)
    fn create_sphere(radius: f32, sectors: u32, stacks: u32) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for stack in 0..=stacks {
            let phi = std::f32::consts::PI * (stack as f32) / (stacks as f32);
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            for sector in 0..=sectors {
                let theta = 2.0 * std::f32::consts::PI * (sector as f32) / (sectors as f32);
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();

                let x = cos_theta * sin_phi;
                let y = cos_phi;
                let z = sin_theta * sin_phi;

                let normal = [x, y, z];
                let pos = [x * radius, y * radius, z * radius];

                vertices.push(Vertex { pos, normal });
            }
        }

        for stack in 0..stacks {
            for sector in 0..sectors {
                let current = stack * (sectors + 1) + sector;
                let next = current + sectors + 1;

                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                indices.push(next);
                indices.push(next + 1);
                indices.push(current + 1);
            }
        }

        (vertices, indices)
    }

    /// Создать матрицу вида-проекции
    fn create_view_projection(&self, aspect: f32) -> Mat4 {
        let view = Mat4::look_at_rh(self.camera_position, self.camera_target, Vec3::Y);
        let projection = Mat4::perspective_rh(
            std::f32::consts::FRAC_PI_4,
            aspect,
            0.1,
            1000.0,
        );
        projection * view
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

        // Создать геометрию сферы
        let (vertices, indices) = Self::create_sphere(1.0, 32, 16);
        self.index_count = indices.len() as u32;

        // Vertex буфер
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Index буфер
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Instance буфер (будет обновляться каждый кадр)
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (100000 * std::mem::size_of::<InstanceData>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Camera uniform буфер
        let camera_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Layout для pipeline
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Shader модуль
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sphere Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("sphere.wgsl"))),
        });

        // Render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sphere Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x3,  // pos
                            1 => Float32x3,  // normal
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<InstanceData>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            2 => Float32x4,  // model row 0
                            3 => Float32x4,  // model row 1
                            4 => Float32x4,  // model row 2
                            5 => Float32x4,  // model row 3
                            6 => Float32x4,  // color
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                sample_checking: None,
            },
            multiview: None,
        });

        self.sphere_vertex_buffer = Some(vertex_buffer);
        self.sphere_index_buffer = Some(index_buffer);
        self.instance_buffer = Some(instance_buffer);
        self.camera_uniform_buffer = Some(camera_uniform_buffer);
        self.pipeline = Some(pipeline);
        self.bind_group = Some(bind_group);

        Ok(())
    }

    /// Рендерить кадр
    pub fn render(&mut self, world: &World) {
        let Some(device) = &self.device else { return };
        let Some(queue) = &self.queue else { return };
        let Some(surface) = &self.surface else { return };
        let Some(config) = &self.surface_config else { return };
        let Some(pipeline) = &self.pipeline else { return };
        let Some(bind_group) = &self.bind_group else { return };
        let Some(vertex_buffer) = &self.sphere_vertex_buffer else { return };
        let Some(index_buffer) = &self.sphere_index_buffer else { return };
        let Some(instance_buffer) = &self.instance_buffer else { return };
        let Some(camera_uniform_buffer) = &self.camera_uniform_buffer else { return };

        // Собрать данные сфер из мира
        let mut instances: Vec<InstanceData> = Vec::new();
        
        for arch in world.archetypes() {
            // Проверить, есть ли в архетипе Position и CollisionShape
            if let (Some(positions), Some(shapes)) = (
                arch.get_component_slice::<Position>(),
                arch.get_component_slice::<CollisionShape>(),
            ) {
                for i in 0..arch.len {
                    let pos = &positions[i];
                    let shape = &shapes[i];
                    
                    // Создать матрицу модели (translation + scale)
                    let radius = match shape {
                        CollisionShape::Sphere(r) => *r,
                        _ => 1.0,
                    };
                    
                    let model = Mat4::from_scale_rotation_translation(
                        Vec3::new(radius, radius, radius),
                        glam::Quat::IDENTITY,
                        Vec3::new(pos.x, pos.y, pos.z),
                    );
                    
                    // Цвет на основе позиции (для визуального разнообразия)
                    let color = [
                        0.5 + 0.5 * ((pos.x * 0.1).sin()),
                        0.5 + 0.5 * ((pos.y * 0.1).sin()),
                        0.5 + 0.5 * ((pos.z * 0.1).sin()),
                        1.0,
                    ];
                    
                    instances.push(InstanceData {
                        model: model.to_cols_array_2d(),
                        color,
                    });
                }
            }
        }
        
        self.instance_count = instances.len() as u32;
        
        // Обновить instance буфер
        if !instances.is_empty() {
            queue.write_buffer(
                instance_buffer,
                0,
                bytemuck::cast_slice(&instances),
            );
        }
        
        // Обновить camera uniform
        let aspect = config.width as f32 / config.height as f32;
        let view_proj = self.create_view_projection(aspect);
        let camera_data = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
        };
        
        queue.write_buffer(
            camera_uniform_buffer,
            0,
            bytemuck::bytes_of(&camera_data),
        );

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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            
            if self.instance_count > 0 {
                render_pass.draw_indexed(0..self.index_count, 0, 0..self.instance_count);
            }
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

