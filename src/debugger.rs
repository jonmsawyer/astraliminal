use bevy::{
    input::keyboard::KeyboardInput,
    input::ButtonState,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    // core::FrameCount,
    prelude::*,
    reflect::TypePath,
    // window::{Cursor, CursorGrabMode, Window, WindowMode, WindowPlugin, WindowResolution},
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};

mod fps;
use fps::{DebuggerFpsPlugin, FpsResource};

mod ui_bundles;
use ui_bundles::{
    DebugUiCharacterLookingAtBundle, DebugUiCharacterPositionBundle, DebugUiContainerBundle,
    DebugUiDirectionBundle, DebugUiFpsBundle, DebugUiIsGroundedBundle, DebugUiIsUpsideDownBundle,
    DebugUiNodeBundle, DebugUiTextBundle, DebugUiTitleBundle,
};

mod ui_components;
use ui_components::{
    DebugUiAxes, DebugUiCharacterLookingAt, DebugUiCharacterPosition, DebugUiContainer,
    DebugUiDirection, DebugUiFps, DebugUiIsGrounded, DebugUiIsUpsideDown,
};

pub const AXIS_LENGTH: f32 = 1000.0;
pub const AXIS_THICKNESS: f32 = 0.1;
pub const AXIS_SPECULAR_TRANSMISSION: f32 = 1.0;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, States)]
pub enum DebugState {
    #[default]
    Disabled,
    Enabled,
}

#[derive(Debug, Default, Clone, Resource)]
pub struct DebugData {
    /// A Vec2 containing the player's controller/keyboard direction.
    pub direction: Vec2,
    /// A boolean representing if the character/player is grounded or not.
    pub is_grounded: bool,
    /// A Vec3 containing the player's position.
    pub character_position: Vec3,
    /// A Vec3 containing the vector of looking direction by the player.
    pub character_looking_at: Vec3,
    /// Is the player trying to look upside down?
    pub is_upside_down: (bool, Vec2),
    /// Reset the player to their starting position.
    pub reset_player: bool,
    /// True if debugger should be shown, false if debugger should be hidden.
    pub is_visible: bool,
}

#[derive(Asset, TypePath, Default, AsBindGroup, Debug, Clone)]
pub struct LineMaterial {
    #[uniform(0)]
    pub base_color: Color,
    #[uniform(0)]
    pub color: Color,
    pub thickness: f32,
    pub specular_transmission: f32,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

impl From<LineMaterial> for StandardMaterial {
    fn from(line_material: LineMaterial) -> StandardMaterial {
        StandardMaterial {
            base_color: line_material.base_color,
            emissive: line_material.base_color,
            thickness: AXIS_THICKNESS,
            ..default()
        }
    }
}

/// A list of lines with a start and end position.
#[derive(Debug, Clone)]
pub struct LineList {
    pub lines: Vec<(Vec3, Vec3)>,
}

impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();

        Mesh::new(
            // This tells wgpu that the positions are list of lines
            // where every pair is a start and end point
            PrimitiveTopology::LineList,
            RenderAssetUsages::RENDER_WORLD,
        )
        // Add the vertices positions as an attribute
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    }
}

/// A list of points that will have a line drawn between each consecutive points.
#[derive(Debug, Clone)]
pub struct LineStrip {
    pub points: Vec<Vec3>,
}

impl From<LineStrip> for Mesh {
    fn from(line: LineStrip) -> Self {
        Mesh::new(
            // This tells wgpu that the positions are a list of points
            // where a line will be drawn between each consecutive point
            PrimitiveTopology::LineStrip,
            RenderAssetUsages::RENDER_WORLD,
        )
        // Add the point positions as an attribute
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, line.points)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DebugUiTextStyle;

impl DebugUiTextStyle {
    /// Return a new TextStyle.
    fn new(font_size: f32, color: Color) -> TextStyle {
        TextStyle {
            font_size,
            color,
            ..default()
        }
    }

    /// Return the default Debugger UI text style.
    fn default() -> TextStyle {
        DebugUiTextStyle::new(24.0, Color::WHITE)
    }

