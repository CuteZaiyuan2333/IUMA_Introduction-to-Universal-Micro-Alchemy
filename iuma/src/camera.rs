use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

pub fn camera_control_system(
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    let (mut transform, mut projection) = query.single_mut();

    // 1. Pan (Middle Mouse Button)
    if mouse_buttons.pressed(MouseButton::Middle) {
        for event in mouse_motion_events.read() {
            // Move camera opposite to mouse motion to simulate "dragging the world"
            // Scale movement by zoom level so panning feels consistent
            transform.translation.x -= event.delta.x * projection.scale;
            transform.translation.y += event.delta.y * projection.scale;
        }
    }

    // 2. Zoom (Scroll Wheel)
    for event in mouse_wheel_events.read() {
        let zoom_speed = 0.1;
        // Decrease scale to zoom in, increase to zoom out
        projection.scale -= event.y * zoom_speed * projection.scale;
        
        // Clamp zoom level to reasonable limits
        projection.scale = projection.scale.clamp(0.1, 5.0);
    }
}
