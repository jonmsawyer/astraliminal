use bevy::input::mouse::*;
use bevy::{ecs::query::Has, app::AppExit, prelude::*};
use bevy::window::PrimaryWindow;
use bevy_xpbd_3d::{math::*, prelude::*};

#[derive(Debug, Clone, Component)]
pub struct PlayerHitbox {
    pub length: f32,
    pub radius: f32,
    pub eye: f32,
    pub position: Vec3,
    pub looking_at: Vec3,
}

impl Default for PlayerHitbox {
    fn default() -> PlayerHitbox {
        let eye: f32 = 1.6256;
        PlayerHitbox {
            length: 1.778,
            radius: 0.762,
            eye,
            position: Vec3::new(-2.0, eye, 5.0),
            looking_at: Vec3::new(0.0, eye, 0.0),
        }
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct GameCursor {}

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MovementAction>()
            .add_systems(
                Update,
                (
                    keyboard_input,
                    gamepad_input,
                    update_grounded,
                    movement,
                    apply_movement_damping,
                ).chain(),
            )
            // .add_systems(Startup, spawn_camera)
            .add_systems(Update, (
                move_camera,
                // pan_orbit_camera,
            ));
        app.add_systems(Startup, setup_cursor);
            // .add_systems(Update, move_cursor);
        #[cfg(target_os = "windows")]
        app.add_systems(Update, cursor_recenter);
    }
}

/// An event sent for a movement input action.
#[derive(Debug, Event)]
pub enum MovementAction {
    Move(Vector2),
    // Camera(Vector2),
    Jump,
}

/// A marker component indicating that an entity is using a character controller.
#[derive(Debug, Component)]
pub struct CharacterController;

/// A marker component indicating that an entity is on the ground.
#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;
/// The acceleration used for character movement.
#[derive(Debug, Component)]
pub struct MovementAcceleration(Scalar);

/// The damping factor used for slowing down movement.
#[derive(Debug, Component)]
pub struct MovementDampingFactor(Scalar);

/// The strength of a jump.
#[derive(Debug, Component)]
pub struct JumpImpulse(Scalar);

/// The maximum angle a slope can have for a character controller
/// to be able to climb and jump. If the slope is steeper than this angle,
/// the character will slide down.
#[derive(Debug, Component)]
pub struct MaxSlopeAngle(Scalar);

/// A bundle that contains the components needed for a basic
/// kinematic character controller.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    rigid_body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    locked_axes: LockedAxes,
    movement: MovementBundle,
}

/// A bundle that contains components for character movement.
#[derive(Debug, Bundle)]
pub struct MovementBundle {
    acceleration: MovementAcceleration,
    damping: MovementDampingFactor,
    jump_impulse: JumpImpulse,
    max_slope_angle: MaxSlopeAngle,
}

impl MovementBundle {
    pub const fn new(
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        Self {
            acceleration: MovementAcceleration(acceleration),
            damping: MovementDampingFactor(damping),
            jump_impulse: JumpImpulse(jump_impulse),
            max_slope_angle: MaxSlopeAngle(max_slope_angle),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(30.0, 0.9, 7.0, PI * 0.45)
    }
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider) -> Self {
        // Create shape caster as a slightly smaller version of collider
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vector::ONE * 0.99, 10);

        Self {
            character_controller: CharacterController,
            rigid_body: RigidBody::Dynamic,
            collider,
            ground_caster: ShapeCaster::new(
                caster_shape,
                Vector::ZERO,
                Quaternion::default(),
                Direction3d::NEG_Y,
            )
            .with_max_time_of_impact(0.2),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            movement: MovementBundle::default(),
        }
    }

    pub fn with_movement(
        mut self,
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        self.movement = MovementBundle::new(acceleration, damping, jump_impulse, max_slope_angle);
        self
    }
}

/// Sends [`MovementAction`] events based on keyboard input.
fn keyboard_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    let quit = keyboard_input.any_pressed([KeyCode::Escape]);

    let horizontal = right as i8 - left as i8;
    let vertical = up as i8 - down as i8;
    let direction = Vector2::new(horizontal as Scalar, vertical as Scalar).clamp_length_max(1.0);

    if direction != Vector2::ZERO {
        movement_event_writer.send(MovementAction::Move(direction));
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_event_writer.send(MovementAction::Jump);
    }

    // movement_event_writer.send(MovementAction::Camera(direction));

    if quit {
        app_exit_events.send(AppExit);
    }
}

