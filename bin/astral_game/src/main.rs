//! Astraliminal game.

use std::fmt::{Display, Formatter, Result as FmtResult};

use astral_core::prelude::*;

// Rotation speed in radians per frame.
const ROTATION_SPEED: f32 = 0.005;

static _STOP_ROTATION_HELP_TEXT: &str = "Press Enter to stop rotation";
static _START_ROTATION_HELP_TEXT: &str = "Press Enter to start rotation";
static _REFLECTION_MODE_HELP_TEXT: &str = "Press Space to switch reflection mode";

// The mode the application is in.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Resource)]
struct AppStatus {
    // Which environment maps the user has requested to display.
    reflection_mode: ReflectionMode,
    // Whether the user has requested the scene to rotate.
    rotating: bool,
}

impl AppStatus {
    // Constructs the help text at the bottom of the screen based on the
    // application status.
    fn _create_text(&self, asset_server: &AssetServer) -> Text {
        let rotation_help_text = if self.rotating {
            _STOP_ROTATION_HELP_TEXT
        } else {
            _START_ROTATION_HELP_TEXT
        };

        Text::from_section(
            format!(
                "{}\n{}\n{}",
                self.reflection_mode, rotation_help_text, _REFLECTION_MODE_HELP_TEXT
            ),
            TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 24.0,
                color: Color::ANTIQUE_WHITE,
            },
        )
    }
}

impl Default for AppStatus {
    fn default() -> Self {
        Self {
            reflection_mode: ReflectionMode::ReflectionProbe,
            rotating: false,
        }
    }
}

// Which environment maps the user has requested to display.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ReflectionMode {
    // No environment maps are shown.
    None = 0,
    // Only a world environment map is shown.
    EnvironmentMap = 1,
    // Both a world environment map and a reflection probe are present. The
    // reflection probe is shown in the sphere.
    #[default]
    ReflectionProbe = 2,
}

impl TryFrom<u32> for ReflectionMode {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ReflectionMode::None),
            1 => Ok(ReflectionMode::EnvironmentMap),
            2 => Ok(ReflectionMode::ReflectionProbe),
            _ => Err(()),
        }
    }
}

impl Display for ReflectionMode {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let text = match *self {
            ReflectionMode::None => "No reflections",
            ReflectionMode::EnvironmentMap => "Environment map",
            ReflectionMode::ReflectionProbe => "Reflection probe",
        };
        formatter.write_str(text)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Resource)]
struct Cubemap {
    is_loaded: bool,
    // index: usize,
    image_handle: Handle<Image>,
}

// The various reflection maps.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Resource)]
struct Cubemaps {
    // The blurry diffuse cubemap. This is used for both the world environment
    // map and the reflection probe. (In reality you wouldn't do this, but this
    // reduces complexity of this example a bit.)
    diffuse: Handle<Image>,

    // The specular cubemap that reflects the world, but not the cubes.
    specular_environment_map: Handle<Image>,

    // The specular cubemap that reflects both the world and the cubes.
    specular_reflection_probe: Handle<Image>,

    // The skybox cubemap image. This is almost the same as
    // `specular_environment_map`.
    skybox: Handle<Image>,
}

// Loads the cubemaps from the assets directory.
impl FromWorld for Cubemaps {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        // Just use the specular map for the skybox since it's not too blurry.
        // In reality you wouldn't do this--you'd use a real skybox texture--but
        // reusing the textures like this saves space in the Bevy repository.
        let specular_map = asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2");

        Cubemaps {
            diffuse: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_reflection_probe: asset_server
                .load("environment_maps/cubes_reflection_probe_specular_rgb9e5_zstd.ktx2"),
            specular_environment_map: specular_map.clone(),
            skybox: specular_map,
        }
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        AstraliminalWindowPlugin,
        PhysicsPlugins::default(),
        CharacterPlugin,
        DebuggerPlugin,
        CursorPlugin,
    ))
    .init_resource::<AppStatus>()
    .init_resource::<Cubemaps>()
    .add_systems(PreUpdate, add_environment_map_to_camera)
    .add_systems(Startup, (setup /*spawn_music*/,))
    .add_systems(Update, asset_loaded)
    .add_systems(Update, change_reflection_type)
    .add_systems(Update, toggle_rotation)
    .add_systems(
        Update,
        rotate_camera
            .after(toggle_rotation)
            .after(change_reflection_type),
    )
    // .add_systems(Update, update_text.after(rotate_camera))
    .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    _app_status: Res<AppStatus>,
    cubemaps: Res<Cubemaps>,
) {
    spawn_environment(&mut commands, &asset_server);
    spawn_cube(&mut commands, &mut meshes, &mut materials);
    spawn_sphere(&mut commands, &mut meshes, &mut materials);
    spawn_golfball(&mut commands, &asset_server);
    spawn_point_light(&mut commands);
    spawn_ambient_light(&mut commands);
    spawn_reflection_probe(&mut commands, &asset_server, &cubemaps);
    spawn_character(&mut commands, &asset_server);
}

