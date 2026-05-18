//! Archetype — SoA хранилище компонентов для одного набора типов

use std::any::TypeId;
use std::collections::HashMap;
use smallvec::SmallVec;

use super::{EntityId, ComponentMask};

pub type ArchetypeId = u32;

/// Архетип хранит компоненты в Structure of Arrays формате
/// для кэш-френдли итерации по одному типу компонента
pub struct Archetype {
    pub id: ArchetypeId,
    pub mask: ComponentMask,
    /// Количество сущностей в этом архетипе
    pub len: usize,
    /// EntityId для каждой сущности (индекс → EntityId)
    pub entities: SmallVec<[EntityId; 64]>,
    /// Индекс EntityId → позиция в arrays
    pub entity_index: HashMap<u64, usize>,
    /// Компонентные массивы (TypeId → данные)
    pub arrays: HashMap<TypeId, Box<dyn ArchetypeArray>>,
}

/// Trait для эрasure типизации компонентных массивов
pub trait ArchetypeArray: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn swap_remove(&mut self, index: usize);
    fn push_default(&mut self);
    fn clone_at(&mut self, src_index: usize);
}

impl<T: Clone + Default + Send + Sync + 'static> ArchetypeArray for SmallVec<[T; 64]> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn swap_remove(&mut self, index: usize) {
        self.swap_remove(index);
    }

    fn push_default(&mut self) {
        self.push(T::default());
    }

    fn clone_at(&mut self, src_index: usize) {
        let value = self[src_index].clone();
        self.push(value);
    }
}

impl Archetype {
    pub fn new(id: ArchetypeId, mask: ComponentMask) -> Self {
        Self {
            id,
            mask,
            len: 0,
            entities: SmallVec::new(),
            entity_index: HashMap::new(),
            arrays: HashMap::new(),
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.entities.capacity()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Добавить новую сущность в архетип
    pub fn spawn(&mut self, entity_id: EntityId) -> usize {
        let index = self.len;
        self.entities.push(entity_id);
        self.entity_index.insert(entity_id.packed(), index);
        
        // Расширить все массивы компонентов
        for array in self.arrays.values_mut() {
            array.push_default();
        }
        
        self.len += 1;
        index
    }

    /// Удалить сущность по индексу (swap-remove)
    pub fn remove_at(&mut self, index: usize) -> Option<EntityId> {
        if index >= self.len {
            return None;
        }

        let entity_id = self.entities[index];
        self.entity_index.remove(&entity_id.packed());

        // Swap-remove для всех массивов
        for array in self.arrays.values_mut() {
            array.swap_remove(index);
        }

        // Заменить удалённый entity на последний
        if index < self.len - 1 {
            let last_entity = self.entities[self.len - 1];
            self.entities[index] = last_entity;
            self.entity_index.insert(last_entity.packed(), index);
        }

        self.entities.pop();
        self.len -= 1;

        Some(entity_id)
    }

    /// Проверить наличие компонента типа T
    pub fn has_component<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.arrays.contains_key(&type_id)
    }

    /// Получить срез компонентов типа T
    pub fn get_component_slice<T: 'static>(&self) -> &[T] {
        let type_id = TypeId::of::<T>();
        self.arrays
            .get(&type_id)
            .and_then(|arr| arr.as_any().downcast_ref::<SmallVec<[T; 64]>>())
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Получить мутабельный срез компонентов типа T
    pub fn get_component_slice_mut<T: 'static>(&mut self) -> &mut [T] {
        let type_id = TypeId::of::<T>();
        self.arrays
            .get_mut(&type_id)
            .and_then(|arr| arr.as_any_mut().downcast_mut::<SmallVec<[T; 64]>>())
            .map(|v| v.as_mut_slice())
            .unwrap_or(&mut [])
    }

    /// Получить компонент типа T для сущности по индексу
    pub fn get_component<T: 'static>(&self, index: usize) -> Option<&T> {
        self.get_component_slice::<T>().get(index)
    }

    /// Получить мутабельный компонент типа T для сущности по индексу
    pub fn get_component_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
        let slice = self.get_component_slice_mut::<T>();
        if index < slice.len() {
            Some(&mut slice[index])
        } else {
            None
        }
    }

    /// Добавить компонентный массив типа T
    pub fn add_component_array<T: Clone + Default + Send + Sync + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        if !self.arrays.contains_key(&type_id) {
            let mut array: SmallVec<[T; 64]> = SmallVec::new();
            array.resize(self.len, T::default());
            self.arrays.insert(type_id, Box::new(array));
        }
    }

    /// Добавить компонент сущности
    pub fn add_component<T: Clone + Default + Send + Sync + 'static>(&mut self, _entity: EntityId, component: T) {
        let type_id = TypeId::of::<T>();
        if !self.arrays.contains_key(&type_id) {
            let mut array: SmallVec<[T; 64]> = SmallVec::new();
            array.resize(self.len, T::default());
            array.push(component);
            self.arrays.insert(type_id, Box::new(array));
        } else {
            let array = self.arrays.get_mut(&type_id).unwrap();
            let vec = array.as_any_mut().downcast_mut::<SmallVec<[T; 64]>>().unwrap();
            vec.push(component);
        }
    }
}
