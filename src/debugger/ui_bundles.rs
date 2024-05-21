use bevy::prelude::*;

use super::{
    ui_components::{
        DebugUiCharacterLookingAt, DebugUiCharacterPosition, DebugUiContainer, DebugUiDirection,
        DebugUiFps, DebugUiIsGrounded, DebugUiIsUpsideDown, DebugUiNode, DebugUiText, DebugUiTitle,
    },
    DebugUiTextStyle,
};

#[derive(Debug, Default, Bundle)]
pub struct DebugUiContainerBundle {
    pub node: NodeBundle,
    pub ui_component: DebugUiContainer,
}

impl DebugUiContainerBundle {
    pub fn new(style: Option<Style>, background_color: Option<Color>) -> Self {
        let style = style.unwrap_or(Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        });

        let background_color: Color =
            background_color.unwrap_or(Color::rgba(0.15, 0.15, 0.15, 0.5));

        Self {
            node: NodeBundle {
                style,
                background_color: background_color.into(),
                z_index: ZIndex::Global(30),
                ..default()
            },
            ui_component: DebugUiContainer,
        }
    }
}

#[derive(Debug, Default, Bundle)]
pub struct DebugUiNodeBundle {
    pub node: NodeBundle,
    pub ui_component: DebugUiNode,
}

impl DebugUiNodeBundle {
    pub fn new(style: Option<Style>, background_color: Option<Color>) -> Self {
        let style = style.unwrap_or(Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        });

        let background_color: BackgroundColor = background_color
            .unwrap_or(Color::rgba(0.0, 0.0, 0.0, 0.0))
            .into();

        Self {
            node: NodeBundle {
                style,
                background_color,
                z_index: ZIndex::Global(30),
                ..default()
            },
            ui_component: DebugUiNode,
        }
    }
}

#[derive(Debug, Default, Bundle)]
pub struct DebugUiTitleBundle {
    pub node: TextBundle,
    pub ui_component: DebugUiTitle,
    pub label: Label,
}

impl DebugUiTitleBundle {
    pub fn new(text_style: Option<TextStyle>) -> DebugUiTitleBundle {
        let text = "Astraliminal Debugger and Statistics";
        let text_style = text_style.unwrap_or(TextStyle {
            font_size: 32.0,
            color: Color::GRAY,
            ..default()
        });
        Self {
            node: TextBundle::from_section(text, text_style)
                // .with_background_color(Color::rgba(0.1, 0.1, 0.1, 0.4)),
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                }),
            ui_component: DebugUiTitle,
            label: Label,
        }
    }
}

#[derive(Debug, Default, Bundle)]
pub struct DebugUiTextBundle {
    pub node: TextBundle,
    pub ui_component: DebugUiText,
    pub label: Label,
}

impl DebugUiTextBundle {
    pub fn new(text: String, text_style: Option<TextStyle>) -> DebugUiTextBundle {
        let text_style = text_style.unwrap_or(DebugUiTextStyle::default());
        Self {
            node: TextBundle::from_section(text, text_style),
            // .with_background_color(Color::rgba(0.8, 0.1, 0.1, 0.2)),
            // .with_style(Style {
            //     width: Val::Percent(50.0),
            //     ..default()
            // }),
            ui_component: DebugUiText,
            label: Label,
        }
    }
}

#[derive(Debug, Default, Bundle)]
pub struct DebugUiFpsBundle {
    pub node: TextBundle,
    pub ui_component: DebugUiFps,
    pub label: Label,
}

impl DebugUiFpsBundle {
    pub fn new(fps: f32, text_style: Option<TextStyle>) -> DebugUiFpsBundle {
        let text = format!("FPS: {}", fps);
        let text_style = text_style.unwrap_or(DebugUiTextStyle::default());
        Self {
            node: TextBundle::from_section(text, text_style),
            // .with_background_color(Color::rgba(0.8, 0.1, 0.1, 0.2)),
            // .with_style(Style {
            //     width: Val::Percent(50.0),
            //     ..default()
            // }),
            ui_component: DebugUiFps,
            label: Label,
        }
    }
}

#[derive(Debug, Default, Bundle)]
pub struct DebugUiDirectionBundle {
    pub node: TextBundle,
    pub ui_component: DebugUiDirection,
    pub label: Label,
}

