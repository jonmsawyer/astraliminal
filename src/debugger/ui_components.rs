use bevy::prelude::*;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Component)]
pub struct DebugUiContainer;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Component)]
pub struct DebugUiNode;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Component)]
pub struct DebugUiText;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Component)]
pub struct DebugUiTitle;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct DebugUiFps;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct DebugUiDirection;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct DebugUiIsGrounded;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct DebugUiCharacterPosition;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct DebugUiCharacterLookingAt;
