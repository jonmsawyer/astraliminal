mod character;
mod cursor;
mod debugger;

use bevy::{
    core::FrameCount,
    prelude::*,
    window::{Cursor, CursorGrabMode, Window, WindowMode, WindowPlugin, WindowResolution},
};

use bevy_xpbd_3d::{math::*, prelude::*};

use character::*;
use cursor::*;
use debugger::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                cursor: Cursor {
                    visible: false,
                    grab_mode: CursorGrabMode::Locked,
                    ..Default::default()
                },
                title: "Astraliminal".to_string(),
                name: Some("astraliminal.app".into()),
                resolution: WindowResolution::new(1024., 768.).with_scale_factor_override(1.0),
                mode: WindowMode::BorderlessFullscreen,
                resizable: false,
                focused: true,
                // This will spawn an invisible window
                // The window will be made visible in the make_visible() system after 5 frames.
                // This is useful when you want to avoid the white window that shows up before
                // the GPU is ready to render the app.
                visible: false,
                ..Default::default()
            }),
            ..Default::default()
        }),
        PhysicsPlugins::default(),
        CharacterPlugin,
        DebuggerPlugin,
        CursorPlugin,
    ))
    .add_systems(Startup, setup)
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
            .with_movement(30.0, 0.92, 7.0, (30.0 as Scalar).to_radians()),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        GravityScale(1.0),
        hitbox.clone(),
        Camera3dBundle {
            transform: Transform::from_translation(hitbox.position)
                .looking_at(hitbox.looking_at, Vec3::Y),
            ..Default::default()
        },
        PanOrbitCamera {
            focus: hitbox.position,
            radius: 0.0,
            ..Default::default()
        },
    ));

    // A cube to move around
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(3.0, 2.0, 3.0),
            ..default()
        },
    ));

    // Environment (see `async_colliders` example for creating colliders from scenes)
    commands.spawn((
        SceneBundle {
            scene: assets.load("models/character_controller_demo.glb#Scene0"),
            transform: Transform::from_rotation(Quat::from_rotation_y(-std::f32::consts::PI * 0.5)),
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

    // Camera
    // let translation = Vec3::new(-2.0, 2.5, 5.0);
    // let radius = translation.length();
    // commands.spawn((
    //     Camera3dBundle {
    //         transform: Transform::from_xyz(-7.0, 9.5, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    //         ..default()
    //     },
    //     PanOrbitCamera {
    //         radius,
    //         ..Default::default()
    //     },
    // ));
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

// use bevy::{input::keyboard::{self, KeyboardInput}, prelude::*};
// use bevy_flycam::prelude::*;
// use bevy_xpbd_3d::prelude::*;

// mod plugin;

// #[derive(Component)]
// struct Player;

// #[derive(Component)]
// struct Cube;

// #[bevy_main]
// fn main() {
//     App::new()
//         .add_plugins((
//             DefaultPlugins,
//             // PlayerPlugin,
//             NoCameraPlayerPlugin,
//             PhysicsPlugins::default(),
//         ))
//         .insert_resource(MovementSettings {
//             sensitivity: 0.00015, // default: 0.00012
//             speed: 12.0, // default: 12.0
//         })
//         .insert_resource(KeyBindings {
//             move_forward: KeyCode::KeyW,
//             move_backward: KeyCode::KeyS,
//             move_left: KeyCode::KeyA,
//             move_right: KeyCode::KeyD,
//             move_ascend: KeyCode::Space,
//             move_descend: KeyCode::ShiftLeft,
//             toggle_grab_cursor: KeyCode::Escape,
//         })
//         .add_systems(Startup, setup)
//         .add_systems(Update, keyboard_input)
//         .run();
// }

// fn setup(
//     // mut commands: Commands,
//     // mut _materials: ResMut<Assets<ColorMaterial>>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     _asset_server: Res<AssetServer>,
// ) {
//     // Plane
//     commands.spawn((
//         RigidBody::Static,
//         Collider::cuboid(8.0, 0.002, 8.0),
//         PbrBundle {
//             mesh: meshes.add(Plane3d::default().mesh().size(8.0, 8.0)),
//             material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
//             ..default()
//         },
//     ));

//     // Cube
//     commands.spawn((
//         RigidBody::Dynamic,
//         AngularVelocity(Vec3::new(2.5, 3.4, 1.6)),
//         Collider::cuboid(1.0, 1.0, 1.0),
//         PbrBundle {
//             mesh: meshes.add(Cuboid::default()),
//             material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
//             transform: Transform::from_xyz(0.0, 4.0, 0.0),
//             ..default()
//         },
//         Cube,
//     ));

//     // Light
//     commands.spawn(PointLightBundle {
//         point_light: PointLight {
//             intensity: 2_000_000.0,
//             shadows_enabled: true,
//             ..default()
//         },
//         transform: Transform::from_xyz(4.0, 8.0, 4.0),
//         ..default()
//     });

//     // Camera
//     commands.spawn((
//         Camera3dBundle {
//             transform: Transform::from_xyz(-4.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
//             ..default()
//         },
//         FlyCam,
//     ));

//     // // circular base
//     // commands.spawn(PbrBundle {
//     //     mesh: meshes.add(Circle::new(4.0)),
//     //     material: materials.add(Color::WHITE),
//     //     transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
//     //     ..default()
//     // });
//     // // cube
//     // commands.spawn(PbrBundle {
//     //     mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
//     //     material: materials.add(Color::rgb_u8(124, 144, 255)),
//     //     transform: Transform::from_xyz(0.0, 0.5, 0.0),
//     //     ..default()
//     // });
//     // // from asset
//     // commands.spawn(SceneBundle  {
//     //     scene: asset_server.load("GolfBall.glb#Scene0"),
//     //     // material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
//     //     transform: Transform::from_xyz(5.0, 0.5, 0.0),
//     //     ..Default::default()
//     // });
//     // // from asset
//     // commands.spawn(SceneBundle  {
//     //     scene: asset_server.load("alien.glb#Scene0"),
//     //     // material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
//     //     transform: Transform::from_xyz(-5.0, 2.5, 0.0),
//     //     ..Default::default()
//     // });
//     // // from asset
//     // commands.spawn(SceneBundle  {
//     //     scene: asset_server.load("cakeBirthday.glb#Scene0"),
//     //     // material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
//     //     transform: Transform::from_xyz(0.0, 0.5, 5.0),
//     //     ..Default::default()
//     // });
//     // // light
//     // commands.spawn(PointLightBundle {
//     //     point_light: PointLight {
//     //         shadows_enabled: true,
//     //         ..default()
//     //     },
//     //     transform: Transform::from_xyz(4.0, 12.0, 4.0),
//     //     ..default()
//     // });
//     // // camera
//     // commands.spawn((
//     //     Camera3dBundle {
//     //         transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
//     //         ..default()
//     //     },
//     //     FlyCam,
//     // ));

//     // commands
//     //     .insert_resource(Player)
//     //     .insert(Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)));
// }

// fn keyboard_input(
//     mut input: EventReader<KeyboardInput>,
//     mut query: Query<Cube>,
//     _time: Res<Time>,
// ) {
//     for key_input in input.read() {
//         info!("{:?}", key_input);
//         if key_input.key_code == KeyCode::F5 {
//             refresh_scene(&mut transform);
//         }
//     }
// }

// fn refresh_scene(mut transform: Transform) {
//     trasform = Transform::from_xyz(0.0, 4.0, 0.0);
// }
