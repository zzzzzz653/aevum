//! Компоненты трансформации: позиция, вращение, скорость, силы

use glam::{Vec3, Quat};
use bytemuck::{Pod, Zeroable};

/// Позиция в мировом пространстве
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for Position {
    fn from(v: Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<Position> for Vec3 {
    fn from(p: Position) -> Self {
        Self::new(p.x, p.y, p.z)
    }
}

/// Вращение (кватернион)
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Rotation {
    fn default() -> Self {
        Self::from(Quat::IDENTITY)
    }
}

impl From<Quat> for Rotation {
    fn from(q: Quat) -> Self {
        Self { x: q.x, y: q.y, z: q.z, w: q.w }
    }
}

impl From<Rotation> for Quat {
    fn from(r: Rotation) -> Self {
        Self::from_array([r.x, r.y, r.z, r.w])
    }
}

impl Rotation {
    pub fn identity() -> Self {
        Self::default()
    }
}

/// Линейная скорость
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for Velocity {
    fn from(v: Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<Velocity> for Vec3 {
    fn from(v: Velocity) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

/// Угловая скорость
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct AngularVelocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for AngularVelocity {
    fn from(v: Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<AngularVelocity> for Vec3 {
    fn from(v: AngularVelocity) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

/// Сила, приложенная к телу (накапливается за тик)
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct Force {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for Force {
    fn from(v: Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<Force> for Vec3 {
    fn from(f: Force) -> Self {
        Self::new(f.x, f.y, f.z)
    }
}

impl Force {
    pub fn zero() -> Self {
        Self::default()
    }
}

/// Момент силы (torque)
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct Torque {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for Torque {
    fn from(v: Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<Torque> for Vec3 {
    fn from(t: Torque) -> Self {
        Self::new(t.x, t.y, t.z)
    }
}
