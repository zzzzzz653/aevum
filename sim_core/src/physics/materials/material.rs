use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Material {
    pub name: [u8; 32],
    pub static_friction: f32,
    pub dynamic_friction: f32,
    pub restitution: f32,
    pub density: f32,
    pub compliance: f32,
    pub color: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
    pub damage_threshold: f32,
    pub damage_multiplier: f32,
    pub deformation: f32,
    pub sound_absorption: f32,
    pub thermal_conductivity: f32,
    pub burn_rate: f32,
    pub melt_point: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: [0; 32],
            static_friction: 0.5,
            dynamic_friction: 0.3,
            restitution: 0.1,
            density: 1000.0,
            compliance: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            roughness: 0.5,
            metallic: 0.0,
            damage_threshold: 10.0,
            damage_multiplier: 1.0,
            deformation: 0.0,
            sound_absorption: 0.1,
            thermal_conductivity: 1.0,
            burn_rate: 0.0,
            melt_point: 0.0,
        }
    }
}