/// Sends [`MovementAction`] events based on gamepad input.
fn gamepad_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    gamepads: Res<Gamepads>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<ButtonInput<GamepadButton>>,
) {
    for gamepad in gamepads.iter() {
        let axis_lx = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickX,
        };
        let axis_ly = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickY,
        };

        if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
            movement_event_writer.send(MovementAction::Move(
                Vector2::new(x as Scalar, y as Scalar).clamp_length_max(1.0),
            ));
        }

        let jump_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::South,
        };

        if buttons.just_pressed(jump_button) {
            movement_event_writer.send(MovementAction::Jump);
        }
    }
}

/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>),
        With<CharacterController>,
    >,
) {
    for (entity, hits, rotation, max_slope_angle) in &mut query {
        // The character is grounded if the shape caster has a hit with a normal
        // that isn't too steep.
        let is_grounded = hits.iter().any(|hit| {
            if let Some(angle) = max_slope_angle {
                rotation.rotate(-hit.normal2).angle_between(Vector::Y).abs() <= angle.0
            } else {
                true
            }
        });

        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

/// Responds to [`MovementAction`] events and moves character controllers accordingly.
fn movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut controllers: Query<(
        &MovementAcceleration,
        &JumpImpulse,
        &mut LinearVelocity,
        &mut PlayerHitbox,
        Has<Grounded>,
    )>,
    mut _windows: Query<&mut Window>,
    mut _camera: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_seconds_f64().adjust_precision();

    for event in movement_event_reader.read() {
        for (
            movement_acceleration,
            jump_impulse,
            mut linear_velocity,
            mut _hitbox,
            is_grounded,
        ) in &mut controllers
        {
            match event {
                MovementAction::Move(direction) => {
                    linear_velocity.x += direction.x * movement_acceleration.0 * delta_time;
                    linear_velocity.z -= direction.y * movement_acceleration.0 * delta_time;
                }
                MovementAction::Jump => {
                    if is_grounded {
                        linear_velocity.y = jump_impulse.0;
                    }
                }
                // MovementAction::Camera(direction) => {
                //     let acceleration = movement_acceleration.0;
                //     let x = direction.x * acceleration * delta_time;
                //     let _y: f32 = 0.0;
                //     let z = -direction.y * acceleration * delta_time;
                //     info!("MovementAction::Camera event.");
                //     info!(" > direction={:?}, x={}, z={}", direction, x, z);
                //     let mut pan = Vec2::new(x, z);
                //     for (mut pan_orbit, mut transform, projection) in camera.iter_mut() {
                //         info!(" > pan_orbit={:?}", pan_orbit);
                //         info!(" > transform={:?}", transform);
                //         info!(" > projection={:?}", projection);
                //         info!(" > movement_acceleration={:?}", movement_acceleration);
                //         // make panning distance independent of resolution and FOV,
                //         let window = get_primary_window_size(&mut windows);
                //         if let Projection::Perspective(projection) = projection {
                //             pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window * acceleration * acceleration;
                //         }
                //         // translate by local axes
                //         let new_x = transform.rotation * Vec3::X * pan.x;
                //         let new_z = transform.rotation * Vec3::Z * pan.y;
                //         // make panning proportional to distance away from focus point
                //         let translation = (new_x + new_z) * pan_orbit.radius;
                //         pan_orbit.focus += translation;
                //         // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
                //         // parent = x and y rotation
                //         // child = z-offset
                //         let rot_matrix = Mat3::from_quat(transform.rotation);
                //         transform.translation =
                //             pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
                //     }
                // }
            }
        }
    }
}

/// Slows down movement in the XZ plane.
fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
    for (damping_factor, mut linear_velocity) in &mut query {
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
        linear_velocity.x *= damping_factor.0;
        linear_velocity.z *= damping_factor.0;
    }
}

/// Tags an entity as capable of panning and orbiting.
#[derive(Debug, Component)]
pub struct PanOrbitCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

