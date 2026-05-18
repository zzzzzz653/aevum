//! Broadphase коллизий — Spatial Hash Grid для быстрого поиска пар

use crate::components::collision::AABB;
use crate::world::{World, EntityId};
use std::collections::HashMap;

/// Ячейка пространственного хэша
type HashKey = (i32, i32, i32);

/// Spatial Hash Grid для broadphase коллизий
pub struct SpatialHashGrid {
    cell_size: f32,
    grid: HashMap<HashKey, Vec<usize>>,
}

impl SpatialHashGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
        }
    }

    /// Хэш позиции в ячейку
    fn hash_position(&self, pos: [f32; 3]) -> HashKey {
        (
            (pos[0] / self.cell_size).floor() as i32,
            (pos[1] / self.cell_size).floor() as i32,
            (pos[2] / self.cell_size).floor() as i32,
        )
    }

    /// Получить все ячейки, которые покрывает AABB
    fn get_covered_cells(&self, aabb: &AABB) -> Vec<HashKey> {
        let min_cell = self.hash_position(aabb.min);
        let max_cell = self.hash_position(aabb.max);
        
        let mut cells = Vec::new();
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    cells.push((x, y, z));
                }
            }
        }
        cells
    }

    /// Построить/grid из всех AABB
    pub fn build(&mut self, entities: &[EntityId], aabbs: &[AABB]) {
        self.grid.clear();
        
        for (idx, aabb) in aabbs.iter().enumerate() {
            let cells = self.get_covered_cells(aabb);
            for cell in cells {
                self.grid.entry(cell).or_insert_with(Vec::new).push(idx);
            }
        }
    }

    /// Найти потенциальные пары коллизий
    pub fn find_pairs(&self) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        let mut seen = std::collections::HashSet::new();
        
        for (_cell, entities) in self.grid.iter() {
            if entities.len() < 2 {
                continue;
            }
            
            for i in 0..entities.len() {
                for j in (i + 1)..entities.len() {
                    let a = entities[i];
                    let b = entities[j];
                    
                    // Упорядочить пару чтобы избежать дубликатов
                    let pair = if a < b { (a, b) } else { (b, a) };
                    
                    if seen.insert(pair) {
                        pairs.push(pair);
                    }
                }
            }
        }
        
        pairs
    }
}

/// Система broadphase — находит потенциальные пары коллизий
pub fn find_collision_pairs(world: &World) -> Vec<(EntityId, EntityId)> {
    let mut result = Vec::new();
    
    // Собрать все entity и их AABB
    let mut entities: Vec<EntityId> = Vec::new();
    let mut aabbs: Vec<AABB> = Vec::new();
    
    for arch in world.archetypes() {
        if !arch.has_component::<AABB>() {
            continue;
        }
        
        let arch_aabbs = arch.get_component_slice::<AABB>();
        for i in 0..arch.len {
            if let Some(entity) = arch.entities.get(i) {
                entities.push(*entity);
                aabbs.push(arch_aabbs[i]);
            }
        }
    }
    
    if entities.is_empty() {
        return result;
    }
    
    // Построить spatial hash
    let mut grid = SpatialHashGrid::new(2.0); // 2 единицы размер ячейки
    grid.build(&entities, &aabbs);
    
    // Найти пары
    for (idx_a, idx_b) in grid.find_pairs() {
        // Проверить пересечение AABB
        if aabbs[idx_a].intersects(&aabbs[idx_b]) {
            result.push((entities[idx_a], entities[idx_b]));
        }
    }
    
    result
}
