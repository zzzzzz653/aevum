//! EntityId — уникальный идентификатор сущности

use std::fmt;

/// EntityId: index + generation для безопасного переиспользования
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId {
    pub index: u32,       // индекс в архетипе
    pub generation: u32,  // поколение (alive/dead проверка)
}

impl EntityId {
    #[inline]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    #[inline]
    pub const fn packed(&self) -> u64 {
        ((self.generation as u64) << 32) | (self.index as u64)
    }

    #[inline]
    pub const fn from_packed(packed: u64) -> Self {
        Self {
            index: packed as u32,
            generation: (packed >> 32) as u32,
        }
    }

    #[inline]
    pub const fn invalid() -> Self {
        Self { index: u32::MAX, generation: u32::MAX }
    }

    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.index != u32::MAX && self.generation != u32::MAX
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({}:{})", self.index, self.generation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack() {
        let id = EntityId::new(123, 456);
        let packed = id.packed();
        let unpacked = EntityId::from_packed(packed);
        assert_eq!(id, unpacked);
    }

    #[test]
    fn test_invalid() {
        let invalid = EntityId::invalid();
        assert!(!invalid.is_valid());
    }
}
