use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::render_asset::RenderAssetUsages;
use crate::resources::*;
use crate::components::*;
use crate::render::FieldMaterial;

pub fn ui_system(
    mut contexts: EguiContexts,
    mut global_consts: ResMut<GlobalConstants>,
    mut alchemy: ResMut<AlchemyRules>,
    mut commands: Commands,
    particle_query: Query<Entity, With<Particle>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FieldMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Ensure textures are initialized
    for p_def in alchemy.particle_types.iter_mut() {
        if p_def.field_texture.is_none() {
            let size = Extent3d { width: 128, height: 1, depth_or_array_layers: 1 };
            let mut image = Image::new_fill(
                size,
                TextureDimension::D2,
                &[255, 255, 255, 255],
                TextureFormat::Rgba8Unorm,
                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD
            );
            update_texture_from_shape(&mut image, &p_def.emission_shape);
            p_def.field_texture = Some(images.add(image));
        }
    }

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
             for entity in particle_query.iter() {
                 commands.entity(entity).despawn_recursive();
             }
        }
    });

    // 2. Alchemy Editor (Matrix)
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

    // 3. Field Shape Editor
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
                
                for i in points_to_remove.iter().rev() {
                    shape.points.remove(*i);
                }

                if ui.button("+ Add Point").clicked() {
                    shape.points.push(CurvePoint { x: 0.5, y: 0.5 });
                    changed = true;
                }

                // Logic Update
                if changed {
                    shape.bake_lut();
                    // Texture Update!
                    if let Some(handle) = &p_def.field_texture {
                        if let Some(image) = images.get_mut(handle) {
                            update_texture_from_shape(image, shape);
                        }
                    }
                }
                
                // Plot Preview
                let (response, painter) = ui.allocate_painter(bevy_egui::egui::Vec2::new(300.0, 100.0), egui::Sense::hover());
                let rect = response.rect;
                
                painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));
                let zero_y = rect.center().y;
                painter.line_segment(
                    [egui::Pos2::new(rect.left(), zero_y), egui::Pos2::new(rect.right(), zero_y)],
                    egui::Stroke::new(1.0, egui::Color32::GRAY),
                );

                if shape.lut.len() > 1 {
                    let points: Vec<egui::Pos2> = shape.lut.iter().enumerate().map(|(i, &val)| {
                        let t = i as f32 / (shape.lut.len() - 1) as f32;
                        let x = rect.left() + t * rect.width();
                        let normalized_y = -val; 
                        let y = rect.center().y + normalized_y * (rect.height() / 2.0);
                        egui::Pos2::new(x, y)
                    }).collect();
                    painter.add(egui::Shape::line(points, egui::Stroke::new(2.0, egui::Color32::YELLOW)));
                }
            });
        }
    });
}

fn update_texture_from_shape(image: &mut Image, shape: &FieldShape) {
    let width = image.texture_descriptor.size.width as usize;
    let data = &mut image.data;
    
    for i in 0..width {
        let t = i as f32 / (width - 1) as f32;
        let lut_idx = ((t * (shape.lut.len() - 1) as f32) as usize).clamp(0, shape.lut.len() - 1);
        let val = shape.lut[lut_idx];
        
        let pixel_val = (val.abs() * 255.0).clamp(0.0, 255.0) as u8;
        
        let idx = i * 4;
        if idx + 3 < data.len() {
            data[idx] = pixel_val;     // R
            data[idx+1] = pixel_val;   // G
            data[idx+2] = pixel_val;   // B
            data[idx+3] = 255;         // A
        }
    }
}

/// Syncs the visuals (Mesh Scale & Intensity) with the AlchemyRules
pub fn sync_field_visualization(
    alchemy: Res<AlchemyRules>,
    particle_query: Query<(Entity, &ParticleTypeID, &Children), With<Particle>>,
    mut transform_query: Query<&mut Transform>,
    mut material_handles: Query<&mut Handle<FieldMaterial>>,
    mut materials: ResMut<Assets<FieldMaterial>>,
) {
    for (_entity, type_id, children) in particle_query.iter() {
        let def = &alchemy.particle_types[type_id.0];
        
        for child in children.iter() {
            // 1. Update Intensity 
            if let Ok(mat_handle) = material_handles.get_mut(*child) {
                if let Some(material) = materials.get_mut(mat_handle.id()) {
                     // Intensity is now purely a multiplier.
                     let target_intensity = (def.emission_shape.strength_scale / 1000.0).clamp(0.2, 2.0);
                     if (material.intensity - target_intensity).abs() > 0.01 {
                         material.intensity = target_intensity;
                     }
                }
            }

            // 2. Update Scale
            if material_handles.get(*child).is_ok() {
                if let Ok(mut transform) = transform_query.get_mut(*child) {
                    let target_scale = def.emission_shape.max_radius;
                    if (transform.scale.x - target_scale).abs() > 0.1 {
                        transform.scale = Vec3::new(target_scale, target_scale, 1.0);
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
            let mesh_handle = meshes.add(Mesh::from(Circle::new(1.0))); 
            let texture_handle = def.field_texture.clone().unwrap_or_default(); 
            let intensity = (def.emission_shape.strength_scale / 1000.0).clamp(0.2, 2.0);

            parent.spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(mesh_handle),
                material: materials.add(FieldMaterial {
                    color: def.default_color,
                    intensity,
                    lut_texture: texture_handle,
                }),
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, -0.1),
                    scale: Vec3::new(def.emission_shape.max_radius, def.emission_shape.max_radius, 1.0),
                    ..default()
                },
                ..default()
            });
        }
    });
}
