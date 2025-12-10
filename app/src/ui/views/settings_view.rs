//! Settings view - application settings configuration

use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, text,
    text_input, Space, rule,
};
use iced::{Alignment, Element, Length, Theme};
use crate::app::{Message, OxideLauncher, View, SettingsTab};
use crate::ui::styles::*;

/// Build the settings view
pub fn settings_view<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let content = column![
        // Header
        row![
            text("Settings").size(24),
            Space::new().width(Length::Fill),
        ],
        
        Space::new().height(16),
        
        // Settings tabs
        settings_tabs(app),
        
        Space::new().height(16),
        
        // Settings content
        scrollable(
            settings_content(app)
        )
        .height(Length::Fill),
        
        // Save button
        Space::new().height(16),
        row![
            Space::new().width(Length::Fill),
            button(text("Save Settings").size(14))
                .padding([10, 20])
                .style(|theme: &Theme, status: button::Status| primary_button_style(theme, status))
                .on_press(Message::SaveSettings),
        ],
    ]
    .spacing(8);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Build settings tabs
fn settings_tabs<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let tabs = vec![
        ("General", SettingsTab::General),
        ("Java", SettingsTab::Java),
        ("Memory", SettingsTab::Memory),
        ("Network", SettingsTab::Network),
        ("API Keys", SettingsTab::APIKeys),
        ("About", SettingsTab::About),
    ];

    let mut tab_row = row![].spacing(4);
    
    for (label, tab) in tabs {
        let is_selected = app.settings_tab == tab;
        let btn = button(text(label).size(14))
            .padding([8, 16])
            .style(move |theme: &Theme, status: button::Status| {
                if is_selected {
                    let mut style = secondary_button_style(theme, status);
                    style.background = Some(iced::Background::Color(iced::Color::from_rgba(0.3, 0.6, 0.3, 0.3)));
                    style
                } else {
                    icon_button_style(theme, status)
                }
            })
            .on_press(Message::SetSettingsTab(tab));
        tab_row = tab_row.push(btn);
    }

    container(tab_row)
        .width(Length::Fill)
        .style(|theme: &Theme| card_container(theme))
        .padding(8)
        .into()
}

/// Build settings content based on selected tab
fn settings_content<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let content: Element<'a, Message> = match app.settings_tab {
        SettingsTab::General => general_settings(app),
        SettingsTab::Java => java_settings(app),
        SettingsTab::Memory => memory_settings(app),
        SettingsTab::Network => network_settings(app),
        SettingsTab::APIKeys => api_keys_settings(app),
        SettingsTab::About => about_settings(app),
    };

    container(content)
        .width(Length::Fill)
        .style(|theme: &Theme| card_container(theme))
        .padding(16)
        .into()
}

