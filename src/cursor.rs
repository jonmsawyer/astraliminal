use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cursor);
        #[cfg(target_os = "windows")]
        app.add_systems(Update, cursor_recenter);
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct GameCursor {}

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
        GameCursor {},
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
fn cursor_recenter(mut q_windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = q_windows.single_mut();
    let center = Vec2::new(primary_window.width() / 2.0, primary_window.height() / 2.0);
    primary_window.set_cursor_position(Some(center));
}
