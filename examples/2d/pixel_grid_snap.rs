//! Shows how to create graphics that snap to the pixel grid by rendering to a texture in 2D

use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};
use bevy_internal::sprite::MaterialMesh2dBundle;
use bevy_internal::window::WindowResized;

const RES_WIDTH: u32 = 160;
const RES_HEIGHT: u32 = 90;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(Msaa::Off)
        .add_startup_systems((setup_camera, setup_sprite, setup_mesh))
        .add_systems((transform_drawables, fit_canvas))
        .run();
}

#[derive(Component)]
struct Canvas;

#[derive(Component)]
struct InGameCamera;

#[derive(Component)]
struct OuterCamera;

#[derive(Component)]
struct Drawable;

fn setup_sprite(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("textures/rpg/chars/sensei/sensei.png");
    // the sample sprite that will be rendered to the canvas
    commands.spawn((
        SpriteBundle {
            texture: texture.clone(),
            transform: Transform::from_xyz(-40., 20., 0.),
            ..default()
        },
        Drawable,
        RenderLayers::layer(1),
    ));

    // same, but skips the pixel-perfect rendering
    commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform::from_xyz(-40., -20., 0.),
            ..default()
        },
        Drawable,
    ));
}

fn setup_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
            transform: Transform::from_xyz(40., 0., 0.).with_scale(Vec3::splat(32.)),
            material: materials.add(ColorMaterial::from(Color::BLACK)),
            ..default()
        },
        RenderLayers::layer(1),
        Drawable,
    ));
}

fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let size = Extent3d {
        width: RES_WIDTH,
        height: RES_HEIGHT,
        ..default()
    };

    // this Image serves as a canvas representing the low-resolution game screen
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    canvas.resize(size);

    let image_handle = images.add(canvas);

    // this camera renders whatever is on Render layer 1 to the canvas
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                // render before the "main pass" camera
                order: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(1),
        InGameCamera,
    ));

    // this places the canvas to the "outer world"
    commands.spawn((
        SpriteBundle {
            texture: image_handle,
            ..default()
        },
        Canvas,
    ));

    // the "outer" camera renders the objects that are in the "outer world"
    // in this example, the canvas and one copy of the sample sprite are in the "outer world"
    commands.spawn((Camera2dBundle::default(), OuterCamera));
}

/// transform drawables to demonstrate grid snapping
fn transform_drawables(time: Res<Time>, mut query: Query<&mut Transform, With<Drawable>>) {
    for mut transform in &mut query {
        let dt = time.delta_seconds();
        transform.rotate_z(dt * PI / 2.);
    }
}

/// scales camera projection to fit the window (integer multiples only)
fn fit_canvas(
    mut resize_event: EventReader<WindowResized>,
    mut q: Query<&mut OrthographicProjection, With<OuterCamera>>,
) {
    for event in resize_event.iter() {
        let h_scale = event.width / RES_WIDTH as f32;
        let v_scale = event.height / RES_HEIGHT as f32;
        let mut projection = q.single_mut();
        projection.scale = 1. / h_scale.min(v_scale).round();
    }
}
