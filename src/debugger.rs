use bevy::{
    input::{
        keyboard::*,
        // mouse::{MouseScrollUnit, MouseWheel},
        ButtonState,
    },
    prelude::*,
};
// use bevy::{ecs::query::Has, app::AppExit, prelude::*};
// use bevy::window::{Window, WindowMode, WindowLevel};
// use bevy_xpbd_3d::{math::*, prelude::*};

mod fps;
use fps::{DebuggerFpsPlugin, FpsResource};

mod ui_bundles;
use ui_bundles::{
    DebugUiCharacterLookingAtBundle, DebugUiCharacterPositionBundle, DebugUiContainerBundle,
    DebugUiDirectionBundle, DebugUiFpsBundle, DebugUiIsGroundedBundle, DebugUiNodeBundle,
    DebugUiTextBundle, DebugUiTitleBundle,
};

mod ui_components;
use ui_components::{
    DebugUiCharacterLookingAt, DebugUiCharacterPosition, DebugUiContainer, DebugUiDirection,
    DebugUiFps, DebugUiIsGrounded,
};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, States)]
pub enum DebugState {
    #[default]
    Disabled,
    Enabled,
}

#[derive(Debug, Default, Copy, Clone, Resource)]
pub struct DebugData {
    /// A Vec2 containing the player's controller/keyboard direction.
    pub direction: Vec2,
    /// A boolean representing if the character/player is grounded or not.
    pub is_grounded: bool,
    /// A Vec3 containing the player's position.
    pub character_position: Vec3,
    /// A Vec3 containing the vector of looking direction by the player.
    pub character_looking_at: Vec3,
    /// Reset the player to their starting position.
    pub reset_player: bool,
    /// True if debugger should be shown, false if debugger should be hidden.
    pub is_visible: bool,
}

pub struct DebuggerPlugin;

impl Plugin for DebuggerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DebuggerFpsPlugin,))
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
                (update_debugger,).run_if(in_state(DebugState::Enabled)),
            );
    }
}

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

fn spawn_debugger(mut commands: Commands) {
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
                                                "[ Diagnostics ]".to_string(),
                                                Some(TextStyle {
                                                    font_size: 30.0,
                                                    ..default()
                                                }),
                                            ));
                                            parent.spawn(DebugUiFpsBundle::new(
                                                0.0,
                                                Some(TextStyle {
                                                    font_size: 24.0,
                                                    ..default()
                                                }),
                                            ));
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
                                                "[ Environment ]".to_string(),
                                                Some(TextStyle {
                                                    font_size: 30.0,
                                                    ..default()
                                                }),
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "GPU...".to_string(),
                                                Some(TextStyle {
                                                    font_size: 24.0,
                                                    ..default()
                                                }),
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "CPU...".to_string(),
                                                Some(TextStyle {
                                                    font_size: 24.0,
                                                    ..default()
                                                }),
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "RAM...".to_string(),
                                                Some(TextStyle {
                                                    font_size: 24.0,
                                                    ..default()
                                                }),
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "Resource Graph...".to_string(),
                                                Some(TextStyle {
                                                    font_size: 24.0,
                                                    ..default()
                                                }),
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
                                                "[ Player ]".to_string(),
                                                Some(TextStyle {
                                                    // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                                    font_size: 30.0,
                                                    ..default()
                                                }),
                                            ));
                                            parent.spawn(DebugUiDirectionBundle::new(
                                                Vec2::ZERO,
                                                None,
                                            ));
                                            parent.spawn(DebugUiIsGroundedBundle::new(true, None));
                                            parent.spawn(DebugUiCharacterPositionBundle::new(
                                                Vec3::ZERO,
                                                None,
                                            ));
                                            // parent.spawn(DebugUiTextBundle::new(
                                            //     "Looking At Coord: x=0.0, y=0.0, z=0.0".to_string(),
                                            //     None,
                                            // ));
                                            parent.spawn(DebugUiCharacterLookingAtBundle::new(
                                                Vec3::ZERO,
                                                None,
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "Looking At Entity: x=0.0, y=0.0, z=0.0"
                                                    .to_string(),
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
                                            None, //Some(Color::rgb(0.15, 0.85, 0.85)),
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn(DebugUiTextBundle::new(
                                                "[ World ]".to_string(),
                                                Some(TextStyle {
                                                    // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                                    font_size: 30.0,
                                                    ..default()
                                                }),
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "Level: -".to_string(),
                                                None,
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "Puzzle #: -".to_string(),
                                                None,
                                            ));
                                            parent.spawn(DebugUiTextBundle::new(
                                                "Checkpoints: -".to_string(),
                                                None,
                                            ));
                                        });
                                });
                        });
                });
        });
}

