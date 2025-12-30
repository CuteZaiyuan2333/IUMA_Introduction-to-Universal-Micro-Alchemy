use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);

#[derive(Component, Debug, Clone, Copy)]
pub struct Mass(pub f32);

#[derive(Component, Debug, Clone, Copy)]
pub struct ParticleTypeID(pub usize);

/// Marks an entity as a simulation particle
#[derive(Component)]
pub struct Particle;

// Define a wrapper for Field Types to avoid confusion with raw integers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldTypeID(pub usize);