// Spawns the player character.
fn spawn_character(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let hitbox: CharacterHitbox = CharacterHitbox::default();
    commands.spawn((
        // PbrBundle {
        //     mesh: meshes.add(Capsule3d::new(hitbox.radius, hitbox.length)),
        //     material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
        //     transform: Transform::from_translation(hitbox.position),
        //     ..default()
        // },
        CharacterControllerBundle::new(Collider::capsule(hitbox.length, hitbox.radius))
            .with_movement(30.0, 0.92, 4.0, (30.0 as Scalar).to_radians()),
        camera_bundle(&hitbox),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        GravityScale(1.0),
        hitbox,
        Skybox {
            image: asset_server.load("images/Ryfjallet_cubemap_bc7.ktx2"),
            brightness: 1000.0,
        },
        MassPropertiesBundle {
            mass: Mass(100.0),
            ..default()
        },
        // PanOrbitCamera {
        //     focus: hitbox.position,
        //     radius: 0.0,
        //     ..Default::default()
        // },
    ));
}

// Spawns the camera.
fn camera_bundle(hitbox: &CharacterHitbox) -> Camera3dBundle {
    Camera3dBundle {
        camera: Camera {
            hdr: true,
            ..default()
        },
        transform: hitbox.transform(),
        ..Default::default()
    }
}

// Creates the sphere mesh and spawns it.
fn spawn_sphere(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let radius: f32 = 1.0;
    let ico_subdivisions: usize = 7;
    // Create a sphere mesh.
    let sphere_mesh = meshes.add(Sphere::new(radius).mesh().ico(ico_subdivisions).unwrap());

    // Create a sphere.
    commands.spawn((
        RigidBody::Dynamic,
        Collider::sphere(radius),
        PbrBundle {
            mesh: sphere_mesh.clone(),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("#ffd891").unwrap(),
                metallic: 1.0,
                perceptual_roughness: 0.0,
                ..StandardMaterial::default()
            }),
            transform: Transform::from_translation(Vec3::new(6.0, 6.0, 0.0).into()),
            ..PbrBundle::default()
        },
        // GravityScale(0.1666667),
        Friction {
            dynamic_coefficient: 0.5,
            static_coefficient: 0.9,
            combine_rule: CoefficientCombine::Average,
        },
        MassPropertiesBundle {
            mass: Mass(1.0),
            ..default()
        },
    ));
}

// Spawns a cube.
fn spawn_cube(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
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
        Friction {
            dynamic_coefficient: 0.5,
            static_coefficient: 0.9,
            combine_rule: CoefficientCombine::Average,
        },
        MassPropertiesBundle {
            mass: Mass(1.0),
            ..default()
        },
    ));
}

// Spawns a golf ball.
fn spawn_golfball(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let _radius: f32 = 1.0;

    // Environment (see `async_colliders` example for creating colliders from scenes)
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("models/GolfBall.glb#Scene0"),
            transform: Transform::from_xyz(3.0, 2.0, 3.0),
            ..SceneBundle::default()
        },
        AsyncSceneCollider::new(Some(ComputedCollider::ConvexHull)),
        RigidBody::Dynamic,
        // GravityScale(0.1666667),
        Friction {
            dynamic_coefficient: 0.5,
            static_coefficient: 0.9,
            combine_rule: CoefficientCombine::Average,
        },
        MassPropertiesBundle {
            mass: Mass(2.0),
            ..default()
        },
    ));
}

// Spawns the reflection probe.
fn spawn_reflection_probe(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    cubemaps: &Cubemaps,
) {
    commands.insert_resource(Cubemap {
        is_loaded: false,
        image_handle: asset_server.load("images/Ryfjallet_cubemap_bc7.ktx2"),
    });

    commands.spawn(ReflectionProbeBundle {
        spatial: SpatialBundle {
            // 2.0 because the sphere's radius is 1.0 and we want to fully enclose it.
            transform: Transform::from_scale(Vec3::splat(2.0)),
            ..SpatialBundle::default()
        },
        light_probe: LightProbe,
        environment_map: EnvironmentMapLight {
            diffuse_map: cubemaps.diffuse.clone(),
            specular_map: cubemaps.specular_reflection_probe.clone(),
            intensity: 5000.0,
        },
    });
}

