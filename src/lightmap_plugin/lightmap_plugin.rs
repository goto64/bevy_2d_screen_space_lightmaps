use bevy::core_pipeline::bloom::BloomSettings;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::camera::RenderTarget;
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_resource::{AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, Extent3d, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::texture::BevyDefault;
use bevy::render::view::RenderLayers;
use bevy::sprite::{Material2d, Material2dKey, Material2dPlugin, MaterialMesh2dBundle};
use bevy::window::PrimaryWindow;


pub struct LightmapPlugin;

impl Plugin for LightmapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BlendTexturesMaterial>::default());
        app.add_systems(Startup, (setup_post_processing_camera, setup_sprite_camera).chain());
        app.init_resource::<CameraTargets>();
    }
}

/// All normal sprites must be added to this camera layer
pub const CAMERA_LAYER_SPRITE: &[u8] = &[1];

/// All light sprites must be added to this camera layer
pub const CAMERA_LAYER_LIGHT: &[u8] = &[2];

#[derive(Component)]
pub struct SpriteCamera;

#[derive(Component)]
pub struct LightCamera;

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

#[derive(AsBindGroup, TypeUuid, TypePath, Asset, Debug, Clone)]
#[uuid = "a67d88f0-a69a-43ba-b4a7-00c0c9ee3332"]
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
        _layout: &MeshVertexBufferLayout,
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

        let sprite_image_handle: Handle<Image> = Handle::weak_from_u128(84562364042238462871);
        let light_image_handle: Handle<Image> = Handle::weak_from_u128(81297563682952991277);

        images.insert(sprite_image_handle.clone(), sprite_image);
        images.insert(light_image_handle.clone(), light_image);

        Self {
            sprite_target: sprite_image_handle,
            light_target: light_image_handle,
        }
    }
}

fn setup_sprite_camera(
    mut commands: Commands,
    camera_targets: Res<CameraTargets>,
) {
    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: true,
                    target: RenderTarget::Image(camera_targets.sprite_target.clone()),
                    ..Default::default()
                },
                ..Default::default()
            },
            Name::new("sprite_camera"),
        ))
        .insert(SpriteCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_SPRITE))
        .insert(UiCameraConfig {
            show_ui: false,
            ..default()
        });

    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: true,
                    target: RenderTarget::Image(camera_targets.light_target.clone()),
                    ..Default::default()
                },
                ..Default::default()
            },
            Name::new("light_camera"),
        ))
        .insert(LightCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_LIGHT))
        .insert(UiCameraConfig {
            show_ui: false,
            ..default()
        });
}

const POST_PROCESSING_QUAD: Handle<Mesh> = Handle::weak_from_u128(23467206864860343678);
const POST_PROCESSING_MATERIAL: Handle<BlendTexturesMaterial> = Handle::weak_from_u128(52374148673736462871);

fn setup_post_processing_camera(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    mut camera_targets: ResMut<CameraTargets>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BlendTexturesMaterial>>,
) {
    let Ok(window) = window.get_single() else { panic!("No window") };
    let primary_size = Vec2::new(
        (window.physical_width() as f64 / window.scale_factor()) as f32,
        (window.physical_height() as f64 / window.scale_factor()) as f32,
    );

    let quad =  Mesh::from(shape::Quad::new(Vec2::new(
        primary_size.x,
        primary_size.y,
    )));
    meshes.insert(POST_PROCESSING_QUAD.clone(), quad);

    *camera_targets = CameraTargets::create(&mut images, &primary_size);

    let material = BlendTexturesMaterial {
        texture1: camera_targets.sprite_target.clone(),
        texture2: camera_targets.light_target.clone(),
    };

    materials.insert(POST_PROCESSING_MATERIAL.clone(), material);

    let layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

    commands.spawn((
        PostProcessingQuad,
        MaterialMesh2dBundle {
            mesh: POST_PROCESSING_QUAD.clone().into(),
            material: POST_PROCESSING_MATERIAL.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                ..default()
            },
            ..default()
        },
        layer,
    ));

    // Camera that renders the final image for the screen
    commands.spawn((
        Name::new("post_processing_camera"),
        Camera2dBundle {
            camera: Camera {
                order: 1,
                hdr: true,
                ..default()
            },
            ..Camera2dBundle::default()
        },
        BloomSettings {
            intensity: 0.1,
            ..default()
        },
        layer
    ));
}