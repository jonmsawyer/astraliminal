//! Character controller.

use std::f32::consts::PI;

use bevy::{app::AppExit, ecs::query::Has, input::mouse::*, prelude::*};
use bevy_xpbd_3d::{math::*, prelude::*};

use super::debugger::DebugData;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MovementAction>()
            .init_resource::<CharacterData>()
            .add_systems(
                Update,
                (
                    keyboard_input,
                    gamepad_input,
                    update_grounded,
                    movement,
                    apply_movement_damping,
                )
                    .chain(),
            )
            .add_systems(Update, (move_camera, reset_player));
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Resource)]
pub struct CharacterData {
    /// Look angle. Values are in degrees.
    pub look_angle: Vec3,
    /// Look sensitivity. Default is 0.5.
    pub look_sensitivity: f32,
    /// Yaw rotation from origin in radians.
    pub yaw: f32,
    /// Pitch rotation from origin in radians.
    pub pitch: f32,
}

impl Default for CharacterData {
    fn default() -> Self {
        CharacterData {
            // last_y_rotation: 0,
            look_angle: Vec3::ZERO,
            look_sensitivity: 0.5,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct CharacterHitbox {
    pub length: f32,
    pub radius: f32,
    pub _eye: f32,
    pub position: Vec3,
    pub looking_at: Vec3,
}

impl Default for CharacterHitbox {
    fn default() -> CharacterHitbox {
        //let _eye: f32 = 1.6256;
        let _eye: f32 = 1.651;
        CharacterHitbox {
            length: 1.778,
            radius: 0.762,
            _eye,
            position: Vec3::new(0.0, _eye, 1.0),
            looking_at: Vec3::new(0.0, _eye, 0.0), // Look at NEG_Z at eye level.
        }
    }
}

impl CharacterHitbox {
    pub fn transform(&self) -> Transform {
        Transform::from_translation(self.position).looking_at(self.looking_at, Vec3::Y)
    }
}
/// An event sent for a movement input action.
#[derive(Debug, Event)]
pub enum MovementAction {
    Move(Vector2),
    Look(Vector2),
    Jump,
}

/// Tags an entity as capable of panning and orbiting.
// #[derive(Debug, Component)]
// pub struct PanOrbitCamera {
//     /// The "focus point" to orbit around. It is automatically updated when panning the camera
//     pub focus: Vec3,
//     pub radius: f32,
//     pub upside_down: bool,
// }

// impl Default for PanOrbitCamera {
//     fn default() -> Self {
//         PanOrbitCamera {
//             focus: Vec3::ZERO,
//             radius: 5.0,
//             upside_down: false,
//         }
//     }
// }

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

/// A marker component indicating that an entity is using a character controller.
#[derive(Debug, Component)]
pub struct CharacterController;

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

/// Sends [`MovementAction`] events based on keyboard input.
fn keyboard_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let forward = keyboard_input.any_pressed([KeyCode::KeyW]);
    let back = keyboard_input.any_pressed([KeyCode::KeyS]);
    let strafe_left = keyboard_input.any_pressed([KeyCode::KeyA]);
    let strafe_right = keyboard_input.any_pressed([KeyCode::KeyD]);

    let look_up = keyboard_input.any_pressed([KeyCode::ArrowUp]);
    let look_down = keyboard_input.any_pressed([KeyCode::ArrowDown]);
    let look_left = keyboard_input.any_pressed([KeyCode::ArrowLeft]);
    let look_right = keyboard_input.any_pressed([KeyCode::ArrowRight]);

    let planar_x = strafe_right as i8 - strafe_left as i8;
    let planar_z = back as i8 - forward as i8;
    let direction = Vector2::new(planar_x as Scalar, -planar_z as Scalar).clamp_length_max(1.0);

    let look_y = look_up as i8 - look_down as i8;
    let look_x = look_right as i8 - look_left as i8;
    let look_direction = Vector2::new(look_x as Scalar, look_y as Scalar).clamp_length_max(1.0);

    let quit = keyboard_input.any_pressed([KeyCode::Escape]);

    if direction.length_squared() > 0.0 {
        movement_event_writer.send(MovementAction::Move(direction));
    }

    if look_direction.length_squared() > 0.0 {
        movement_event_writer.send(MovementAction::Look(look_direction));
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_event_writer.send(MovementAction::Jump);
    }

    if quit {
        app_exit_events.send(AppExit);
    }
}

/// Sends [`MovementAction`] events based on gamepad input.
fn _gamepad_input(
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
    mut debug_data: ResMut<DebugData>,
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
            debug_data.is_grounded = true;
            commands.entity(entity).insert(Grounded);
        } else {
            debug_data.is_grounded = false;
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

/// Responds to [`MovementAction`] events and moves character controllers accordingly.
fn movement(
    mut controllers: Query<(
        &MovementAcceleration,
        &JumpImpulse,
        &mut LinearVelocity,
        &mut CharacterHitbox,
        &mut Transform,
        &mut Projection,
        Has<Grounded>,
    )>,
    mut _windows: Query<&mut Window>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut debug_data: ResMut<DebugData>,
    time: Res<Time>,
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
            // mut pan_orbit_camera,
            transform,
            mut _projection,
            is_grounded,
        ) in &mut controllers
        {
            let forward = transform.forward();
            let back = transform.back();
            let left = transform.left();
            let right = transform.right();

            debug_data.character_position = transform.translation;

            match event {
                MovementAction::Move(direction) => {
                    debug_data.direction = *direction;
                    if direction.x > 0.0 {
                        linear_velocity.x += right.x * movement_acceleration.0 * delta_time;
                        linear_velocity.z += right.z * movement_acceleration.0 * delta_time;
                    }
                    if direction.x < 0.0 {
                        linear_velocity.x += left.x * movement_acceleration.0 * delta_time;
                        linear_velocity.z += left.z * movement_acceleration.0 * delta_time;
                    }
                    if direction.y > 0.0 {
                        linear_velocity.x += forward.x * movement_acceleration.0 * delta_time;
                        linear_velocity.z += forward.z * movement_acceleration.0 * delta_time;
                    }
                    if direction.y < 0.0 {
                        linear_velocity.x += back.x * movement_acceleration.0 * delta_time;
                        linear_velocity.z += back.z * movement_acceleration.0 * delta_time;
                    }
                }
                MovementAction::Jump => {
                    debug_data.is_grounded = is_grounded;
                    if is_grounded {
                        linear_velocity.y = jump_impulse.0;
                    }
                }
                _ => {}
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

/// Update camera based on where the player is at.
fn move_camera(
    mut mouse_motion_event: EventReader<MouseMotion>,
    mut look_event: EventReader<MovementAction>,
    mut windows: Query<&mut Window>,
    mut transform_query: Query<&mut Transform, With<Projection>>,
    mut debug_data: ResMut<DebugData>,
    mut character_data: ResMut<CharacterData>,
) {
    let mut rotation_move = Vec2::ZERO;
    let mut look_direction = Vec2::ZERO;

    for mme in mouse_motion_event.read() {
        rotation_move += mme.delta;
    }

    for le in look_event.read() {
        match le {
            MovementAction::Look(direction) => {
                look_direction = *direction;
            }
            _ => {}
        }
    }

    for mut transform in transform_query.iter_mut() {
        let mut delta_x = 0.0;
        let mut delta_y = 0.0;

        // If the camera is "upside" down, panning horizontally would be inverted,
        // so invert the input to make it correct.
        let up = transform.rotation * Vec3::Y;
        debug_data.is_upside_down = up.y < 0.0;

        let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
        character_data.yaw = yaw; // In radians.
        character_data.pitch = pitch; // In radians.

        if rotation_move.length_squared() > 0.0 {
            let window = get_primary_window_size(&mut windows);
            delta_x = {
                let delta = rotation_move.x / window.x * PI * 2.0;
                if debug_data.is_upside_down {
                    -delta
                } else {
                    delta
                }
            };

            delta_y = rotation_move.y / window.y * PI;
        }

        if look_direction.length_squared() > 0.0 {
            delta_x = look_direction.x * 0.05;
            delta_y = look_direction.y * 0.05;
        }

        // Apply look update
        character_data.pitch = (character_data.pitch - delta_y * character_data.look_sensitivity)
            .clamp(-PI / 2., PI / 2.);
        character_data.yaw -= delta_x * character_data.look_sensitivity;
        transform.rotation =
            Quat::from_euler(EulerRot::ZYX, 0.0, character_data.yaw, character_data.pitch);

        let (x, y, z) = transform.rotation.to_euler(EulerRot::YXZ);
        character_data.look_angle = Vec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees());

        debug_data.character_looking_at = transform.rotation.xyz();
    }

    // Consume any remaining events, so they don't pile up if we don't need them
    // (and also to avoid Bevy warning us about not checking events every frame update).
    mouse_motion_event.clear();
    look_event.clear();
}

// /// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
// fn _pan_orbit_camera(
//     mut windows: Query<&mut Window>,
//     mut ev_motion: EventReader<MouseMotion>,
//     mut ev_scroll: EventReader<MouseWheel>,
//     mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
//     input_mouse: ResMut<ButtonInput<MouseButton>>,
//     // mut _mouse_button_input_events: EventReader<MouseButtonInput>,
// ) {
//     // change input mapping for orbit and panning here
//     let orbit_button = MouseButton::Right;
//     let pan_button = MouseButton::Middle;

//     let mut pan = Vec2::ZERO;
//     let mut rotation_move = Vec2::ZERO;
//     let mut scroll = 0.0;
//     let mut orbit_button_changed = false;

//     if input_mouse.pressed(orbit_button) {
//         for ev in ev_motion.read() {
//             rotation_move += ev.delta;
//         }
//     } else if input_mouse.pressed(pan_button) {
//         // Pan only if we're not rotating at the moment
//         for ev in ev_motion.read() {
//             pan += ev.delta;
//         }
//     }
//     for ev in ev_scroll.read() {
//         scroll += ev.y;
//     }
//     if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
//         orbit_button_changed = true;
//     }

//     for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
//         if orbit_button_changed {
//             // only check for upside down when orbiting started or ended this frame
//             // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
//             let up = transform.rotation * Vec3::Y;
//             pan_orbit.upside_down = up.y <= 0.0;
//         }

//         let mut any = false;
//         if rotation_move.length_squared() > 0.0 {
//             any = true;
//             let window = get_primary_window_size(&mut windows);
//             let delta_x = {
//                 let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
//                 if pan_orbit.upside_down {
//                     -delta
//                 } else {
//                     delta
//                 }
//             };
//             let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
//             let yaw = Quat::from_rotation_y(-delta_x);
//             let pitch = Quat::from_rotation_x(-delta_y);
//             transform.rotation = yaw * transform.rotation; // rotate around global y axis
//             transform.rotation = transform.rotation * pitch; // rotate around local x axis
//         } else if pan.length_squared() > 0.0 {
//             any = true;
//             // make panning distance independent of resolution and FOV,
//             let window = get_primary_window_size(&mut windows);
//             if let Projection::Perspective(projection) = projection {
//                 pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
//             }
//             // translate by local axes
//             let right = transform.rotation * Vec3::X * -pan.x;
//             let up = transform.rotation * Vec3::Y * pan.y;
//             // make panning proportional to distance away from focus point
//             let translation = (right + up) * pan_orbit.radius;
//             pan_orbit.focus += translation;
//         } else if scroll.abs() > 0.0 {
//             any = true;
//             pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
//             // dont allow zoom to reach zero or you get stuck
//             pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
//         }

//         if any {
//             // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
//             // parent = x and y rotation
//             // child = z-offset
//             let rot_matrix = Mat3::from_quat(transform.rotation);
//             transform.translation =
//                 pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
//         }
//     }

//     // consume any remaining events, so they don't pile up if we don't need them
//     // (and also to avoid Bevy warning us about not checking events every frame update)
//     ev_motion.clear();
// }

fn get_primary_window_size(windows: &mut Query<&mut Window>) -> Vec2 {
    let window = windows.get_single_mut().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}

// /// Spawn a camera like this
// fn _spawn_camera(mut commands: Commands) {
//     let hitbox = PlayerHitbox::default();
//     commands.spawn((
//         Camera3dBundle {
//             transform: Transform::from_translation(hitbox.position)
//                 .looking_at(hitbox.looking_at, Vec3::Y),
//             ..Default::default()
//         },
//         PanOrbitCamera {
//             focus: hitbox.looking_at,
//             radius: 0.1,
//             ..Default::default()
//         },
//     ));
// }

fn reset_player(
    mut debug_data: ResMut<DebugData>,
    mut query: Query<&mut Transform, With<Projection>>,
) {
    if debug_data.is_changed() {
        if debug_data.reset_player {
            for mut transform in query.iter_mut() {
                let hitbox = CharacterHitbox::default();
                *transform = hitbox.transform();
                // pan_orbit_camera.focus = transform.translation;
            }
            debug_data.reset_player = false;
        }
    }
}
