//! Custom styles for UI elements

use iced::widget::{button, container, scrollable, text_input, pick_list, rule};
use iced::{Background, Border, Color, Shadow, Theme, Vector};
use super::theme::OxideColors;

/// Container style for the main content area
pub fn content_container(theme: &Theme) -> container::Style {
    let colors = get_theme_colors(theme);
    container::Style {
        background: Some(Background::Color(colors.background)),
        text_color: Some(colors.text_primary),
        ..Default::default()
    }
}

/// Container style for cards/panels
pub fn card_container(theme: &Theme) -> container::Style {
    let colors = get_theme_colors(theme);
    container::Style {
        background: Some(Background::Color(colors.surface)),
        border: Border {
            color: colors.border,
            width: 1.0,
            radius: 8.0.into(),
        },
        text_color: Some(colors.text_primary),
        ..Default::default()
    }
}

/// Container style for the sidebar
pub fn sidebar_container(theme: &Theme) -> container::Style {
    let colors = get_theme_colors(theme);
    container::Style {
        background: Some(Background::Color(colors.background_light)),
        text_color: Some(colors.text_primary),
        border: Border {
            color: colors.border,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Container style for the toolbar
pub fn toolbar_container(theme: &Theme) -> container::Style {
    let colors = get_theme_colors(theme);
    container::Style {
        background: Some(Background::Color(colors.background_light)),
        text_color: Some(colors.text_primary),
        border: Border {
            color: colors.border,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Primary button style
#[derive(Debug, Clone)]
pub struct OxideThemeWrapper(pub Theme);

impl Default for OxideThemeWrapper {
    fn default() -> Self {
        Self(Theme::Dark)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ButtonClass {
    #[default]
    Primary,
}

impl button::Catalog for OxideThemeWrapper {
    type Class<'a> = ButtonClass;

    fn default<'a>() -> Self::Class<'a> {
        ButtonClass::default()
    }

    fn style(&self, _class: &Self::Class<'_>, status: button::Status) -> button::Style {
        primary_button_style(&self.0, status)
    }
}

pub fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let colors = get_theme_colors(theme);
    let mut base = button::Style {
        background: Some(Background::Color(colors.primary)),
        text_color: Color::WHITE,
        border: Border {
            color: colors.primary,
            width: 0.0,
            radius: 6.0.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    };

    match status {
        button::Status::Hovered => {
            base.background = Some(Background::Color(colors.primary_hover));
        }
        button::Status::Pressed => {
            base.background = Some(Background::Color(colors.primary_pressed));
        }
        button::Status::Disabled => {
            base.background = Some(Background::Color(colors.text_muted));
        }
        _ => {}
    }

    base
}

/// Secondary/outline button style
pub fn secondary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let colors = get_theme_colors(theme);
    let mut base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: colors.text_primary,
        border: Border {
            color: colors.border,
            width: 1.0,
            radius: 6.0.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    };

    match status {
        button::Status::Hovered => {
            base.background = Some(Background::Color(colors.surface_hover));
        }
        button::Status::Pressed => {
            base.background = Some(Background::Color(colors.background_lighter));
        }
        button::Status::Disabled => {
            base.text_color = colors.text_muted;
        }
        _ => {}
    }

    base
}

/// Icon button style (transparent background)
pub fn icon_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let colors = get_theme_colors(theme);
    let mut base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: colors.text_secondary,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    };

    match status {
        button::Status::Hovered => {
            base.background = Some(Background::Color(colors.surface_hover));
            base.text_color = colors.text_primary;
        }
        button::Status::Pressed => {
            base.background = Some(Background::Color(colors.background_lighter));
        }
        _ => {}
    }

    base
}

/// Danger button style
pub fn danger_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let colors = get_theme_colors(theme);
    let mut base = button::Style {
        background: Some(Background::Color(colors.danger)),
        text_color: Color::WHITE,
        border: Border {
            color: colors.danger,
            width: 0.0,
            radius: 6.0.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    };

    match status {
        button::Status::Hovered => {
            base.background = Some(Background::Color(Color::from_rgb8(229, 57, 53)));
        }
        button::Status::Pressed => {
            base.background = Some(Background::Color(Color::from_rgb8(198, 40, 40)));
        }
        button::Status::Disabled => {
            base.background = Some(Background::Color(colors.text_muted));
        }
        _ => {}
    }

    base
}

/// Instance card button style
pub fn instance_card_style(theme: &Theme, status: button::Status, selected: bool) -> button::Style {
    let colors = get_theme_colors(theme);
    let mut base = button::Style {
        background: Some(Background::Color(colors.surface)),
        text_color: colors.text_primary,
        border: Border {
            color: if selected { colors.primary } else { colors.border },
            width: if selected { 2.0 } else { 1.0 },
            radius: 8.0.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    };

    match status {
        button::Status::Hovered => {
            base.background = Some(Background::Color(colors.surface_hover));
            base.border.color = colors.primary;
        }
        button::Status::Pressed => {
            base.background = Some(Background::Color(colors.background_lighter));
        }
        _ => {}
    }

    base
}

/// Text input style
pub fn text_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let colors = get_theme_colors(theme);
    let mut base = text_input::Style {
        background: Background::Color(colors.background_lighter),
        border: Border {
            color: colors.border,
            width: 1.0,
            radius: 6.0.into(),
        },
        icon: colors.text_muted,
        placeholder: colors.text_muted,
        value: colors.text_primary,
        selection: colors.selection,
    };

    match status {
        text_input::Status::Focused { .. } => {
            base.border.color = colors.primary;
        }
        text_input::Status::Hovered => {
            base.border.color = colors.border_light;
        }
        text_input::Status::Disabled => {
            base.background = Background::Color(colors.background);
            base.value = colors.text_muted;
        }
        _ => {}
    }

    base
}

/// Scrollable style
pub fn scrollable_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let colors = get_theme_colors(theme);
    
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(colors.scrollbar),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(colors.scrollbar),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                },
            },
        },
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(colors.surface),
            border: Border {
                color: colors.border,
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: Shadow::default(),
            icon: colors.text_primary,
        },
    }
}

/// Divider/rule style
pub fn divider_style(theme: &Theme) -> rule::Style {
    let colors = get_theme_colors(theme);
    rule::Style {
        color: colors.border,
        snap: true,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
    }
}

/// Helper to get theme colors based on current theme
fn get_theme_colors(theme: &Theme) -> OxideColors {
    // Check if it's a dark theme
    let palette = theme.palette();
    let is_dark = palette.background.r < 0.5;
    
    if is_dark {
        OxideColors::dark()
    } else {
        OxideColors::light()
    }
}
