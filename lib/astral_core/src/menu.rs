//! Astraliminal's Window plugin.

use bevy::{
    core::FrameCount,
    prelude::*,
    window::{Cursor, CursorGrabMode, Window, WindowMode, WindowPlugin, WindowResolution},
};

/// Astraliminal's version.
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct AstraliminalWindowPlugin;

impl Plugin for AstraliminalWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
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
        }))
        .add_systems(Update, make_visible);
    }
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