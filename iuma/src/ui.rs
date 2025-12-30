use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use crate::resources::*;
use crate::components::*;
use crate::render::FieldMaterial;

pub fn ui_system(
    mut contexts: EguiContexts,
    mut global_consts: ResMut<GlobalConstants>,
    mut alchemy: ResMut<AlchemyRules>,
    mut commands: Commands,
    particle_query: Query<&Particle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FieldMaterial>>,
) {
    // 1. Control Panel
    egui::Window::new("Universal Control").show(contexts.ctx_mut(), |ui| {
        ui.heading("Universe Constants");
        
        ui.horizontal(|ui| {
            ui.label("Light Speed (C):");
            ui.add(egui::Slider::new(&mut global_consts.light_speed, 100.0..=5000.0));
        });

        ui.horizontal(|ui| {
            ui.label("Time Scale:");
            ui.add(egui::Slider::new(&mut global_consts.time_scale, 0.0..=5.0));
        });
        
        ui.horizontal(|ui| {
            ui.checkbox(&mut global_consts.bloom_enabled, "Enable Bloom");
            if global_consts.bloom_enabled {
                ui.add(egui::Slider::new(&mut global_consts.bloom_intensity, 0.0..=1.0).text("Intensity"));
            }
        });

        ui.separator();

        ui.heading("Particle Spawner");
        ui.label(format!("Total Particles: {}", particle_query.iter().count()));

        for (idx, p_def) in alchemy.particle_types.iter().enumerate() {
            if ui.button(format!("Spawn {} (Type {})", p_def.name, idx)).clicked() {
                 spawn_particle(
                     &mut commands, 
                     idx, 
                     Vec2::new(0.0, 0.0), 
                     p_def, 
                     &mut meshes, 
                     &mut materials
                );
            }
        }
        
        ui.separator();
        if ui.button("Clear All Particles").clicked() {
             // To be implemented
        }
    });

    // 2. Alchemy Editor
    egui::Window::new("Alchemy Matrix").show(contexts.ctx_mut(), |ui| {
        ui.label("Interaction Weights (Force Multiplier)");
        
        let num_types = alchemy.particle_types.len();

        egui::Grid::new("interaction_matrix").striped(true).show(ui, |ui| {
            ui.label(""); 
            for i in 0..num_types {
                ui.centered_and_justified(|ui| {
                    ui.label(format!("Field\n{}", alchemy.particle_types[i].name));
                });
            }
            ui.end_row();

            for subject_idx in 0..num_types {
                ui.label(format!("Subject\n{}", alchemy.particle_types[subject_idx].name));
                
                for field_source_idx in 0..num_types {
                    if let Some(field_id) = alchemy.particle_types[field_source_idx].emits_field {
                        let weight = alchemy.interactions.entry((subject_idx, field_id)).or_insert(0.0);
                        ui.add(egui::DragValue::new(weight).speed(0.1));
                    } else {
                        ui.label("-");
                    }
                }
                ui.end_row();
            }
        });
    });

    // 3. Field Curve Editor
    egui::Window::new("Field Shape Editor").show(contexts.ctx_mut(), |ui| {
        for p_def in alchemy.particle_types.iter_mut() {
            ui.collapsing(format!("{} Field Shape", p_def.name), |ui| {
                let shape = &mut p_def.emission_shape;
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("Max Radius:");
                    if ui.add(egui::Slider::new(&mut shape.max_radius, 50.0..=1000.0)).changed() {
                        changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Global Strength:");
                    if ui.add(egui::Slider::new(&mut shape.strength_scale, 0.0..=5000.0)).changed() {
                        changed = true;
                    }
                });

                ui.separator();
                ui.label("Curve Points (Distance 0.0 -> 1.0)");
                
                // Point Editor
                let mut points_to_remove = Vec::new();
                for (i, point) in shape.points.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("P{}:", i));
                        if ui.add(egui::Slider::new(&mut point.x, 0.0..=1.0).text("Dist")).changed() { changed = true; }
                        if ui.add(egui::Slider::new(&mut point.y, -1.0..=1.0).text("Val")).changed() { changed = true; }
                        if ui.button("X").clicked() {
                            points_to_remove.push(i);
                            changed = true;
                        }
                    });
                }
                
                // Remove deleted points (reverse order to keep indices valid)
                for i in points_to_remove.iter().rev() {
                    shape.points.remove(*i);
                }

                if ui.button("+ Add Point").clicked() {
                    shape.points.push(CurvePoint { x: 0.5, y: 0.5 });
                    changed = true;
                }

                // If anything changed, re-bake the LUT
                if changed {
                    shape.bake_lut();
                }
                
                // --- PLOT PREVIEW ---
                // Try to use egui_plot if available. 
                // Since I cannot verify if 'bevy_egui' exports 'egui_plot' without checking Cargo features,
                // I will use a simple custom painter for now to avoid compilation errors.
                // It's safer and sufficient for MVP.
                
                let (response, painter) = ui.allocate_painter(bevy_egui::egui::Vec2::new(300.0, 100.0), egui::Sense::hover());
                let rect = response.rect;
                
                // Background
                painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));
                
                // Draw Zero Line
                let zero_y = rect.center().y;
                painter.line_segment(
                    [egui::Pos2::new(rect.left(), zero_y), egui::Pos2::new(rect.right(), zero_y)],
                    egui::Stroke::new(1.0, egui::Color32::GRAY),
                );

                // Draw Curve from LUT
                if shape.lut.len() > 1 {
                    let points: Vec<egui::Pos2> = shape.lut.iter().enumerate().map(|(i, &val)| {
                        let t = i as f32 / (shape.lut.len() - 1) as f32;
                        let x = rect.left() + t * rect.width();
                        // Map -1.0..1.0 to rect height. 0 is center.
                        // val=1.0 -> top, val=-1.0 -> bottom
                        // Actually, let's map 1.0 to top (min y), -1.0 to bottom (max y)
                        let normalized_y = -val; // invert because screen Y is down
                        let y = rect.center().y + normalized_y * (rect.height() / 2.0);
                        egui::Pos2::new(x, y)
                    }).collect();

                    painter.add(egui::Shape::line(points, egui::Stroke::new(2.0, egui::Color32::YELLOW)));
                }
            });
        }
    });
}

