use bevy::prelude::*;
use bevy::input::keyboard::*;
// use bevy::{ecs::query::Has, app::AppExit, prelude::*};
// use bevy::window::{Window, WindowMode, WindowLevel};
use bevy::input::ButtonState;
// use bevy_xpbd_3d::{math::*, prelude::*};

mod fps;
use fps::{DebuggerFpsPlugin, FpsResource};

#[derive(Debug, Default, Copy, Clone, Resource)]
pub struct DebugData {
    /// A Vec3 containing the player's position.
    pub player_position: Vec3,
    /// A Vec3 containing the vector of looking direction by the player.
    pub player_looking_at: Vec3,
    /// True if debugger should be shown, false if debugger should be hidden.
    pub is_visible: bool,
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct DebugUi;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct DebugUiTitle;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct DebugUiFps;

pub struct DebuggerPlugin;

impl Plugin for DebuggerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                DebuggerFpsPlugin,
            ))
            .init_resource::<DebugData>()
            .add_event::<KeyboardInput>()
            .add_systems(Update, (
                keyboard_input,
                // gamepad_input,
            ).chain())
            .add_systems(Startup, (
                setup_debugger,
                // spawn_box,
                //spawn_text,
            ))
            .add_systems(Update, (
                update_debugger,
            ));
    }
}


// /// A bundle that contains the components needed for a basic
// /// kinematic character controller.
// #[derive(Bundle)]
// pub struct CharacterControllerBundle {
//     character_controller: CharacterController,
//     rigid_body: RigidBody,
//     collider: Collider,
//     ground_caster: ShapeCaster,
//     locked_axes: LockedAxes,
//     movement: MovementBundle,
// }

fn keyboard_input(
    mut input: EventReader<KeyboardInput>,
    mut debug_state: ResMut<DebugData>,
    _time: Res<Time>,
) {
    for key_input in input.read() {
        if key_input.key_code == KeyCode::F3 && key_input.state == ButtonState::Pressed {
            if debug_state.is_visible {
                debug_state.is_visible = false;
                info!("{:?} was pressed. Debugger disabled.", key_input);
            } else {
                debug_state.is_visible = true;
                info!("{:?} was pressed. Debugger enabled.", key_input);
            }
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

fn setup_debugger(
    mut commands: Commands,
) {
    let container_bundle = (
        NodeBundle {
            style: Style {
                display: Display::None,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Start,
                ..default()
            },
            background_color: Color::rgba(0.4, 0.4, 0.4, 0.2).into(),
            ..default()
        },
        DebugUi {},
    );

    let title_text = "Debugger UI";
    let title_bundle = (
        TextBundle::from_section(
            title_text,
            TextStyle {
                font_size: 22.0,
                color: Color::WHITE,
                ..default()
            },
        ) // Set the alignment of the Text
        .with_text_justify(JustifyText::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Relative,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        DebugUi {},
        DebugUiTitle {},
    );

    let fps_text = "FPS:";
    let fps_bundle = (
        TextBundle::from_section(
            fps_text,
            TextStyle {
                font_size: 16.0,
                color: Color::GRAY,
                ..default()
            }
        ),
        DebugUi {},
        DebugUiFps {},
    );

    let container_entity = commands.spawn(container_bundle).id();
    let title_entity = commands.spawn(title_bundle).id();
    let fps_entity = commands.spawn(fps_bundle).id();

    commands.entity(container_entity).push_children(&[title_entity, fps_entity]);
}

    // let square = NodeBundle {
    //     style: Style {
    //         width: Val::Px(300.),
    //         border: UiRect::all(Val::Px(2.)),
    //         ..default()
    //     },
    //     background_color: Color::rgba(0.65, 0.65, 0.65, 0.3).into(),
    //     ..default()
    // };

//     let _parent = commands.spawn((container, DebugUI {})).id();
//     // let child = commands.spawn(square).id();

//     // commands.entity(parent).push_children(&[child]);
// }


fn update_debugger(
    mut commands: Commands,
    mut style_query: Query<&mut Style, With<DebugUi>>,
    mut fps_query: Query<Entity, With<DebugUiFps>>,
    debug_data: Res<DebugData>,
    fps: Res<FpsResource<25>>,
) {
    let fps = if fps.average > f32::EPSILON {
        fps.average
    } else {
        0.0
    };

    if debug_data.is_changed() {
        for mut style in style_query.iter_mut() {
            if debug_data.is_visible {
                info!("Debugger is visible.");
                style.display = Display::DEFAULT;
            } else {
                info!("Debugger is hidden.");
                style.display = Display::None;
            }
        }
    }

    let fps_text = format!("FPS: {}", fps);
    for entity in fps_query.iter_mut() {
        commands.entity(entity).despawn_recursive();
        commands.spawn((
            TextBundle::from_section(
                fps_text.clone(),
                TextStyle {
                    font_size: 16.0,
                    color: Color::GRAY,
                    ..default()
                }
            ),
            DebugUi {},
            DebugUiFps {},
        ));
    }
}

// fn spawn_box(mut commands: Commands) {
//     let container = NodeBundle {
//         style: Style {
//             width: Val::Percent(100.0),
//             height: Val::Percent(100.0),
//             justify_content: JustifyContent::Center,
//             ..default()
//         },
//         ..default()
//     };

//     let square = NodeBundle {
//         style: Style {
//             width: Val::Px(200.),
//             border: UiRect::all(Val::Px(2.)),
//             ..default()
//         },
//         background_color: Color::rgb(0.65, 0.65, 0.65).into(),
//         ..default()
//     };

//     let parent = commands.spawn((container, DebugUI {})).id();
//     let child = commands.spawn(square).id();

//     commands.entity(parent).push_children(&[child]);
// }

// fn spawn_text(mut commands: Commands) {
//     let text = "Hello world!";

//     commands.spawn((
//         TextBundle::from_section(
//             text,
//             TextStyle {
//                 font_size: 100.0,
//                 color: Color::WHITE,
//                 ..default()
//             },
//         ) // Set the alignment of the Text
//         .with_text_justify(JustifyText::Center)
//         // Set the style of the TextBundle itself.
//         .with_style(Style {
//             position_type: PositionType::Absolute,
//             bottom: Val::Px(5.0),
//             right: Val::Px(5.0),
//             ..default()
//         }),
//         DebugUI {}
//     ));
// }