    /// Return the heading Debugger UI text style.
    fn heading() -> TextStyle {
        DebugUiTextStyle::new(30.0, Color::WHITE)
    }
}

pub struct DebuggerPlugin;

impl Plugin for DebuggerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DebuggerFpsPlugin, MaterialPlugin::<LineMaterial>::default()))
            .insert_state(DebugState::Disabled)
            .init_resource::<DebugData>()
            .add_event::<KeyboardInput>()
            .add_systems(OnEnter(DebugState::Enabled), (spawn_debugger,))
            .add_systems(OnExit(DebugState::Enabled), (despawn_debugger,))
            .add_systems(
                Update,
                ((
                    keyboard_input,
                    // gamepad_input,
                )
                    .chain(),),
            )
            .add_systems(
                PreUpdate,
                (
                    update_debugger_ui,
                    update_fps,
                    update_direction,
                    update_is_grounded,
                    update_character_position,
                    update_character_looking_at,
                    update_is_upside_down,
                )
                    .run_if(in_state(DebugState::Enabled)),
            );
    }
}

/// Process keyboard input for the Debugger UI.
fn keyboard_input(
    mut input: EventReader<KeyboardInput>,
    mut debug_data: ResMut<DebugData>,
    mut next_state: ResMut<NextState<DebugState>>,
) {
    for key_input in input.read() {
        if key_input.key_code == KeyCode::F3 && key_input.state == ButtonState::Pressed {
            if debug_data.is_visible {
                debug_data.is_visible = false;
                next_state.set(DebugState::Disabled);
                info!("{:?} was pressed. Debugger disabled.", key_input);
            } else {
                debug_data.is_visible = true;
                next_state.set(DebugState::Enabled);
                info!("{:?} was pressed. Debugger enabled.", key_input);
            }
        }
        if key_input.key_code == KeyCode::F5 && key_input.state == ButtonState::Pressed {
            debug_data.reset_player = true;
        }
    }
}

// /// Sends [`MovementAction`] events based on gamepad input.
// fn gamepad_input(
//     mut movement_event_writer: EventWriter<MovementAction>,
//     gamepads: Res<Gamepads>,
//     axes: Res<Axis<GamepadAxis>>,
//     buttons: Res<ButtonInput<GamepadButton>>,
// ) {
//     for gamepad in gamepads.iter() {
//         let axis_lx = GamepadAxis {
//             gamepad,
//             axis_type: GamepadAxisType::LeftStickX,
//         };
//         let axis_ly = GamepadAxis {
//             gamepad,
//             axis_type: GamepadAxisType::LeftStickY,
//         };

//         if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
//             movement_event_writer.send(MovementAction::Move(
//                 Vector2::new(x as Scalar, y as Scalar).clamp_length_max(1.0),
//             ));
//         }

//         let jump_button = GamepadButton {
//             gamepad,
//             button_type: GamepadButtonType::South,
//         };

//         if buttons.just_pressed(jump_button) {
//             movement_event_writer.send(MovementAction::Jump);
//         }
//     }
// }