pub fn sync_field_visualization(
    alchemy: Res<AlchemyRules>,
    particle_query: Query<(Entity, &ParticleTypeID, &Children), With<Particle>>,
    mut material_handles: Query<&mut Handle<FieldMaterial>>,
    mut materials: ResMut<Assets<FieldMaterial>>,
) {
    for (_entity, type_id, children) in particle_query.iter() {
        let def = &alchemy.particle_types[type_id.0];
        
        for child in children.iter() {
            if let Ok(mat_handle) = material_handles.get_mut(*child) {
                if let Some(material) = materials.get_mut(mat_handle.id()) {
                     // Approximate visual intensity from the first point of the curve (usually x=0)
                     let start_val = def.emission_shape.points.first().map(|p| p.y).unwrap_or(1.0);
                     let target_intensity = (def.emission_shape.strength_scale * start_val.abs() / 2000.0).clamp(0.2, 1.0);
                     
                     if (material.intensity - target_intensity).abs() > 0.01 {
                         material.intensity = target_intensity;
                     }
                }
            }
        }
    }
}

fn spawn_particle(
    commands: &mut Commands, 
    type_id: usize, 
    pos: Vec2, 
    def: &ParticleTypeDefinition,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<FieldMaterial>>,
) {
    let mut rng = rand::thread_rng();
    use rand::Rng;
    
    let jitter_x = rng.gen_range(-50.0..50.0);
    let jitter_y = rng.gen_range(-50.0..50.0);
    let start_pos = Vec3::new(pos.x + jitter_x, pos.y + jitter_y, 0.0);

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE, 
                custom_size: Some(Vec2::new(4.0, 4.0)),
                ..default()
            },
            transform: Transform::from_translation(start_pos),
            ..default()
        },
        Particle,
        ParticleTypeID(type_id),
        Mass(def.default_mass),
        Velocity(Vec2::ZERO),
    ))
    .with_children(|parent| {
        if def.emits_field.is_some() {
            let radius = def.emission_shape.max_radius;
            let mesh_handle = meshes.add(Mesh::from(Circle::new(radius))); 
            
            let start_val = def.emission_shape.points.first().map(|p| p.y).unwrap_or(1.0);
            let intensity = (def.emission_shape.strength_scale * start_val.abs() / 2000.0).clamp(0.2, 1.0);

            parent.spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(mesh_handle),
                material: materials.add(FieldMaterial {
                    color: def.default_color,
                    intensity,
                }),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)), 
                ..default()
            });
        }
    });
}