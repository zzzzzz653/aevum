//! World — главный ECS контейнер

use super::{Archetype, ArchetypeId, ComponentMask, EntityId};

/// Мировой таймер для детерминированной симуляции
#[derive(Debug, Clone)]
pub struct TickData {
    pub current_tick: u64,
    pub dt: f32,
}

impl Default for TickData {
    fn default() -> Self {
        Self {
            current_tick: 0,
            dt: 0.01, // 100 Hz по умолчанию
        }
    }
}

/// Главный ECS мир
pub struct World {
    /// Архетипы, ключ = битовая маска компонентов
    archetypes: std::collections::HashMap<u64, Archetype>,
    /// Счётчик сгенерированных entity
    entity_counter: u64,
    /// Mapping packed EntityId → (ArchetypeId, index)
    entity_location: std::collections::HashMap<u64, (ArchetypeId, usize)>,
    /// Следующий доступный archetype ID
    next_archetype_id: ArchetypeId,
    /// Данные тика
    pub tick_data: TickData,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        let mut world = Self {
            archetypes: std::collections::HashMap::new(),
            entity_counter: 0,
            entity_location: std::collections::HashMap::new(),
            next_archetype_id: 0,
            tick_data: TickData::default(),
        };
        // Создать пустой архетип для сущностей без компонентов
        world.get_or_create_archetype(ComponentMask::empty());
        world
    }

    /// Получить или создать архетип по маске
    fn get_or_create_archetype(&mut self, mask: ComponentMask) -> &mut Archetype {
        let key = mask.bits;
        if !self.archetypes.contains_key(&key) {
            let id = self.next_archetype_id;
            self.next_archetype_id += 1;
            self.archetypes.insert(key, Archetype::new(id, mask));
        }
        self.archetypes.get_mut(&key).unwrap()
    }

    /// Создать новую сущность
    pub fn spawn(&mut self) -> EntityId {
        let generation = (self.entity_counter >> 32) as u32;
        let index = self.entity_counter as u32;
        let entity_id = EntityId::new(index, generation);
        self.entity_counter += 1;

        // Добавить в пустой архетип - нужно получить arch_index до того как отпустим borrow
        let (arch_id, arch_index) = {
            let arch = self.get_or_create_archetype(ComponentMask::empty());
            let arch_index = arch.spawn(entity_id);
            (arch.id, arch_index)
        };
        
        self.entity_location.insert(entity_id.packed(), (arch_id, arch_index));

        entity_id
    }

    /// Удалить сущность
    pub fn despawn(&mut self, entity_id: EntityId) -> bool {
        if let Some(&(arch_id, index)) = self.entity_location.get(&entity_id.packed()) {
            // Найти архетип и удалить
            if let Some(arch_key) = self.find_archetype_key(arch_id) {
                if let Some(arch) = self.archetypes.get_mut(&arch_key) {
                    arch.remove_at(index);
                    self.entity_location.remove(&entity_id.packed());

                    // Обновить индексы для оставшихся сущностей
                    for idx in index..arch.len {
                        let other_entity = arch.entities[idx];
                        self.entity_location.insert(other_entity.packed(), (arch_id, idx));
                    }

                    return true;
                }
            }
        }
        false
    }

    fn find_archetype_key(&self, arch_id: ArchetypeId) -> Option<u64> {
        self.archetypes.iter().find_map(|(&k, v)| {
            if v.id == arch_id { Some(k) } else { None }
        })
    }

    /// Получить все архетипы
    pub fn archetypes(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.values()
    }

    /// Получить мутабельные архетипы
    pub fn archetypes_mut(&mut self) -> impl Iterator<Item = &mut Archetype> {
        self.archetypes.values_mut()
    }

    /// Инкремент тика
    pub fn step(&mut self, dt: f32) {
        self.tick_data.current_tick += 1;
        self.tick_data.dt = dt;
    }

    /// Получить текущий тик
    pub fn current_tick(&self) -> u64 {
        self.tick_data.current_tick
    }

    /// Количество сущностей
    pub fn entity_count(&self) -> usize {
        self.entity_location.len()
    }

    /// Проверка существования сущности
    pub fn exists(&self, entity_id: EntityId) -> bool {
        self.entity_location.contains_key(&entity_id.packed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_despawn() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        
        assert!(world.exists(e1));
        assert!(world.exists(e2));
        assert_eq!(world.entity_count(), 2);

        world.despawn(e1);
        assert!(!world.exists(e1));
        assert!(world.exists(e2));
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_tick() {
        let mut world = World::new();
        assert_eq!(world.current_tick(), 0);
        world.step(0.01);
        assert_eq!(world.current_tick(), 1);
    }
}
