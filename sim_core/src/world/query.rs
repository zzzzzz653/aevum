//! Query — итерация по архетипам с нужными компонентами

use super::Archetype;

/// Query для выбора архетипов с определёнными компонентами
pub struct Query {
    required_components: Vec<std::any::TypeId>,
}

impl Query {
    pub fn default() -> Self {
        Self {
            required_components: Vec::new(),
        }
    }

    pub fn with<T: 'static>(mut self) -> Self {
        self.required_components.push(std::any::TypeId::of::<T>());
        self
    }

    pub fn matches_archetype(&self, arch: &Archetype) -> bool {
        for type_id in &self.required_components {
            if !arch.arrays.contains_key(type_id) {
                return false;
            }
        }
        true
    }
}
