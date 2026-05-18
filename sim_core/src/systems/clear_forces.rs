//! Система очистки сил - обнуляет силы после каждого шага

use crate::world::World;
use crate::components::transform::Force;

/// Очистить все накопленные силы
pub fn clear_forces(world: &mut World) {
    for arch in world.archetypes_mut() {
        if !arch.has_component::<Force>() {
            continue;
        }
        
        let forces = arch.get_component_slice_mut::<Force>();
        for i in 0..arch.len {
            forces[i].x = 0.0;
            forces[i].y = 0.0;
            forces[i].z = 0.0;
        }
    }
}
