use bevy::{
    // render::{
    //     mesh::{MeshVertexBufferLayout, PrimitiveTopology},
    //     render_asset::RenderAssetUsages,
    //     render_resource::{
    //         AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
    //         SpecializedMeshPipelineError,
    //     },
    // },
    audio::{PlaybackMode, PlaybackSettings},
    // pbr::{MaterialPipeline, MaterialPipelineKey},
    // reflect::TypePath,
    core::FrameCount,
    prelude::*,
    window::{Cursor, CursorGrabMode, Window, WindowMode, WindowPlugin, WindowResolution},
};

use bevy_xpbd_3d::{math::*, prelude::*};

mod character;
use character::*;

mod cursor;
use cursor::*;

mod debugger;
use debugger::*;

/// Astraliminal's version.
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                cursor: Cursor {
                    visible: false,
                    grab_mode: CursorGrabMode::Locked,
                    ..default()
                },
                title: format!("Astralimimnal v{}", VERSION),
                name: Some("astraliminal.app".into()),
                resolution: WindowResolution::new(1024.0, 768.0).with_scale_factor_override(1.0),
                mode: WindowMode::BorderlessFullscreen,
                resizable: false,
                focused: true,
                // This will spawn an invisible window
                // The window will be made visible in the make_visible() system after 5 frames.
                // This is useful when you want to avoid the white window that shows up before
                // the GPU is ready to render the app.
                visible: false,
                ..default()
            }),
            ..default()
        }),
        PhysicsPlugins::default(),
        CharacterPlugin,
        DebuggerPlugin,
        CursorPlugin,
    ))
    .add_systems(Startup, (setup /*spawn_music*/,))
    .add_systems(Update, make_visible)
    .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    // Player
    let hitbox = PlayerHitbox::default();

    commands.spawn((
        // PbrBundle {
        //     mesh: meshes.add(Capsule3d::new(hitbox.radius, hitbox.length)),
        //     material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
        //     transform: Transform::from_translation(hitbox.position),
        //     ..default()
        // },
        CharacterControllerBundle::new(Collider::capsule(hitbox.length, hitbox.radius))
            .with_movement(30.0, 0.92, 4.0, (30.0 as Scalar).to_radians()),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        GravityScale(1.0),
        hitbox.clone(),
        Camera3dBundle {
            transform: Transform::from_translation(hitbox.position)
                .looking_at(hitbox.looking_at, Vec3::Y),
            ..Default::default()
        },
        // MassPropertiesBundle {
        //     mass: Mass(200.0),
        //     ..default()
        // },
        // PanOrbitCamera {
        //     focus: hitbox.position,
        //     radius: 0.0,
        //     ..Default::default()
        // },
    ));

    // A cube to move around
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::rgb(0.8, 0.1, 0.2)),
            transform: Transform::from_xyz(3.0, 10.0, 3.0),
            ..default()
        },
        // GravityScale(0.1666667),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Max),
        MassPropertiesBundle {
            mass: Mass(1.0),
            ..default()
        },
    ));

    // // A golf ball to move around
    // commands.spawn((
    //     // RigidBody::Dynamic,
    //     // Collider::cuboid(1.0, 1.0, 1.0),
    //     // PbrBundle {
    //     //     mesh: meshes.add(Cuboid::default()),
    //     //     // material: materials.add(Color::rgb(0.1, 0.8, 0.2)),
    //     //     transform: Transform::from_xyz(3.0, 2.0, 3.0),
    //     //     ..default()
    //     // },
    //     SceneBundle {
    //         scene: assets.load("models/GolfBall.glb#Scene0"),
    //         ..default()
    //     },
    //     AsyncSceneCollider::new(Some(ComputedCollider::ConvexHull)),
    //     RigidBody::Dynamic,
    //     // Transform::from_xyz(0.0, 1.0, 0.0),
    // ));

    // Environment (see `async_colliders` example for creating colliders from scenes)
    commands.spawn((
        SceneBundle {
            scene: assets.load("models/character_controller_demo.glb#Scene0"),
            // transform: Transform::from_rotation(Quat::from_rotation_y(-std::f32::consts::PI * 0.5)),
            // transform: Transform::from_rotation(Quat::from_rotation_y(0.0)),
            ..default()
        },
        AsyncSceneCollider::new(Some(ComputedCollider::ConvexHull)),
        RigidBody::Static,
    ));

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 2_000_000.0,
            range: 50.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 15.0, 0.0),
        ..default()
    });
}

fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    // The delay may be different for your app or system.
    if frames.0 == 5 {
        // At this point the gpu is ready to show the app so we can make the window visible.
        // Alternatively, you could toggle the visibility in Startup.
        // It will work, but it will have one white frame before it starts rendering
        window.single_mut().visible = true;
    }
}

fn _spawn_music(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(AudioBundle {
        source: asset_server.load("sound/Patrick De Arteaga/Su Turno.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Loop,
            ..default()
        },
        ..default()
    });
}
