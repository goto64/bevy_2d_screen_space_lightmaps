use bevy::asset::weak_handle;
use bevy::core_pipeline::bloom::Bloom;
use bevy::math::FloatOrd;
use bevy::render::camera::{ClearColorConfig, ImageRenderTarget};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::mesh::{MeshVertexBufferLayoutRef};
use bevy::render::render_resource::{AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, Extent3d, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
use bevy::sprite::{Material2d, Material2dKey, Material2dPlugin};
use bevy::window::{PrimaryWindow, WindowResized};

pub struct LightmapPlugin;

impl Plugin for LightmapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BlendTexturesMaterial>::default());
        app.add_systems(Startup, (setup_post_processing_camera, setup_sprite_camera).chain());
        app.add_systems(Update, on_resize_window);
        app.init_resource::<LightmapPluginSettings>();
        app.init_resource::<CameraTargets>();
    }
}

#[derive(Resource)]
pub struct LightmapPluginSettings {
    clear_color: ClearColorConfig,
    ambient_light: Color,
    bloom: Option<Bloom>,
}

impl Default for LightmapPluginSettings {
    fn default() -> Self {
        Self {
            clear_color: ClearColorConfig::Default,
            ambient_light: Color::srgb(0.3, 0.3, 0.3),
            bloom: None,
        }
    }
}

/// All normal sprites must be added to this camera layer
pub const CAMERA_LAYER_SPRITE: &[usize] = &[1];

/// All light sprites must be added to this camera layer
pub const CAMERA_LAYER_LIGHT: &[usize] = &[2];

/// Camera that has the final composite image
pub const CAMERA_FINAL_IMAGE: &[usize] = &[50];

#[derive(Component)]
pub struct SpriteCamera;

#[derive(Component)]
pub struct LightCamera;

#[derive(Component)]
pub struct AnyNormalCamera;

#[derive(Component)]
struct PostProcessingQuad;

const BLEND_ADD: BlendState = BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },

    alpha: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
};

#[derive(AsBindGroup, TypePath, Asset, Debug, Clone)]
struct BlendTexturesMaterial {
    #[texture(1)]
    #[sampler(2)]
    pub texture1: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub texture2: Handle<Image>,
}


impl Material2d for BlendTexturesMaterial {
    fn fragment_shader() -> ShaderRef {
        "lightmap_shader/blend_mult_textures.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BLEND_ADD);
            }
        }

        Ok(())
    }
}

#[derive(Resource, Default)]
struct CameraTargets {
    pub sprite_target: Handle<Image>,
    pub light_target: Handle<Image>,
}

impl CameraTargets {
    pub fn create(images: &mut Assets<Image>, sizes: &Vec2) -> Self {
        let target_size = Extent3d {
            width: sizes.x as u32,
            height: sizes.y as u32,
            ..default()
        };

        let mut sprite_image = Image {
            texture_descriptor: TextureDescriptor {
                label:           Some("target_sprite"),
                size:            target_size,
                dimension:       TextureDimension::D2,
                format:          TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count:    1,
                usage:           TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
                view_formats:    &[],
            },
            ..default()
        };
        let mut light_image = Image {
            texture_descriptor: TextureDescriptor {
                label:           Some("target_light"),
                size:            target_size,
                dimension:       TextureDimension::D2,
                format:          TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count:    1,
                usage:           TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
                view_formats:    &[],
            },
            ..default()
        };

        // Fill images data with zeroes.
        sprite_image.resize(target_size);
        light_image.resize(target_size);

        let sprite_image_handle: Handle<Image> = weak_handle!("4098187b-13f5-4531-a584-a2b4c7ad3e05");
        let light_image_handle: Handle<Image> = weak_handle!("13e36f32-8c1d-4566-be37-5a7da7cea2ca");

        images.insert(&sprite_image_handle, sprite_image);
        images.insert(&light_image_handle, light_image);

        Self {
            sprite_target: sprite_image_handle,
            light_target: light_image_handle,
        }
    }
}

