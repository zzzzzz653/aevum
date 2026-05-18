//! Компоненты коллизий: AABB, ContactManifold

use bytemuck::{Pod, Zeroable};
use smallvec::SmallVec;

/// Axis-Aligned Bounding Box
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct AABB {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Default for AABB {
    fn default() -> Self {
        Self {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        }
    }
}

impl AABB {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    pub fn from_center_half_extents(center: [f32; 3], half_extents: [f32; 3]) -> Self {
        Self {
            min: [
                center[0] - half_extents[0],
                center[1] - half_extents[1],
                center[2] - half_extents[2],
            ],
            max: [
                center[0] + half_extents[0],
                center[1] + half_extents[1],
                center[2] + half_extents[2],
            ],
        }
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min[0] <= other.max[0] && self.max[0] >= other.min[0] &&
        self.min[1] <= other.max[1] && self.max[1] >= other.min[1] &&
        self.min[2] <= other.max[2] && self.max[2] >= other.min[2]
    }

    pub fn contains_point(&self, point: [f32; 3]) -> bool {
        point[0] >= self.min[0] && point[0] <= self.max[0] &&
        point[1] >= self.min[1] && point[1] <= self.max[1] &&
        point[2] >= self.min[2] && point[2] <= self.max[2]
    }

    pub fn volume(&self) -> f32 {
        (self.max[0] - self.min[0]) * 
        (self.max[1] - self.min[1]) * 
        (self.max[2] - self.min[2])
    }

    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }
}

/// Точка контакта между двумя телами
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ContactPoint {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub penetration: f32,
    pub impulse_normal: f32,
    pub impulse_tangent: [f32; 2],
    pub contact_id: u64,
}

impl Default for ContactPoint {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            penetration: 0.0,
            impulse_normal: 0.0,
            impulse_tangent: [0.0; 2],
            contact_id: 0,
        }
    }
}

/// Манифольд контактов между парой тел
pub struct ContactManifold {
    pub entity_a: u64,
    pub entity_b: u64,
    pub contacts: SmallVec<[ContactPoint; 4]>,
    pub age: u32,
    pub material_pair: (u32, u32),
}

impl ContactManifold {
    pub fn new(entity_a: u64, entity_b: u64, material_pair: (u32, u32)) -> Self {
        Self {
            entity_a,
            entity_b,
            contacts: SmallVec::new(),
            age: 0,
            material_pair,
        }
    }

    pub fn add_contact(&mut self, contact: ContactPoint) {
        self.contacts.push(contact);
    }

    pub fn clear(&mut self) {
        self.contacts.clear();
        self.age = 0;
    }

    pub fn tick(&mut self) {
        self.age += 1;
    }
}
