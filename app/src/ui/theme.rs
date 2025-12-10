//! Application theme configuration

use iced::Color;
use iced::Theme;
use iced::theme::Palette;

/// Theme variant enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OxideTheme {
    #[default]
    Dark,
    Light,
}

impl OxideTheme {
    /// Convert to Iced Theme
    pub fn to_iced_theme(&self) -> Theme {
        match self {
            OxideTheme::Dark => oxide_theme(true),
            OxideTheme::Light => oxide_theme(false),
        }
    }
}

/// OxideLauncher color palette
pub struct OxideColors {
    // Primary colors
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_pressed: Color,
    
    // Background colors
    pub background: Color,
    pub background_light: Color,
    pub background_lighter: Color,
    pub surface: Color,
    pub surface_hover: Color,
    
    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    
    // Accent colors
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub info: Color,
    
    // Border colors
    pub border: Color,
    pub border_light: Color,
    
    // Special colors
    pub selection: Color,
    pub scrollbar: Color,
}

impl OxideColors {
    /// Dark theme colors (default)
    pub fn dark() -> Self {
        Self {
            // Primary (green accent like Prism)
            primary: Color::from_rgb8(76, 175, 80),
            primary_hover: Color::from_rgb8(102, 187, 106),
            primary_pressed: Color::from_rgb8(56, 142, 60),
            
            // Backgrounds
            background: Color::from_rgb8(30, 30, 30),
            background_light: Color::from_rgb8(40, 40, 40),
            background_lighter: Color::from_rgb8(50, 50, 50),
            surface: Color::from_rgb8(45, 45, 45),
            surface_hover: Color::from_rgb8(55, 55, 55),
            
            // Text
            text_primary: Color::from_rgb8(255, 255, 255),
            text_secondary: Color::from_rgb8(200, 200, 200),
            text_muted: Color::from_rgb8(140, 140, 140),
            
            // Accents
            success: Color::from_rgb8(76, 175, 80),
            warning: Color::from_rgb8(255, 152, 0),
            danger: Color::from_rgb8(244, 67, 54),
            info: Color::from_rgb8(33, 150, 243),
            
            // Borders
            border: Color::from_rgb8(60, 60, 60),
            border_light: Color::from_rgb8(80, 80, 80),
            
            // Special
            selection: Color::from_rgba8(76, 175, 80, 0.3),
            scrollbar: Color::from_rgb8(70, 70, 70),
        }
    }
    
    /// Light theme colors
    pub fn light() -> Self {
        Self {
            // Primary (green accent)
            primary: Color::from_rgb8(76, 175, 80),
            primary_hover: Color::from_rgb8(56, 142, 60),
            primary_pressed: Color::from_rgb8(46, 125, 50),
            
            // Backgrounds
            background: Color::from_rgb8(250, 250, 250),
            background_light: Color::from_rgb8(245, 245, 245),
            background_lighter: Color::from_rgb8(255, 255, 255),
            surface: Color::from_rgb8(255, 255, 255),
            surface_hover: Color::from_rgb8(245, 245, 245),
            
            // Text
            text_primary: Color::from_rgb8(33, 33, 33),
            text_secondary: Color::from_rgb8(97, 97, 97),
            text_muted: Color::from_rgb8(158, 158, 158),
            
            // Accents
            success: Color::from_rgb8(76, 175, 80),
            warning: Color::from_rgb8(245, 124, 0),
            danger: Color::from_rgb8(211, 47, 47),
            info: Color::from_rgb8(25, 118, 210),
            
            // Borders
            border: Color::from_rgb8(224, 224, 224),
            border_light: Color::from_rgb8(189, 189, 189),
            
            // Special
            selection: Color::from_rgba8(76, 175, 80, 0.2),
            scrollbar: Color::from_rgb8(200, 200, 200),
        }
    }
}

/// Create the Iced theme for OxideLauncher
pub fn oxide_theme(dark: bool) -> Theme {
    let colors = if dark {
        OxideColors::dark()
    } else {
        OxideColors::light()
    };
    
    Theme::custom(
        "Oxide".to_string(),
        Palette {
            background: colors.background,
            text: colors.text_primary,
            primary: colors.primary,
            success: colors.success,
            danger: colors.danger,
            warning: Color::from_rgb(0.9, 0.7, 0.2),
        }
    )
}

/// Get the current theme colors
pub fn get_colors(dark: bool) -> OxideColors {
    if dark {
        OxideColors::dark()
    } else {
        OxideColors::light()
    }
}
