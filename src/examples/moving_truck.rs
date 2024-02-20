use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_2d_screen_space_lightmaps::lightmap_plugin::lightmap_plugin::{CAMERA_LAYER_SPRITE, CAMERA_LAYER_LIGHT, LightmapPlugin, AnyNormalCamera};

const NORMAL_LIGHT_LAYER_Z: f32 = 0.0;
const OCCLUDER_LIGHT_LAYER_Z: f32 = 1.0;

const SPRITE_FLOOR_LAYER_Z: f32 = 0.0;
const SPRITE_OBJECT_LAYER_Z: f32 = 1.0;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { ..Default::default() }),
            ..default()
    }).set(ImagePlugin::default_nearest()));

    app.add_plugins(LightmapPlugin);
    app.add_systems(Startup, (spawn_road_entities, spawn_truck_entities));
    app.add_systems(Update, (move_truck, camera_movement));
    app.run();
}

fn spawn_road_entities(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let road_tex = asset_server.load("sample_art/road_segment.png");
    let grass_tex = asset_server.load("sample_art/grass.png");
    let y_top = -64.0 * 5.0;
    for i in 0..11 {
        commands.spawn(SpriteBundle {
            texture: road_tex.clone(),
            transform: Transform::from_xyz(64.0, y_top + 64.0 * i as f32, SPRITE_FLOOR_LAYER_Z),
            ..default()
        }).insert(RenderLayers::from_layers(CAMERA_LAYER_SPRITE));

        // Right side grass
        commands.spawn(SpriteBundle {
            texture: grass_tex.clone(),
            transform: Transform::from_xyz(96.0, y_top + 64.0 * i as f32, SPRITE_FLOOR_LAYER_Z),
            ..default()
        }).insert(RenderLayers::from_layers(CAMERA_LAYER_SPRITE));

        // Left side grass
        commands.spawn(SpriteBundle {
            texture: grass_tex.clone(),
            sprite: Sprite {
                flip_x: true,
                flip_y: false,
                ..default()
            },
            transform: Transform::from_xyz(32.0, y_top + 64.0 * i as f32, SPRITE_FLOOR_LAYER_Z),
            ..default()
        }).insert(RenderLayers::from_layers(CAMERA_LAYER_SPRITE));
    }

    let street_light_tex = asset_server.load("sample_art/street_light.png");
    let y_top = -64.0 * 5.0 + 32.0;
    for i in 0..5 {
        // Street-light, right side of road
        commands.spawn(SpriteBundle {
            texture: street_light_tex.clone(),
            transform: Transform::from_xyz(64.0 + 31.0, y_top + 128.0 * i as f32, SPRITE_OBJECT_LAYER_Z),
            ..default()
        }).insert(RenderLayers::from_layers(CAMERA_LAYER_SPRITE));

        // Street-light, left side of road
        commands.spawn(SpriteBundle {
            texture: street_light_tex.clone(),
            sprite: Sprite {
                flip_x: true,
                flip_y: false,
                ..default()
            },
            transform: Transform::from_xyz(31.0, y_top + 128.0 * i as f32, SPRITE_OBJECT_LAYER_Z),
            ..default()
        }).insert(RenderLayers::from_layers(CAMERA_LAYER_SPRITE));
    }

    let yellow_light_corona_tex = asset_server.load("sample_art/yellow_light_corona.png");
    let y_top = -64.0 * 5.0 + 32.0;
    for i in 0..5 {
        commands.spawn(SpriteBundle {
            texture: yellow_light_corona_tex.clone(),
            transform: Transform::from_xyz(64.0 + 25.0, y_top + 128.0 * i as f32, NORMAL_LIGHT_LAYER_Z),
            ..default()
        }).insert(RenderLayers::from_layers(CAMERA_LAYER_LIGHT));

        commands.spawn(SpriteBundle {
            texture: yellow_light_corona_tex.clone(),
            sprite: Sprite {
                flip_x: true,
                flip_y: false,
                ..default()
            },
            transform: Transform::from_xyz(38.0, y_top + 128.0 * i as f32, NORMAL_LIGHT_LAYER_Z),
            ..default()
        }).insert(RenderLayers::from_layers(CAMERA_LAYER_LIGHT));
    }
}

#[derive(Component)]
pub struct TruckEntity;

#[derive(Resource)]
pub struct TruckEntityRef {
    id: Entity
}

const TRUCK_START_Y: f32 = 64.0 * 5.0 - 77.0;

fn spawn_truck_entities(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let truck_id = commands.spawn(SpriteBundle {
        texture: asset_server.load("sample_art/truck_sprite.png"),
        transform: Transform::from_xyz(64.0, TRUCK_START_Y, SPRITE_OBJECT_LAYER_Z),
        ..default()
    })
        .insert(RenderLayers::from_layers(CAMERA_LAYER_SPRITE))
        .insert(TruckEntity)
        .id();

    commands.insert_resource(TruckEntityRef { id: truck_id });

    // This blocks street light from lighting up the top of the truck
    // It must have the same color as the ambient light
    commands.spawn(SpriteBundle {
        texture: asset_server.load("sample_art/truck_light_occluder.png"),
        transform: Transform::from_xyz(64.0, TRUCK_START_Y, OCCLUDER_LIGHT_LAYER_Z),
        ..default()
    })
        .insert(RenderLayers::from_layers(CAMERA_LAYER_LIGHT))
        .insert(TruckEntity);

    // Putting headlights on higher Z layer, so it is the dominant light
    commands.spawn(SpriteBundle {
        texture: asset_server.load("sample_art/double_light_cone.png"),
        transform: Transform::from_xyz(64.0, TRUCK_START_Y - 51.0, NORMAL_LIGHT_LAYER_Z + 0.1),
        ..default()
    })
        .insert(RenderLayers::from_layers(CAMERA_LAYER_LIGHT))
        .insert(TruckEntity);
    commands.spawn(SpriteBundle {
        texture: asset_server.load("sample_art/rear_lights.png"),
        transform: Transform::from_xyz(64.0, TRUCK_START_Y + 40.0 + 15.0, NORMAL_LIGHT_LAYER_Z),
        ..default()
    })
        .insert(RenderLayers::from_layers(CAMERA_LAYER_LIGHT))
        .insert(TruckEntity);
}

fn move_truck(
    mut truck_q: Query<&mut Transform, With<TruckEntity>>,
    truck_entity_ref: Res<TruckEntityRef>,
    time: Res<Time>,
) {
    let mut move_y: f32 = -32.0;
    let mut move_back = false;
    if let Ok(main_transform) = truck_q.get(truck_entity_ref.id) {
        if main_transform.translation.y < -310.0 {
            // Return truck to the top of the road
            move_y = 580.0;
            move_back = true;
        }
    }

    for mut transform in truck_q.iter_mut() {
        if move_back {
            transform.translation.y += move_y;
        } else {
            transform.translation.y += move_y * time.delta_seconds();
        }
    }
}

pub fn camera_movement(
    mut wheel: EventReader<MouseWheel>,
    mut query: Query<&mut OrthographicProjection, With<AnyNormalCamera>>,
) {
    for ev in wheel.read() {
        for mut ortho in query.iter_mut() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    if ev.y > 0.0 {
                        ortho.scale -= 0.1;
                    } else if ev.y < 0.0 {
                        ortho.scale += 0.1;
                    }
                }
                _ => {}
            }
            ortho.scale = ortho.scale.clamp(0.5, 1.5);
        }
    }
}