//! Astraliminal's Window plugin.

use bevy::{
    app::AppExit,
    core::FrameCount,
    prelude::*,
    window::{Cursor, CursorGrabMode, Window, WindowMode, WindowPlugin, WindowResolution},
};

/// Astraliminal's version.
const ASTRAL_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Astraliminal's compile datetime. See `../build.rs`.
const ASTRAL_COMPILE_DATETIME: &str = env!("ASTRAL_COMPILE_DATETIME");
/// Default window height
const WINDOW_HEIGHT: f32 = 768.0;
/// Default window width
const WINDOW_WIDTH: f32 = 1024.0;
/// Default window scale factor.
const WINDOW_SCALE_FACTOR: f32 = 1.0;

pub struct AstraliminalWindowPlugin;

impl Plugin for AstraliminalWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    cursor: Cursor {
                        visible: false,
                        grab_mode: CursorGrabMode::None,
                        ..default()
                    },
                    title: format!(
                        "Astralimimnal v{}{}",
                        ASTRAL_VERSION, ASTRAL_COMPILE_DATETIME
                    ),
                    name: Some("astraliminal.app".into()),
                    resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT)
                        .with_scale_factor_override(WINDOW_SCALE_FACTOR),
                    mode: WindowMode::Windowed,
                    resizable: false,
                    focused: true,
                    // This will spawn an invisible window.
                    // The window will be made visible in the `make_visible` system after 5 frames.
                    // This is useful when you want to avoid the white window that shows up before
                    // the GPU is ready to render the app.
                    visible: false,
                    ..default()
                }),
                ..default()
            }),
        )
        .add_systems(Update, (make_visible, keyboard_input));
    }
}

/// Make the window visible. Depends on the frame count when starting up. This is to get
/// rid of the annoying white window at startup.
fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    // The delay may be different for your app or system.
    if frames.0 == 5 {
        // At this point the gpu is ready to show the app so we can make the window visible.
        // Alternatively, you could toggle the visibility in Startup.
        // It will work, but it will have one white frame before it starts rendering.
        window.single_mut().visible = true;
    }
}

// TODO: Make the `keyboard_input` system only available when the main game is running.
//       Refactor this later when there are menus and app context in place.

/// Process keyboard input for the main window.
fn keyboard_input(
    key_code: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: ResMut<Events<AppExit>>,
) {
    // TODO: pull key code from config.
    if key_code.any_pressed([KeyCode::Escape]) {
        app_exit_events.send(AppExit);
    }
}
