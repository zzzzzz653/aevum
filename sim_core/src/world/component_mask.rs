//! ComponentMask — битовая маска компонентов (до 64 типов)

use std::ops::{BitOr, BitAnd, BitXor, Not};

/// Битовая маска компонентов для быстрого фильтра в ECS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ComponentMask {
    pub bits: u64,
}

impl ComponentMask {
    #[inline]
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    #[inline]
    pub const fn all() -> Self {
        Self { bits: u64::MAX }
    }

    #[inline]
    pub const fn with(mut self, component_id: u8) -> Self {
        assert!(component_id < 64, "Component ID must be < 64");
        self.bits |= 1u64 << component_id;
        self
    }

    #[inline]
    pub const fn without(mut self, component_id: u8) -> Self {
        assert!(component_id < 64, "Component ID must be < 64");
        self.bits &= !(1u64 << component_id);
        self
    }

    #[inline]
    pub const fn has(&self, component_id: u8) -> bool {
        assert!(component_id < 64, "Component ID must be < 64");
        self.bits & (1u64 << component_id) != 0
    }

    #[inline]
    pub const fn intersects(&self, other: Self) -> bool {
        self.bits & other.bits != 0
    }

    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }

    #[inline]
    pub fn count(&self) -> u32 {
        self.bits.count_ones()
    }
}

impl BitOr for ComponentMask {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self { bits: self.bits | rhs.bits }
    }
}

impl BitAnd for ComponentMask {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self { bits: self.bits & rhs.bits }
    }
}

impl BitXor for ComponentMask {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self { bits: self.bits ^ rhs.bits }
    }
}

impl Not for ComponentMask {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self { bits: !self.bits }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_operations() {
        let m1 = ComponentMask::empty().with(0).with(2);
        let m2 = ComponentMask::empty().with(1).with(2);
        
        assert!(m1.has(0));
        assert!(!m1.has(1));
        assert!(m1.has(2));
        
        let union = m1 | m2;
        assert!(union.has(0));
        assert!(union.has(1));
        assert!(union.has(2));
        
        let intersection = m1 & m2;
        assert!(!intersection.has(0));
        assert!(!intersection.has(1));
        assert!(intersection.has(2));
    }
}