fn spawn_environment(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    // Environment (see `async_colliders` example for creating colliders from scenes)
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("models/character_controller_demo2.glb#Scene0"),
            // transform: Transform::from_rotation(Quat::from_rotation_y(-std::f32::consts::PI * 0.5)),
            // transform: Transform::from_rotation(Quat::from_rotation_y(0.0)),
            ..SceneBundle::default()
        },
        AsyncSceneCollider::new(Some(ComputedCollider::ConvexHull)),
        RigidBody::Static,
    ));
}

fn spawn_point_light(commands: &mut Commands) {
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

fn spawn_ambient_light(commands: &mut Commands) {
    // Ambient light
    // NOTE: The ambient light is used to scale how bright the environment map is so with a bright
    // environment map, use an appropriate color and brightness to match
    commands.insert_resource(AmbientLight {
        color: Color::rgb_u8(210, 220, 240),
        brightness: 1000.0,
    });
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

fn asset_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if !cubemap.is_loaded && asset_server.load_state(&cubemap.image_handle) == LoadState::Loaded {
        let image = images.get_mut(&cubemap.image_handle).unwrap();
        // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
        // so they appear as one texture. The following code reconfigures the texture as necessary.
        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array(image.height() / image.width());
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        for mut skybox in &mut skyboxes {
            skybox.image = cubemap.image_handle.clone();
        }

        cubemap.is_loaded = true;
    }
}

// Adds a world environment map to the camera. This separate system is needed because the camera is
// managed by the scene spawner, as it's part of the glTF file with the cubes, so we have to add
// the environment map after the fact.
fn add_environment_map_to_camera(
    mut commands: Commands,
    query: Query<Entity, Added<Camera3d>>,
    cubemaps: Res<Cubemaps>,
) {
    for camera_entity in query.iter() {
        commands
            .entity(camera_entity)
            .insert(create_camera_environment_map_light(&cubemaps))
            .insert(Skybox {
                image: cubemaps.skybox.clone(),
                brightness: 5000.0,
            });
    }
}

// A system that handles switching between different reflection modes.
fn change_reflection_type(
    mut commands: Commands,
    light_probe_query: Query<Entity, With<LightProbe>>,
    camera_query: Query<Entity, With<Camera3d>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut app_status: ResMut<AppStatus>,
    cubemaps: Res<Cubemaps>,
    asset_server: Res<AssetServer>,
) {
    // Only do anything if space was pressed.
    if !keyboard.just_pressed(KeyCode::F6) {
        return;
    }

    // Switch reflection mode.
    app_status.reflection_mode =
        ReflectionMode::try_from((app_status.reflection_mode as u32 + 1) % 3).unwrap();

    // Add or remove the light probe.
    for light_probe in light_probe_query.iter() {
        commands.entity(light_probe).despawn();
    }
    match app_status.reflection_mode {
        ReflectionMode::None | ReflectionMode::EnvironmentMap => {}
        ReflectionMode::ReflectionProbe => {
            spawn_reflection_probe(&mut commands, &asset_server, &cubemaps)
        }
    }

    // Add or remove the environment map from the camera.
    for camera in camera_query.iter() {
        match app_status.reflection_mode {
            ReflectionMode::None => {
                commands.entity(camera).remove::<EnvironmentMapLight>();
            }
            ReflectionMode::EnvironmentMap | ReflectionMode::ReflectionProbe => {
                commands
                    .entity(camera)
                    .insert(create_camera_environment_map_light(&cubemaps));
            }
        }
    }
}

// A system that handles enabling and disabling rotation.
fn toggle_rotation(keyboard: Res<ButtonInput<KeyCode>>, mut app_status: ResMut<AppStatus>) {
    if keyboard.just_pressed(KeyCode::F7) {
        app_status.rotating = !app_status.rotating;
    }
}

// A system that updates the help text.
fn _update_text(
    mut text_query: Query<&mut Text>,
    app_status: Res<AppStatus>,
    asset_server: Res<AssetServer>,
) {
    for mut text in text_query.iter_mut() {
        *text = app_status._create_text(&asset_server);
    }
}

// Creates the world environment map light, used as a fallback if no reflection
// probe is applicable to a mesh.
fn create_camera_environment_map_light(cubemaps: &Cubemaps) -> EnvironmentMapLight {
    EnvironmentMapLight {
        diffuse_map: cubemaps.diffuse.clone(),
        specular_map: cubemaps.specular_environment_map.clone(),
        intensity: 5000.0,
    }
}

// Rotates the camera a bit every frame.
fn rotate_camera(
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    app_status: Res<AppStatus>,
) {
    if !app_status.rotating {
        return;
    }

    for mut transform in camera_query.iter_mut() {
        transform.translation = Vec2::from_angle(ROTATION_SPEED)
            .rotate(transform.translation.xz())
            .extend(transform.translation.y)
            .xzy();
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
