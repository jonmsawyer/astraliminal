use bevy::prelude::*;

pub struct DebuggerFpsPlugin;

impl Plugin for DebuggerFpsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<FpsResource<25>>()
            .add_systems(Update, (
                update_fps,
            ));
    }
}

/// Resource for calculating our Frames Per Second
#[derive(Debug, Default, Copy, Clone, Resource)]
pub struct FpsResource<const N: usize> {
    /// Current average FPS.
    pub average: f32,

    /// Sum of per-frame time deltas.
    pub sum: f32,

    /// Current measurements count since last recalculation.
    pub count: usize,
}

impl<const N: usize> FpsResource<N> {
    /// Add a new time delta measurement.
    pub fn add(&mut self, delta: f32) {
        self.sum += delta;
        self.count = (self.count + 1) % N;

        if self.count == 0 {
            // Average delta would be sum/len, but we want average FPS which is the reciprocal
            self.average = N as f32 / self.sum;
            self.sum = 0.;
        }
    }
}

fn update_fps(
    mut fps: ResMut<FpsResource<25>>,
    time: Res<Time>,
) {
    fps.add(time.delta_seconds());
}
