//! Narrowphase коллизий — детектирование контактов между парами форм

use crate::components::collision::{ContactPoint, ContactManifold};
use crate::components::physics::CollisionShape;
use crate::world::EntityId;

/// Менеджер контактов — хранит и обновляет манифольды
pub struct ContactManager {
    manifolds: Vec<ContactManifold>,
}

impl ContactManager {
    pub fn new() -> Self {
        Self {
            manifolds: Vec::new(),
        }
    }

    /// Найти контакты между двумя сущностями
    pub fn find_contacts(
        &mut self,
        entity_a: EntityId,
        shape_a: &CollisionShape,
        pos_a: [f32; 3],
        entity_b: EntityId,
        shape_b: &CollisionShape,
        pos_b: [f32; 3],
    ) -> Option<&ContactManifold> {
        // Простая проверка только для сфер пока
        if shape_a.tag == CollisionShape::SPHERE && shape_b.tag == CollisionShape::SPHERE {
            return self.sphere_sphere(entity_a, shape_a, pos_a, entity_b, shape_b, pos_b);
        }
        
        // Для других форм — заглушка
        None
    }

    /// Sphere-Sphere контакт
    fn sphere_sphere(
        &mut self,
        entity_a: EntityId,
        shape_a: &CollisionShape,
        pos_a: [f32; 3],
        entity_b: EntityId,
        shape_b: &CollisionShape,
        pos_b: [f32; 3],
    ) -> Option<&ContactManifold> {
        let r_a = shape_a.radius();
        let r_b = shape_b.radius();
        
        let dx = pos_b[0] - pos_a[0];
        let dy = pos_b[1] - pos_a[1];
        let dz = pos_b[2] - pos_a[2];
        
        let dist_sq = dx * dx + dy * dy + dz * dz;
        let radius_sum = r_a + r_b;
        
        // Нет контакта если расстояние больше суммы радиусов
        if dist_sq >= radius_sum * radius_sum {
            return None;
        }
        
        let dist = dist_sq.sqrt();
        
        // Нормаль от A к B
        let normal = if dist > 0.0001 {
            [dx / dist, dy / dist, dz / dist]
        } else {
            [1.0, 0.0, 0.0] // вырожденный случай
        };
        
        // Глубина проникновения
        let penetration = radius_sum - dist;
        
        // Точка контакта (посередине проникновения)
        let contact_point = [
            pos_a[0] + normal[0] * (r_a - penetration * 0.5),
            pos_a[1] + normal[1] * (r_a - penetration * 0.5),
            pos_a[2] + normal[2] * (r_a - penetration * 0.5),
        ];
        
        // Создать или обновить манифольд
        let mat_pair = (0u32, 0u32); // TODO: взять из материалов
        
        // Ищем существующий манифольд
        let manifold_idx = self.manifolds.iter().position(|m| {
            (m.entity_a == entity_a.packed() && m.entity_b == entity_b.packed()) ||
            (m.entity_a == entity_b.packed() && m.entity_b == entity_a.packed())
        });
        
        if let Some(idx) = manifold_idx {
            let manifold = &mut self.manifolds[idx];
            manifold.age += 1;
            manifold.contacts.clear();
            manifold.contacts.push(ContactPoint {
                position: contact_point,
                normal,
                penetration,
                impulse_normal: 0.0,
                impulse_tangent: [0.0; 2],
                contact_id: idx as u64,
            });
            return Some(&self.manifolds[idx]);
        } else {
            // Новый манифольд
            let mut manifold = ContactManifold::new(entity_a.packed(), entity_b.packed(), mat_pair);
            manifold.add_contact(ContactPoint {
                position: contact_point,
                normal,
                penetration,
                impulse_normal: 0.0,
                impulse_tangent: [0.0; 2],
                contact_id: self.manifolds.len() as u64,
            });
            self.manifolds.push(manifold);
            return self.manifolds.last();
        }
    }

    /// Получить все активные манифольды
    pub fn manifolds(&self) -> &[ContactManifold] {
        &self.manifolds
    }

    /// Очистить старые контакты
    pub fn clear_old(&mut self, max_age: u32) {
        self.manifolds.retain(|m| m.age < max_age);
    }

    /// Сбросить импульсы для нового шага
    pub fn reset_impulses(&mut self) {
        for manifold in &mut self.manifolds {
            for contact in &mut manifold.contacts {
                contact.impulse_normal = 0.0;
                contact.impulse_tangent = [0.0; 2];
            }
        }
    }
}

impl Default for ContactManager {
    fn default() -> Self {
        Self::new()
    }
}
