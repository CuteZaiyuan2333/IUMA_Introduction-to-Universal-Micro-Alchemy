#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct FieldMaterial {
    color: vec4<f32>,
    intensity: f32,
};

@group(2) @binding(0) var<uniform> material: FieldMaterial;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // UV coordinates are 0.0 to 1.0. Center is 0.5, 0.5.
    let uv = mesh.uv;
    let center = vec2<f32>(0.5, 0.5);
    
    // Calculate distance from center
    let dist = distance(uv, center);
    
    // Circular cutout (radius 0.5)
    if (dist > 0.5) {
        discard;
    }

    // Radial Falloff (1.0 at center, 0.0 at edge)
    // We can tweak this power to make the field "softer" or "sharper"
    // Using 1.0 / dist mimics gravity/electric field falloff nicely but needs clamping
    // Let's use a soft smoothstep for better visual aesthetics
    let falloff = 1.0 - smoothstep(0.0, 0.5, dist);
    
    // Apply intensity
    let alpha = falloff * material.intensity;

    return vec4<f32>(material.color.rgb, alpha);
}