/// General settings tab
fn general_settings<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    column![
        text("General Settings").size(18),
        Space::new().height(16),
        
        // Theme selection
        row![
            text("Theme:").size(14).width(Length::Fixed(200.0)),
            pick_list(
                vec!["Dark", "Light", "System"],
                Some(&app.settings_theme as &str),
                |theme| Message::SettingsThemeChanged(theme.to_string())
            )
            .width(Length::Fixed(150.0))
            .padding(8),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        
        Space::new().height(8),
        
        // Language selection
        row![
            text("Language:").size(14).width(Length::Fixed(200.0)),
            pick_list(
                vec!["English", "Spanish", "French", "German", "Japanese", "Chinese"],
                Some(&app.settings_language as &str),
                |lang| Message::SettingsLanguageChanged(lang.to_string())
            )
            .width(Length::Fixed(150.0))
            .padding(8),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        
        Space::new().height(16),
        rule::horizontal(1),
        Space::new().height(16),
        
        // Data directories
        text("Directories").size(16),
        Space::new().height(8),
        
        row![
            text("Data Directory:").size(14).width(Length::Fixed(200.0)),
            text_input("", &app.settings_data_dir)
                .on_input(Message::SettingsDataDirChanged)
                .padding(8)
                .width(Length::Fill),
            button(text("Browse").size(14))
                .padding([8, 12])
                .on_press(Message::BrowseDataDir),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        
        row![
            text("Instances Directory:").size(14).width(Length::Fixed(200.0)),
            text_input("", &app.settings_instances_dir)
                .on_input(Message::SettingsInstancesDirChanged)
                .padding(8)
                .width(Length::Fill),
            button(text("Browse").size(14))
                .padding([8, 12])
                .on_press(Message::BrowseInstancesDir),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        
        Space::new().height(16),
        rule::horizontal(1),
        Space::new().height(16),
        
        // Behavior options
        text("Behavior").size(16),
        Space::new().height(8),
        
        checkbox(app.settings_close_on_launch)
            .label("Close launcher when game starts")
            .on_toggle(Message::SettingsCloseOnLaunchChanged),
        
        checkbox(app.settings_show_console)
            .label("Show console when game is running")
            .on_toggle(Message::SettingsShowConsoleChanged),
        
        checkbox(app.settings_check_updates)
            .label("Check for updates on startup")
            .on_toggle(Message::SettingsCheckUpdatesChanged),
    ]
    .spacing(8)
    .into()
}

/// Java settings tab
fn java_settings<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    column![
        text("Java Settings").size(18),
        Space::new().height(16),
        
        // Auto-detect Java
        row![
            button(text("Auto-detect Java Installations").size(14))
                .padding([10, 20])
                .on_press(Message::AutoDetectJava),
        ],
        
        Space::new().height(16),
        rule::horizontal(1),
        Space::new().height(16),
        
        // Java path
        text("Default Java Installation").size(16),
        Space::new().height(8),
        
        row![
            text("Java Path:").size(14).width(Length::Fixed(150.0)),
            text_input("Auto-detect", &app.settings_java_path)
                .on_input(Message::SettingsJavaPathChanged)
                .padding(8)
                .width(Length::Fill),
            button(text("Browse").size(14))
                .padding([8, 12])
                .on_press(Message::BrowseJavaPath),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        
        Space::new().height(16),
        
        // Detected Java installations
        text("Detected Java Installations").size(16),
        Space::new().height(8),
        
        // TODO: List detected Java installations
        text("Click 'Auto-detect' to scan for Java installations")
            .size(14)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
        
        Space::new().height(16),
        rule::horizontal(1),
        Space::new().height(16),
        
        // Default JVM arguments
        text("Default JVM Arguments").size(16),
        Space::new().height(8),
        
        text_input("Enter default JVM arguments...", &app.settings_jvm_args)
            .on_input(Message::SettingsJvmArgsChanged)
            .padding(8)
            .width(Length::Fill),
        
        text("These arguments will be used for all instances unless overridden")
            .size(12)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
    ]
    .spacing(8)
    .into()
}

/// Memory settings tab
fn memory_settings<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    column![
        text("Memory Settings").size(18),
        Space::new().height(16),
        
        text("Default Memory Allocation").size(16),
        text("These values will be used for new instances")
            .size(12)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
        
        Space::new().height(16),
        
        row![
            text("Minimum RAM (MB):").size(14).width(Length::Fixed(200.0)),
            text_input("512", &app.settings_min_ram)
                .on_input(Message::SettingsMinRamChanged)
                .padding(8)
                .width(Length::Fixed(100.0)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        
        row![
            text("Maximum RAM (MB):").size(14).width(Length::Fixed(200.0)),
            text_input("4096", &app.settings_max_ram)
                .on_input(Message::SettingsMaxRamChanged)
                .padding(8)
                .width(Length::Fixed(100.0)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        
        Space::new().height(16),
        
        // System memory info
        text("System Memory Information").size(16),
        Space::new().height(8),
        
        // TODO: Get actual system memory
        text("Total RAM: 16 GB")
            .size(14)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
        text("Available RAM: 8 GB")
            .size(14)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
        
        Space::new().height(16),
        
        text("⚠ Allocating too much RAM can cause issues. Generally, 4-8 GB is sufficient for most modpacks.")
            .size(12)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.9, 0.7, 0.2))
            }),
    ]
    .spacing(8)
    .into()
}

/// Network settings tab
fn network_settings<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    column![
        text("Network Settings").size(18),
        Space::new().height(16),
        
        // Download settings
        text("Download Settings").size(16),
        Space::new().height(8),
        
        row![
            text("Concurrent Downloads:").size(14).width(Length::Fixed(200.0)),
            text_input("4", &app.settings_concurrent_downloads)
                .on_input(Message::SettingsConcurrentDownloadsChanged)
                .padding(8)
                .width(Length::Fixed(100.0)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        
        row![
            text("Download Timeout (seconds):").size(14).width(Length::Fixed(200.0)),
            text_input("30", &app.settings_download_timeout)
                .on_input(Message::SettingsDownloadTimeoutChanged)
                .padding(8)
                .width(Length::Fixed(100.0)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        
        Space::new().height(16),
        rule::horizontal(1),
        Space::new().height(16),
        
        // Proxy settings
        text("Proxy Settings").size(16),
        Space::new().height(8),
        
        checkbox(app.settings_use_proxy)
            .label("Use proxy")
            .on_toggle(Message::SettingsUseProxyChanged),
        
        row![
            text("Proxy Host:").size(14).width(Length::Fixed(200.0)),
            text_input("", &app.settings_proxy_host)
                .on_input(Message::SettingsProxyHostChanged)
                .padding(8)
                .width(Length::Fill),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        
        row![
            text("Proxy Port:").size(14).width(Length::Fixed(200.0)),
            text_input("", &app.settings_proxy_port)
                .on_input(Message::SettingsProxyPortChanged)
                .padding(8)
                .width(Length::Fixed(100.0)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(8)
    .into()
}

/// API Keys settings tab
fn api_keys_settings<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    column![
        text("API Keys").size(18),
        Space::new().height(16),
        
        text("Configure API keys for various services. These are optional but enable additional features.")
            .size(14)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
        
        Space::new().height(16),
        
        // Microsoft Azure Client ID
        text("Microsoft Authentication").size(16),
        Space::new().height(8),
        
        row![
            text("Azure Client ID:").size(14).width(Length::Fixed(200.0)),
            text_input("", &app.settings_msa_client_id)
                .on_input(Message::SettingsMsaClientIdChanged)
                .padding(8)
                .width(Length::Fill),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        
        text("Required for Microsoft account login. Get one from Azure Portal.")
            .size(12)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
        
        Space::new().height(16),
        rule::horizontal(1),
        Space::new().height(16),
        
        // CurseForge API Key
        text("CurseForge").size(16),
        Space::new().height(8),
        
        row![
            text("CurseForge API Key:").size(14).width(Length::Fixed(200.0)),
            text_input("", &app.settings_curseforge_api_key)
                .on_input(Message::SettingsCurseforgeApiKeyChanged)
                .padding(8)
                .width(Length::Fill)
                .secure(true),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        
        text("Required for CurseForge mod downloads. Get one from CurseForge for Studios.")
            .size(12)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
        
        Space::new().height(16),
        rule::horizontal(1),
        Space::new().height(16),
        
        // Modrinth API Token
        text("Modrinth").size(16),
        Space::new().height(8),
        
        row![
            text("Modrinth API Token:").size(14).width(Length::Fixed(200.0)),
            text_input("Optional", &app.settings_modrinth_api_token)
                .on_input(Message::SettingsModrinthApiTokenChanged)
                .padding(8)
                .width(Length::Fill)
                .secure(true),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        
        text("Optional. Provides higher rate limits and access to private projects.")
            .size(12)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
    ]
    .spacing(8)
    .into()
}

/// About settings tab
fn about_settings<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    column![
        text("About OxideLauncher").size(18),
        Space::new().height(16),
        
        container(
            column![
                text("OxideLauncher").size(32),
                text("A modern Minecraft launcher written in Rust").size(14),
            ]
            .align_x(Alignment::Center)
            .spacing(4)
        )
        .width(Length::Fill)
        .center_x(Length::Fill),
        
        Space::new().height(24),
        
        row![
            text("Version:").size(14).width(Length::Fixed(150.0)),
            text(env!("CARGO_PKG_VERSION")).size(14),
        ]
        .spacing(8),
        
        row![
            text("Build:").size(14).width(Length::Fixed(150.0)),
            text("Debug").size(14),
        ]
        .spacing(8),
        
        row![
            text("Rust Version:").size(14).width(Length::Fixed(150.0)),
            text("1.75+").size(14),
        ]
        .spacing(8),
        
        Space::new().height(16),
        rule::horizontal(1),
        Space::new().height(16),
        
        text("Credits").size(16),
        Space::new().height(8),
        
        text("Built with:").size(14),
        text("• Iced - A cross-platform GUI library for Rust").size(12),
        text("• Tokio - An asynchronous runtime for Rust").size(12),
        text("• Reqwest - An ergonomic HTTP client").size(12),
        text("• Serde - A serialization framework").size(12),
        
        Space::new().height(16),
        
        text("Inspired by Prism Launcher and MultiMC").size(12),
        
        Space::new().height(16),
        rule::horizontal(1),
        Space::new().height(16),
        
        row![
            button(text("GitHub").size(14))
                .padding([8, 16])
                .on_press(Message::OpenUrl("https://github.com".to_string())),
            button(text("Report Issue").size(14))
                .padding([8, 16])
                .on_press(Message::OpenUrl("https://github.com/issues".to_string())),
            button(text("Check for Updates").size(14))
                .padding([8, 16])
                .on_press(Message::CheckForUpdates),
        ]
        .spacing(8),
    ]
    .spacing(8)
    .into()
}