/// Spawn the Debugger UI.
fn spawn_debugger(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Display axes.

    // Y-axis
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(LineList {
                lines: vec![(
                    Vec3::new(0.0, -AXIS_LENGTH, 0.0),
                    Vec3::new(0.0, AXIS_LENGTH, 0.0),
                )],
            }),
            // transform: Transform::from_xyz(0.0, 0.0, 0.0),
            material: materials.add(LineMaterial {
                base_color: Color::BLUE,
                color: Color::BLUE,
                thickness: AXIS_THICKNESS,
                specular_transmission: AXIS_SPECULAR_TRANSMISSION,
            }),
            ..default()
        },
        DebugUiAxes,
    ));

    // Z-axis
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(LineList {
                lines: vec![(
                    Vec3::new(0.0, 0.0, -AXIS_LENGTH),
                    Vec3::new(0.0, 0.0, AXIS_LENGTH),
                )],
            }),
            // transform: Transform::from_xyz(0.0, 0.0, 0.0),
            material: materials.add(LineMaterial {
                base_color: Color::GREEN,
                color: Color::GREEN,
                thickness: AXIS_THICKNESS,
                specular_transmission: AXIS_SPECULAR_TRANSMISSION,
            }),
            ..default()
        },
        DebugUiAxes,
    ));

    // X-axis
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(LineList {
                lines: vec![(
                    Vec3::new(-AXIS_LENGTH, 0.0, 0.0),
                    Vec3::new(AXIS_LENGTH, 0.0, 0.0),
                )],
            }),
            // transform: Transform::from_xyz(0.0, 0.0, 0.0),
            material: materials.add(LineMaterial {
                base_color: Color::RED,
                color: Color::RED,
                thickness: AXIS_THICKNESS,
                specular_transmission: AXIS_SPECULAR_TRANSMISSION,
            }),
            ..default()
        },
        DebugUiAxes,
    ));

    // Root node
    commands
        .spawn(DebugUiNodeBundle::new(None, None))
        .with_children(|parent| {
            // Top panel (horizonal fill)
            parent
                .spawn(DebugUiContainerBundle::new(None, None))
                .with_children(|parent| {
                    // Title
                    parent
                        .spawn(DebugUiNodeBundle::new(
                            Some(Style {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                ..default()
                            }),
                            Some(Color::rgba(0.1, 0.1, 0.1, 0.8)),
                        ))
                        .with_children(|parent| {
                            parent.spawn(DebugUiTitleBundle::new(None));
                        });
                    // Bottom panel (for left and right panels)
                    parent
                        .spawn(DebugUiNodeBundle::new(None, None))
                        .with_children(|parent| {
                            parent
                                .spawn(DebugUiNodeBundle::new(
                                    Some(Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Row,
                                        ..default()
                                    }),
                                    None,
                                ))
                                .with_children(|parent| {
                                    // Top Left Panel
                                    parent
                                        .spawn(DebugUiNodeBundle::new(
                                            Some(Style {
                                                width: Val::Percent(50.0),
                                                height: Val::Percent(100.0),
                                                flex_direction: FlexDirection::Column,
                                                padding: UiRect::all(Val::Px(10.0)),
                                                ..default()
                                            }),
                                            None, //Some(Color::rgb(0.85, 0.15, 0.15)),
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn(DebugUiTextBundle::new(
                                                "[ Diagnostics ]".into(),
                                                None,
                                            ));
                                            parent.spawn(DebugUiFpsBundle::new(0.0, None));
                                        });
                                    // Top Right Panel
                                    parent
                                        .spawn(DebugUiNodeBundle::new(
                                            Some(Style {
                                                width: Val::Percent(50.0),
                                                height: Val::Percent(100.0),
                                                flex_direction: FlexDirection::Column,
                                                align_items: AlignItems::FlexEnd,
                                                padding: UiRect::all(Val::Px(10.0)),
                                                ..default()
                                            }),
                                            None, //Some(Color::rgb(0.15, 0.15, 0.85)),
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn(DebugUiTextBundle::new(
                                                "[ Environment ]".into(),
                                                Some(DebugUiTextStyle::heading()),
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "GPU...".into(),
                                                None,
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "CPU...".into(),
                                                None,
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "RAM...".into(),
                                                None,
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "Resource Graph...".into(),
                                                None,
                                            ));
                                        });
                                });
                            parent
                                .spawn(DebugUiNodeBundle::new(
                                    Some(Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Row,
                                        ..default()
                                    }),
                                    None,
                                ))
                                .with_children(|parent| {
                                    // Bottom Left Panel
                                    parent
                                        .spawn(DebugUiNodeBundle::new(
                                            Some(Style {
                                                width: Val::Percent(50.0),
                                                height: Val::Percent(100.0),
                                                flex_direction: FlexDirection::ColumnReverse,
                                                padding: UiRect::all(Val::Px(10.0)),
                                                ..default()
                                            }),
                                            None, //Some(Color::rgb(0.85, 0.85, 0.15)),
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn(DebugUiTextBundle::new(
                                                "[ Player ]".into(),
                                                Some(DebugUiTextStyle::heading()),
                                            ));
                                            parent.spawn(DebugUiDirectionBundle::new(
                                                Vec2::ZERO,
                                                None,
                                            ));
                                            parent.spawn(DebugUiIsUpsideDownBundle::new(
                                                (false, Vec2::ZERO),
                                                None,
                                            ));
                                            parent.spawn(DebugUiIsGroundedBundle::new(true, None));
                                            parent.spawn(DebugUiCharacterPositionBundle::new(
                                                Vec3::ZERO,
                                                None,
                                            ));
                                            // parent.spawn(DebugUiTextBundle::new(
                                            //     "Looking At Coord: x=0.0, y=0.0, z=0.0".into(),
                                            //     None,
                                            // ));
                                            parent.spawn(DebugUiCharacterLookingAtBundle::new(
                                                Vec3::ZERO,
                                                None,
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "Looking At Entity: x=0.0, y=0.0, z=0.0".into(),
                                                None,
                                            ));
                                        });
                                    // Bottom Right Panel
                                    parent
                                        .spawn(DebugUiNodeBundle::new(
                                            Some(Style {
                                                width: Val::Percent(50.0),
                                                height: Val::Percent(100.0),
                                                flex_direction: FlexDirection::ColumnReverse,
                                                align_items: AlignItems::FlexEnd,
                                                padding: UiRect::all(Val::Px(10.0)),
                                                ..default()
                                            }),
                                            None,
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn(DebugUiTextBundle::new(
                                                "[ World ]".into(),
                                                Some(DebugUiTextStyle::heading()),
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "Level: -".into(),
                                                None,
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "Puzzle #: -".into(),
                                                None,
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "Checkpoints: -".into(),
                                                None,
                                            ));
                                        });
                                });
                        });
                });
        });
}

