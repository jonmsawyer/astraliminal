//! Astraliminal library.

mod window;

pub mod prelude {
    use super::*;
    pub use bevy::prelude::*;
    pub use window::AstraliminalWindowPlugin;
}

use prelude::*;

pub struct AstraliminalPlugins;

impl Plugin for AstraliminalPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((AstraliminalWindowPlugin,));
    }
}
