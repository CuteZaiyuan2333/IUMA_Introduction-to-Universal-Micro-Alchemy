mod components;
mod resources;
mod ui;
mod physics;
mod render;
mod camera; // New module

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
                present_mode: bevy::window::PresentMode::AutoNoVsync, 
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(FieldVisPlugin)
        .init_resource::<GlobalConstants>()
        .init_resource::<AlchemyRules>()
        
        .add_systems(Startup, setup_camera)
        
        // Physics & Logic
        .add_systems(Update, (
            camera::camera_control_system, // Add camera control
            ui::ui_system,
            ui::sync_field_visualization, 
            physics::particle_interaction_system,
            physics::physics_integration_system,
        ).chain())
        
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}