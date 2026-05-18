//! Физические компоненты: RigidBody, массы, силы

use bytemuck::{Pod, Zeroable};

/// Тип физического тела
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BodyType {
    Static = 0,    // не двигается (земля, стены)
    Dynamic = 1,   // интегрируется, реагирует на силы
    Kinematic = 2, // двигается скриптом
}

impl Default for BodyType {
    fn default() -> Self {
        BodyType::Dynamic
    }
}

// BodyType не может быть Pod из-за enum, но это нормально для использования в RigidBody

/// Форма коллизии (union-style, 32 байта)
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CollisionShape {
    pub tag: u32,  // Sphere=0, Box=1, Capsule=2, Cylinder=3, ConvexHull=4, HeightField=5, None=6
    pub data: [f32; 7], // данные формы
}

impl CollisionShape {
    pub const SPHERE: u32 = 0;
    pub const BOX: u32 = 1;
    pub const CAPSULE: u32 = 2;
    pub const CYLINDER: u32 = 3;
    pub const CONVEX_HULL: u32 = 4;
    pub const HEIGHT_FIELD: u32 = 5;
    pub const NONE: u32 = 6;

    pub fn sphere(radius: f32) -> Self {
        Self {
            tag: Self::SPHERE,
            data: [radius, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn box_shape(half_extents: [f32; 3]) -> Self {
        Self {
            tag: Self::BOX,
            data: [half_extents[0], half_extents[1], half_extents[2], 0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn capsule(radius: f32, half_height: f32) -> Self {
        Self {
            tag: Self::CAPSULE,
            data: [radius, half_height, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }

    #[inline]
    pub fn radius(&self) -> f32 {
        self.data[0]
    }

    #[inline]
    pub fn half_extents(&self) -> [f32; 3] {
        [self.data[0], self.data[1], self.data[2]]
    }
}

impl Default for CollisionShape {
    fn default() -> Self {
        Self::sphere(1.0)
    }
}

/// Жёсткое тело
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RigidBody {
    pub body_type: BodyType,
    pub shape: CollisionShape,
    pub mass: f32,
    pub inv_mass: f32,
    pub inertia: [f32; 9],      // 3x3 матрица
    pub inv_inertia: [f32; 9],
    pub material_id: u32,
    pub sleeping: bool,
}

// RigidBody не может быть Pod из-за BodyType и bool, но это нормально

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            body_type: BodyType::Dynamic,
            shape: CollisionShape::default(),
            mass: 1.0,
            inv_mass: 1.0,
            inertia: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            inv_inertia: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            material_id: 0,
            sleeping: false,
        }
    }
}

impl RigidBody {
    pub fn new(body_type: BodyType, shape: CollisionShape, mass: f32) -> Self {
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        
        // Простая инерция для сферы/бокса
        let inertia = match shape.tag {
            CollisionShape::SPHERE => {
                let r = shape.radius();
                let ii = 0.4 * mass * r * r;
                [ii, 0.0, 0.0, 0.0, ii, 0.0, 0.0, 0.0, ii]
            }
            CollisionShape::BOX => {
                let e = shape.half_extents();
                [
                    mass * (e[1]*e[1] + e[2]*e[2]) / 3.0,
                    0.0, 0.0,
                    0.0,
                    mass * (e[0]*e[0] + e[2]*e[2]) / 3.0,
                    0.0,
                    0.0, 0.0,
                    mass * (e[0]*e[0] + e[1]*e[1]) / 3.0,
                ]
            }
            _ => [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        };

        let inv_inertia = if inv_mass > 0.0 {
            [
                if inertia[0] > 0.0 { 1.0 / inertia[0] } else { 0.0 },
                0.0, 0.0,
                0.0,
                if inertia[4] > 0.0 { 1.0 / inertia[4] } else { 0.0 },
                0.0,
                0.0, 0.0,
                if inertia[8] > 0.0 { 1.0 / inertia[8] } else { 0.0 },
            ]
        } else {
            [0.0; 9]
        };

        Self {
            body_type,
            shape,
            mass,
            inv_mass,
            inertia,
            inv_inertia,
            material_id: 0,
            sleeping: false,
        }
    }

    #[inline]
    pub fn is_static(&self) -> bool {
        self.body_type == BodyType::Static
    }

    #[inline]
    pub fn is_dynamic(&self) -> bool {
        self.body_type == BodyType::Dynamic
    }
}
