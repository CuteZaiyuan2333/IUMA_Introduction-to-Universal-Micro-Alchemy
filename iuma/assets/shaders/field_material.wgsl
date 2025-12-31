#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct FieldMaterial {
    color: vec4<f32>,
    intensity: f32,
};

@group(2) @binding(0) var<uniform> material: FieldMaterial;
@group(2) @binding(1) var lut_texture: texture_2d<f32>; // Actually 1D data, but stored in 2D for compatibility
@group(2) @binding(2) var lut_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(mesh.uv, center);
    
    // Circular cutout (radius 0.5 in UV space)
    if (dist > 0.5) {
        discard;
    }

    // Map distance (0.0 -> 0.5) to UV (0.0 -> 1.0)
    // dist * 2.0 covers the full radius
    let sample_u = dist * 2.0; 
    
    // Sample the LUT
    // We sample at (u, 0.5). Texture is Nx1 pixels.
    let curve_val = textureSample(lut_texture, lut_sampler, vec2<f32>(sample_u, 0.5)).r;

    // Apply intensity. 
    // Note: curve_val can be negative (repulsion), but for visuals we take abs() 
    // or maybe we only visualize attraction? Usually fields are energy, so abs() makes sense.
    // Let's use abs() so "strong repulsion" looks as bright as "strong attraction".
    let alpha = abs(curve_val) * material.intensity;

    return vec4<f32>(material.color.rgb, alpha);
}