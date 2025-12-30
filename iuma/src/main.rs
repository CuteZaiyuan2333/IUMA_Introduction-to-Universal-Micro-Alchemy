mod components;
mod resources;
mod ui;
mod physics;
mod render;

use bevy::prelude::*;
use components::*;
use resources::*;
use bevy_egui::EguiPlugin;
use render::FieldVisPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "IUMA: Introduction to Universal Micro Alchemy".into(),
                resolution: (1280.0, 720.0).into(),
                present_mode: bevy::window::PresentMode::AutoNoVsync, // Unlock FPS for smoother sim
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(FieldVisPlugin) // Add our custom rendering plugin
        .init_resource::<GlobalConstants>()
        .init_resource::<AlchemyRules>()
        
        .add_systems(Startup, setup_camera)
        
        // Physics & Logic
        .add_systems(Update, (
            ui::ui_system,
            ui::sync_field_visualization, // Add sync system
            physics::particle_interaction_system,
            physics::physics_integration_system,
        ).chain()) // .chain() ensures order if needed, though here interaction -> integration is implicitly handled by component mutation
        
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
