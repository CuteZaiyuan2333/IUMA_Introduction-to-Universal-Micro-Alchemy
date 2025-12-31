use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin};

pub struct FieldVisPlugin;

impl Plugin for FieldVisPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<FieldMaterial>::default());
    }
}

/// A transparent, additive material that fades out from the center.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FieldMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub intensity: f32,
    
    #[texture(1)]
    #[sampler(2)]
    pub lut_texture: Handle<Image>,
}

impl Material2d for FieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/field_material.wgsl".into()
    }
}
