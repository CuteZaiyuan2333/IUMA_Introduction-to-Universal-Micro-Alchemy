use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource)]
pub struct GlobalConstants {
    pub light_speed: f32,
    pub time_scale: f32,
    pub bloom_enabled: bool,
    pub bloom_intensity: f32,
}

impl Default for GlobalConstants {
    fn default() -> Self {
        Self {
            light_speed: 1000.0,
            time_scale: 1.0,
            bloom_enabled: true,
            bloom_intensity: 0.3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CurvePoint {
    pub x: f32, // Normalized Distance (0.0 to 1.0)
    pub y: f32, // Normalized Strength (-1.0 to 1.0)
}

/// Defines the shape of a field emitted by a particle.
#[derive(Debug, Clone)]
pub struct FieldShape {
    pub max_radius: f32,
    pub strength_scale: f32, 
    
    // User defined points for the curve editor
    pub points: Vec<CurvePoint>,
    
    // Baked Lookup Table for fast physics
    // Size should be e.g., 100
    pub lut: Vec<f32>, 
}

impl FieldShape {
    pub fn new_linear_falloff(radius: f32, strength: f32) -> Self {
        let mut shape = Self {
            max_radius: radius,
            strength_scale: strength,
            points: vec![
                CurvePoint { x: 0.0, y: 1.0 },
                CurvePoint { x: 1.0, y: 0.0 },
            ],
            lut: Vec::new(),
        };
        shape.bake_lut();
        shape
    }

    pub fn bake_lut(&mut self) {
        let size = 100;
        self.lut.clear();
        
        // Sort points by x just in case
        self.points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

        for i in 0..size {
            let t = i as f32 / (size - 1) as f32; // 0.0 to 1.0
            
            // Linear Interpolation between points
            let val = self.sample_points(t);
            self.lut.push(val);
        }
    }

    fn sample_points(&self, t: f32) -> f32 {
        if self.points.is_empty() { return 0.0; }
        if t <= self.points[0].x { return self.points[0].y; }
        if t >= self.points.last().unwrap().x { return self.points.last().unwrap().y; }

        for window in self.points.windows(2) {
            let p0 = &window[0];
            let p1 = &window[1];
            if t >= p0.x && t <= p1.x {
                // Lerp
                let factor = (t - p0.x) / (p1.x - p0.x);
                return p0.y + (p1.y - p0.y) * factor;
            }
        }
        0.0
    }
}

/// The definition of a particle type (The "Blueprint")
#[derive(Debug, Clone)]
pub struct ParticleTypeDefinition {
    pub name: String,
    pub default_mass: f32,
    pub default_color: Color,
    pub emits_field: Option<usize>, 
    pub emission_shape: FieldShape,
}

/// The central Alchemy definition
#[derive(Resource)]
pub struct AlchemyRules {
    pub particle_types: Vec<ParticleTypeDefinition>,
    // Interaction Matrix: How particle type P responds to field F
    // Key: (ParticleTypeID, FieldID), Value: Weight
    pub interactions: HashMap<(usize, usize), f32>,
}

impl Default for AlchemyRules {
    fn default() -> Self {
        let mut rules = Self {
            particle_types: Vec::new(),
            interactions: HashMap::new(),
        };

        // Define Type 0: Proton-like
        rules.particle_types.push(ParticleTypeDefinition {
            name: "Proton".to_string(),
            default_mass: 1.0,
            default_color: Color::RED,
            emits_field: Some(0),
            emission_shape: FieldShape::new_linear_falloff(300.0, 1000.0),
        });

        // Define Type 1: Electron-like
        rules.particle_types.push(ParticleTypeDefinition {
            name: "Electron".to_string(),
            default_mass: 1.0,
            default_color: Color::BLUE,
            emits_field: Some(1),
            emission_shape: FieldShape::new_linear_falloff(300.0, 1000.0),
        });

        // Default Interactions
        rules.interactions.insert((0, 0), 1.0);
        rules.interactions.insert((1, 1), 1.0);
        rules.interactions.insert((0, 1), -2.0);
        rules.interactions.insert((1, 0), -2.0);

        rules
    }
}