/// Update camera based on where the capsule is.
fn move_camera(
    mut windows: Query<&mut Window>,
    mut ev_motion: EventReader<MouseMotion>,
    // mut ev_scroll: EventReader<MouseWheel>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
    input_mouse: ResMut<ButtonInput<MouseButton>>,
) {
    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;

    let mut rotation_move = Vec2::ZERO;
    let mut _scroll: f32 = 0.0;
    let mut orbit_button_changed = false;

    // if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.read() {
            rotation_move += ev.delta;
        }
    // }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, _projection) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let window = get_primary_window_size(&mut windows);
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down {
                    delta
                } else {
                    -delta
                }
            };
            let delta_y = -rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_axis_angle(Vec3::Y, delta_x);
            let pitch = Quat::from_axis_angle(Vec3::X, delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }

    // consume any remaining events, so they don't pile up if we don't need them
    // (and also to avoid Bevy warning us about not checking events every frame update)
    ev_motion.clear();
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
fn _pan_orbit_camera(
    mut windows: Query<&mut Window>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
    input_mouse: ResMut<ButtonInput<MouseButton>>,
    // mut _mouse_button_input_events: EventReader<MouseButtonInput>,
) {
    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Middle;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.read() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.read() {
            pan += ev.delta;
        }
    }
    for ev in ev_scroll.read() {
        scroll += ev.y;
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let window = get_primary_window_size(&mut windows);
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            let window = get_primary_window_size(&mut windows);
            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            }
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }

    // consume any remaining events, so they don't pile up if we don't need them
    // (and also to avoid Bevy warning us about not checking events every frame update)
    ev_motion.clear();
}

fn get_primary_window_size(windows: &mut Query<&mut Window>) -> Vec2 {
    let window = windows.get_single_mut().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}

/// Spawn a camera like this
fn _spawn_camera(mut commands: Commands) {
    let hitbox = PlayerHitbox::default();
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(hitbox.position).looking_at(
                hitbox.looking_at,
                Vec3::Y,
            ),
            ..Default::default()
        },
        PanOrbitCamera {
            focus: hitbox.looking_at,
            radius: 0.1,
            ..Default::default()
        },
    ));
}

fn setup_cursor(
    mut windows: Query<&mut Window>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let window: Mut<Window> = windows.single_mut();
    // window.cursor.visible = false;
    let cursor_spawn: Vec3 = Vec3::ZERO;
    let center_x = window.width() / 2.0;
    let center_y = window.height() / 2.0;
    let cursor_width: f32 = 32.0; // in pixels
    let cursor_height: f32 = 32.0; // in pixels

    commands.spawn((
        ImageBundle {
            image: asset_server.load("icons/cursor/precision.png").into(),
            style: Style {
                // display: Display::None,
                position_type: PositionType::Absolute,
                // position: UiRect::all(Val::Auto),
                width: Val::Px(cursor_width),
                height: Val::Px(cursor_height),
                left: Val::Px(center_x - cursor_width / 2.0), // half the image width
                top: Val::Px(center_y - cursor_height / 2.0), // half the image height
                ..default()
            },
            z_index: ZIndex::Global(15),
            transform: Transform::from_translation(cursor_spawn),
            ..default()
        },
        GameCursor {}
    ));
}

fn _move_cursor(window: Query<&Window>, mut cursor: Query<&mut Style, With<GameCursor>>) {
    let window: &Window = window.single();
    let center_x = window.width() / 2.0;
    let center_y = window.height() / 2.0;
    if let Some(_position) = window.cursor_position() {
        let mut img_style = cursor.single_mut();
        // img_style.left = Val::Px(position.x - 16.0);
        // img_style.top = Val::Px(position.y - 16.0);
        img_style.left = Val::Px(center_x - 16.0);
        img_style.top = Val::Px(center_y - 16.0);
    }
}

#[cfg(target_os = "windows")]
fn cursor_recenter(
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut primary_window = q_windows.single_mut();
    let center = Vec2::new(
        primary_window.width() / 2.0,
        primary_window.height() / 2.0,
    );
    primary_window.set_cursor_position(Some(center));
}
