//! Astraliminal game.

use bevy::prelude::App;

use astral_core::AstraliminalPlugins;

fn main() {
    let mut app = App::new();
    app.add_plugins(AstraliminalPlugins).run();
}