fn update_debugger(
    mut _commands: Commands,
    mut container_query: Query<&mut Style, With<DebugUiContainer>>,
    mut fps_query: Query<
        &mut Text,
        (
            With<DebugUiFps>,
            Without<DebugUiDirection>,
            Without<DebugUiIsGrounded>,
            Without<DebugUiCharacterPosition>,
            Without<DebugUiCharacterLookingAt>,
        ),
    >,
    mut direction_query: Query<
        &mut Text,
        (
            With<DebugUiDirection>,
            Without<DebugUiFps>,
            Without<DebugUiIsGrounded>,
            Without<DebugUiCharacterPosition>,
            Without<DebugUiCharacterLookingAt>,
        ),
    >,
    mut is_grounded_query: Query<
        &mut Text,
        (
            With<DebugUiIsGrounded>,
            Without<DebugUiDirection>,
            Without<DebugUiFps>,
            Without<DebugUiCharacterPosition>,
            Without<DebugUiCharacterLookingAt>,
        ),
    >,
    mut position_query: Query<
        &mut Text,
        (
            With<DebugUiCharacterPosition>,
            Without<DebugUiFps>,
            Without<DebugUiDirection>,
            Without<DebugUiIsGrounded>,
            Without<DebugUiCharacterLookingAt>,
        ),
    >,
    mut looking_at_query: Query<
        &mut Text,
        (
            With<DebugUiCharacterLookingAt>,
            Without<DebugUiFps>,
            Without<DebugUiDirection>,
            Without<DebugUiIsGrounded>,
            Without<DebugUiCharacterPosition>,
        ),
    >,
    debug_data: Res<DebugData>,
    fps: Res<FpsResource<25>>,
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

    // Process FPS counter.
    // for (parent, _entity) in fps_query.iter_mut() {
    for mut text in fps_query.iter_mut() {
        let fps = if fps.average > f32::EPSILON {
            fps.average
        } else {
            0.0
        };
        *text = Text::from_section(
            format!("FPS: {}", fps),
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        );
    }

    // Process Direction info.
    for mut text in direction_query.iter_mut() {
        *text = Text::from_section(
            format!(
                "Direction: x={}, z={}",
                debug_data.direction.x, debug_data.direction.y
            ),
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        );
    }

    // Process is_grounded.
    for mut text in is_grounded_query.iter_mut() {
        *text = Text::from_section(
            format!("Is Grounded?: {}", debug_data.is_grounded),
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        );
    }

    // Process character position.
    for mut text in position_query.iter_mut() {
        *text = Text::from_section(
            format!(
                "Coordinates: x={}, y={}, z={}",
                debug_data.character_position.x,
                debug_data.character_position.y,
                debug_data.character_position.z,
            ),
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        );
    }

    // Process character looking at.
    for mut text in looking_at_query.iter_mut() {
        *text = Text::from_section(
            format!(
                "Looking At Coord: x={}, y={}, z={}",
                debug_data.character_looking_at.x,
                debug_data.character_looking_at.y,
                debug_data.character_looking_at.z,
            ),
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        );
    }
}

fn despawn_debugger(
    mut commands: Commands,
    mut container_query: Query<Entity, With<DebugUiContainer>>,
) {
    commands
        .entity(container_query.single_mut())
        .despawn_recursive();
}