fn setup_sprite_camera(
    mut commands: Commands,
    camera_targets: Res<CameraTargets>,
    lightmap_plugin_settings: Res<LightmapPluginSettings>,
) {
    commands
        .spawn((
            Camera2d,
            Camera {
                hdr: true,
                target: RenderTarget::Image(ImageRenderTarget {
                    handle: camera_targets.sprite_target.clone(),
                    scale_factor: FloatOrd(1.0),
                }),
                clear_color: lightmap_plugin_settings.clear_color.clone(),
                ..Default::default()
            },
            Name::new("sprite_camera"),
        ))
        .insert(SpriteCamera)
        .insert(AnyNormalCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_SPRITE));

    commands
        .spawn((
            Camera2d,
            Camera {
                hdr: true,
                target: RenderTarget::Image(ImageRenderTarget {
                    handle: camera_targets.light_target.clone(),
                    scale_factor: FloatOrd(1.0),
                }),
                clear_color: ClearColorConfig::Custom(lightmap_plugin_settings.ambient_light),
                ..Default::default()
            },
            Name::new("light_camera"),
        ))
        .insert(LightCamera)
        .insert(AnyNormalCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_LIGHT));
}

const POST_PROCESSING_QUAD: Handle<Mesh> = weak_handle!("36f3e6a8-0837-4544-9ca9-05dfd13ac6b8");
const POST_PROCESSING_MATERIAL: Handle<BlendTexturesMaterial> = weak_handle!("f9e193d4-d737-433d-820e-9e42b1c70af9");

fn setup_post_processing_camera(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    lightmap_plugin_settings: Res<LightmapPluginSettings>,
    mut camera_targets: ResMut<CameraTargets>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BlendTexturesMaterial>>,
) {
    let Ok(window) = window.single() else { panic!("No window") };
    let primary_size = Vec2::new(
        (window.physical_width() as f32 / window.scale_factor()) as f32,
        (window.physical_height() as f32 / window.scale_factor()) as f32,
    );

    let quad =  Mesh::from(Rectangle::new(primary_size.x, primary_size.y));
    meshes.insert(&POST_PROCESSING_QUAD, quad);

    *camera_targets = CameraTargets::create(&mut images, &primary_size);

    let material = BlendTexturesMaterial {
        texture1: camera_targets.sprite_target.clone(),
        texture2: camera_targets.light_target.clone(),
    };

    materials.insert(&POST_PROCESSING_MATERIAL, material);

    let layer = RenderLayers::from_layers(CAMERA_FINAL_IMAGE);

    commands.spawn((
        PostProcessingQuad,
        MeshMaterial2d(POST_PROCESSING_MATERIAL.clone()),
        Mesh2d(POST_PROCESSING_QUAD.clone().into()),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.5)),
        layer.clone(),
    ));

    // Camera that renders the final image for the screen
    let camera_id = commands.spawn((
        Name::new("post_processing_camera"),
        (
            Camera2d,
            Camera {
                order: 1,
                hdr: true,
                ..default()
            },
        ),
        layer
    )).id();

    if lightmap_plugin_settings.bloom.is_some() {
        commands.entity(camera_id).insert(lightmap_plugin_settings.bloom.clone().unwrap());
    }
}

fn on_resize_window(
    mut resize_reader: EventReader<WindowResized>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut camera_targets: ResMut<CameraTargets>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<BlendTexturesMaterial>>,
) {
    for ev in resize_reader.read() {
        let Ok(window) = window.single() else { panic!("No window") };
        let primary_size = Vec2::new(
            (ev.width / window.scale_factor()) as f32,
            (ev.height / window.scale_factor()) as f32,
        );

        let quad =  Mesh::from(Rectangle::new(primary_size.x, primary_size.y));
        meshes.insert(&POST_PROCESSING_QUAD, quad);

        *camera_targets = CameraTargets::create(&mut images, &primary_size);

        let material = BlendTexturesMaterial {
            texture1: camera_targets.sprite_target.clone(),
            texture2: camera_targets.light_target.clone(),
        };

        materials.insert(&POST_PROCESSING_MATERIAL, material);
    }
}