impl DebugUiDirectionBundle {
    pub fn new(direction: Vec2, text_style: Option<TextStyle>) -> Self {
        let text = format!("WASD Direction: x={}, z={}", direction.x, direction.y);
        let text_style = text_style.unwrap_or(DebugUiTextStyle::default());
        Self {
            node: TextBundle::from_section(text, text_style),
            // .with_background_color(Color::rgba(0.8, 0.1, 0.1, 0.2)),
            // .with_style(Style {
            //     width: Val::Percent(50.0),
            //     ..default()
            // }),
            ui_component: DebugUiDirection,
            label: Label,
        }
    }
}

#[derive(Debug, Default, Bundle)]
pub struct DebugUiIsUpsideDownBundle {
    pub node: TextBundle,
    pub ui_component: DebugUiIsUpsideDown,
    pub label: Label,
}

impl DebugUiIsUpsideDownBundle {
    pub fn new(upside_down: (bool, Vec2), text_style: Option<TextStyle>) -> Self {
        let text = format!(
            "Is Upside Down?: {}\nRotation Y: {:?}",
            upside_down.0, upside_down.1
        );
        let text_style = text_style.unwrap_or(DebugUiTextStyle::default());
        Self {
            node: TextBundle::from_section(text, text_style),
            // .with_background_color(Color::rgba(0.8, 0.1, 0.1, 0.2)),
            // .with_style(Style {
            //     width: Val::Percent(50.0),
            //     ..default()
            // }),
            ui_component: DebugUiIsUpsideDown,
            label: Label,
        }
    }
}

#[derive(Debug, Default, Bundle)]
pub struct DebugUiIsGroundedBundle {
    pub node: TextBundle,
    pub ui_component: DebugUiIsGrounded,
    pub label: Label,
}

impl DebugUiIsGroundedBundle {
    pub fn new(is_grounded: bool, text_style: Option<TextStyle>) -> Self {
        let text = format!("Is Grounded?: {}", is_grounded);
        let text_style = text_style.unwrap_or(DebugUiTextStyle::default());
        Self {
            node: TextBundle::from_section(text, text_style),
            // .with_background_color(Color::rgba(0.8, 0.1, 0.1, 0.2)),
            // .with_style(Style {
            //     width: Val::Percent(50.0),
            //     ..default()
            // }),
            ui_component: DebugUiIsGrounded,
            label: Label,
        }
    }
}

#[derive(Debug, Default, Bundle)]
pub struct DebugUiCharacterPositionBundle {
    pub node: TextBundle,
    pub ui_component: DebugUiCharacterPosition,
    pub label: Label,
}

impl DebugUiCharacterPositionBundle {
    pub fn new(character_position: Vec3, text_style: Option<TextStyle>) -> Self {
        let text = format!(
            "Position: x={}, y={}, z={}",
            character_position.x, character_position.y, character_position.z,
        );
        let text_style = text_style.unwrap_or(DebugUiTextStyle::default());
        Self {
            node: TextBundle::from_section(text, text_style),
            // .with_background_color(Color::rgba(0.8, 0.1, 0.1, 0.2)),
            // .with_style(Style {
            //     width: Val::Percent(50.0),
            //     ..default()
            // }),
            ui_component: DebugUiCharacterPosition,
            label: Label,
        }
    }
}

#[derive(Debug, Default, Bundle)]
pub struct DebugUiCharacterLookingAtBundle {
    pub node: TextBundle,
    pub ui_component: DebugUiCharacterLookingAt,
    pub label: Label,
}

impl DebugUiCharacterLookingAtBundle {
    pub fn new(character_looking_at: Vec3, text_style: Option<TextStyle>) -> Self {
        let text = format!(
            "Looking At Coord: x={}, y={}, z={}",
            character_looking_at.x, character_looking_at.y, character_looking_at.z,
        );
        let text_style = text_style.unwrap_or(DebugUiTextStyle::default());
        Self {
            node: TextBundle::from_section(text, text_style),
            // .with_background_color(Color::rgba(0.8, 0.1, 0.1, 0.2)),
            // .with_style(Style {
            //     width: Val::Percent(50.0),
            //     ..default()
            // }),
            ui_component: DebugUiCharacterLookingAt,
            label: Label,
        }
    }
}
