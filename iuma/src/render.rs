use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin};
use bevy::core_pipeline::bloom::BloomSettings;
use crate::resources::GlobalConstants;

pub struct FieldVisPlugin;

impl Plugin for FieldVisPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<FieldMaterial>::default())
           .add_systems(Update, toggle_bloom);
    }
}

/// A transparent, additive material that fades out from the center.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FieldMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub intensity: f32, // Multiplier for the alpha/brightness
}

impl Material2d for FieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/field_material.wgsl".into()
    }
}

fn toggle_bloom(
    mut commands: Commands,
    global_consts: Res<GlobalConstants>,
    camera_query: Query<(Entity, Option<&BloomSettings>), With<Camera>>,
) {
    if global_consts.is_changed() {
        let (cam_entity, bloom_settings) = camera_query.single();
        
        if global_consts.bloom_enabled {
            if bloom_settings.is_none() {
                commands.entity(cam_entity).insert(BloomSettings {
                    intensity: global_consts.bloom_intensity,
                    ..default()
                });
            } else if let Some(current) = bloom_settings {
                 // Update intensity if changed
                 if (current.intensity - global_consts.bloom_intensity).abs() > 0.01 {
                     commands.entity(cam_entity).insert(BloomSettings {
                        intensity: global_consts.bloom_intensity,
                        ..default()
                     });
                 }
            }
        } else {
            if bloom_settings.is_some() {
                commands.entity(cam_entity).remove::<BloomSettings>();
            }
        }
    }
}