/// Display or hide the Debugger UI.
fn update_debugger_ui(
    mut container_query: Query<&mut Style, With<DebugUiContainer>>,
    debug_data: Res<DebugData>,
) {
    if debug_data.is_changed() {
        for mut style in container_query.iter_mut() {
            if debug_data.is_visible {
                style.display = Display::DEFAULT;
            } else {
                style.display = Display::None;
            }
        }
    }
}

/// Update the FPS in the Debugger UI.
fn update_fps(mut fps_query: Query<&mut Text, With<DebugUiFps>>, fps: Res<FpsResource<25>>) {
    for mut text in fps_query.iter_mut() {
        let fps = if fps.average > f32::EPSILON {
            fps.average
        } else {
            0.0
        };

        *text = Text::from_section(format!("FPS: {}", fps), DebugUiTextStyle::default());
    }
}

/// Update the player's movement direction in the Debugger UI.
fn update_direction(
    mut direction_query: Query<&mut Text, With<DebugUiDirection>>,
    debug_data: Res<DebugData>,
) {
    for mut text in direction_query.iter_mut() {
        let mut direction = "WASD Direction: ".to_string();

        if debug_data.direction.y > 0.0 {
            direction.push_str("Forward");
        } else if debug_data.direction.y < 0.0 {
            direction.push_str("Backward");
        } else {
            direction.push_str("-");
        }

        direction.push_str("/");

        if debug_data.direction.x > 0.0 {
            direction.push_str("S. Right");
        } else if debug_data.direction.x < 0.0 {
            direction.push_str("S. Left");
        } else {
            direction.push_str("-");
        }

        *text = Text::from_section(direction, DebugUiTextStyle::default());
    }
}

/// Update the is grounded status in the Debugger UI.
fn update_is_grounded(
    mut is_grounded_query: Query<&mut Text, With<DebugUiIsGrounded>>,
    debug_data: Res<DebugData>,
) {
    for mut text in is_grounded_query.iter_mut() {
        *text = Text::from_section(
            format!("Is Grounded?: {}", debug_data.is_grounded),
            DebugUiTextStyle::default(),
        );
    }
}

/// Update the player's character position in the Debugger UI.
fn update_character_position(
    mut position_query: Query<&mut Text, With<DebugUiCharacterPosition>>,
    debug_data: Res<DebugData>,
) {
    for mut text in position_query.iter_mut() {
        *text = Text::from_section(
            format!(
                "Coordinates:\n  x = {:.6}\n  y = {:.6}\n  z = {:.6}",
                debug_data.character_position.x,
                debug_data.character_position.y,
                debug_data.character_position.z,
            ),
            DebugUiTextStyle::default(),
        );
    }
}

/// Update the player's character looking at in the Debugger UI.
fn update_character_looking_at(
    mut looking_at_query: Query<&mut Text, With<DebugUiCharacterLookingAt>>,
    debug_data: Res<DebugData>,
) {
    for mut text in looking_at_query.iter_mut() {
        *text = Text::from_section(
            format!(
                "Looking At Coord:\n  x = {:.6}\n  y = {:.6}\n  z = {:.6}",
                debug_data.character_looking_at.x,
                debug_data.character_looking_at.y,
                debug_data.character_looking_at.z,
            ),
            DebugUiTextStyle::default(),
        );
    }
}

/// Update the is upside down status in the Debugger UI.
fn update_is_upside_down(
    mut is_upside_down_query: Query<&mut Text, With<DebugUiIsUpsideDown>>,
    debug_data: Res<DebugData>,
) {
    for mut text in is_upside_down_query.iter_mut() {
        *text = Text::from_section(
            format!(
                "Is Upside Down? {}\nRotation Y: {:?}",
                debug_data.is_upside_down.0, debug_data.is_upside_down.1,
            ),
            DebugUiTextStyle::default(),
        );
    }
}

/// Despawn the Debugger UI.
fn despawn_debugger(
    mut commands: Commands,
    mut container_query: Query<Entity, With<DebugUiContainer>>,
    axes_query: Query<Entity, With<DebugUiAxes>>,
) {
    commands
        .entity(container_query.single_mut())
        .despawn_recursive();
    for axis in axes_query.iter() {
        commands.entity(axis).despawn_recursive();
    }
}
