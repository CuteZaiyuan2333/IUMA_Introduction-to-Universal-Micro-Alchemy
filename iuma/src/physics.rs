use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

/// Calculates forces and updates velocities based on "Field" interactions.
/// MVP Version: Brute-force O(N^2) CPU calculation to simulate fields.
pub fn particle_interaction_system(
    mut query: Query<(Entity, &mut Velocity, &Transform, &ParticleTypeID, &Mass)>,
    global_consts: Res<GlobalConstants>,
    alchemy: Res<AlchemyRules>,
) {
    // 1. Collect all particle positions first (to avoid borrowing issues)
    // Store reference to the whole FieldShape to access LUT
    let particles: Vec<(Entity, Vec3, Option<usize>, &FieldShape)> = query.iter()
        .map(|(e, _, t, pid, _)| {
            let p_def = &alchemy.particle_types[pid.0];
            (
                e, 
                t.translation, 
                p_def.emits_field, 
                &p_def.emission_shape
            )
        })
        .collect();

    // 2. Iterate and apply forces
    for (entity, mut velocity, transform, type_id, mass) in query.iter_mut() {
        let mut total_force = Vec2::ZERO;
        let my_pos = transform.translation.truncate();
        let my_type = type_id.0;

        for (other_e, other_pos_3d, other_emits_field, field_shape) in &particles {
            if entity == *other_e { continue; } 

            let field_id = match other_emits_field {
                Some(id) => id,
                None => continue,
            };

            let weight = alchemy.interactions.get(&(my_type, *field_id)).copied().unwrap_or(0.0);
            if weight == 0.0 { continue; }

            let other_pos = other_pos_3d.truncate();
            let delta = other_pos - my_pos;
            let distance = delta.length();
            
            // Check radius
            if distance < 0.1 || distance > field_shape.max_radius { continue; } 

            let direction = delta / distance;

            // --- CURVE LOOKUP ---
            // Normalize distance to 0..1
            let t = distance / field_shape.max_radius;
            
            // Map to LUT index (0..99)
            // Clamp index to be safe
            let lut_idx = ((t * (field_shape.lut.len() - 1) as f32) as usize).clamp(0, field_shape.lut.len() - 1);
            
            // Sample
            let normalized_strength = field_shape.lut[lut_idx];
            
            // Apply scale
            // Note: In 1/r models, strength is huge near 0. 
            // Our curve returns 0..1 (or -1..1). 
            // So we multiply by strength_scale.
            let field_strength = normalized_strength * field_shape.strength_scale;

            // Force = Direction * Strength * Weight
            total_force += direction * field_strength * weight;
        }

        let acceleration = total_force / mass.0;
        let dt = 0.016 * global_consts.time_scale; 

        velocity.0 += acceleration * dt;
    }
}

/// Applies velocity to position and enforces Light Speed
pub fn physics_integration_system(
    mut query: Query<(&mut Transform, &mut Velocity)>,
    global_consts: Res<GlobalConstants>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds() * global_consts.time_scale;
    let c = global_consts.light_speed;

    for (mut transform, mut velocity) in query.iter_mut() {
        let speed = velocity.0.length();
        if speed > c {
            velocity.0 = velocity.0.normalize() * c;
        }

        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;
    }
}