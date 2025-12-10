//! Instance detail view - shows details and options for a specific instance

use iced::widget::{
    button, column, container, row, scrollable, text, text_input,
    Space, rule,
};
use iced::{Alignment, Element, Length, Theme};
use crate::app::{Message, OxideLauncher, View, InstanceTab};
use crate::core::instance::{Instance, ModLoader};
use crate::ui::styles::*;

/// Build the instance detail view
pub fn instance_detail_view<'a>(instance: &'a Instance, app: &'a OxideLauncher) -> Element<'a, Message> {
    let mod_loader_text = match &instance.mod_loader {
        Some(loader) => format!("{} {}", loader.loader_type.name(), loader.version),
        None => "Vanilla".to_string(),
    };

    let content = column![
        // Header with back button and instance name
        row![
            button(text("← Back").size(14))
                .padding([8, 16])
                .on_press(Message::Navigate(View::Instances)),
            Space::new().width(Length::Fill),
            text(&instance.name).size(24),
            Space::new().width(Length::Fill),
            button(text("▶ Launch").size(14))
                .padding([10, 20])
                .style(|theme: &Theme, status: button::Status| primary_button_style(theme, status))
                .on_press(Message::LaunchInstance(instance.id.clone())),
        ]
        .align_y(Alignment::Center)
        .padding(8),
        
        rule::horizontal(1),
        Space::new().height(16),
        
        // Main content
        scrollable(
            row![
                // Left side - instance info
                column![
                    // Instance icon and basic info
                    container(
                        column![
                            container(
                                text("🎮").size(80)
                            )
                            .width(Length::Fill)
                            .center_x(Length::Fill),
                            
                            Space::new().height(16),
                            
                            text(&instance.name).size(20),
                            text(format!("Minecraft {}", &instance.minecraft_version)).size(14),
                              text(mod_loader_text).size(14),                            Space::new().height(8),
                            
                            text(format_play_time(instance.total_played_seconds))
                                .size(12)
                                .style(|_theme: &Theme| iced::widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
                                }),
                            
                            if let Some(last_played) = &instance.last_played {
                                text(format!("Last played: {}", last_played.format("%Y-%m-%d %H:%M")))
                                    .size(12)
                                    .style(|_theme: &Theme| iced::widget::text::Style {
                                        color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
                                    })
                            } else {
                                text("Never played")
                                    .size(12)
                                    .style(|_theme: &Theme| iced::widget::text::Style {
                                        color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
                                    })
                            },
                        ]
                        .spacing(4)
                        .align_x(Alignment::Center)
                        .padding(16)
                    )
                    .style(|theme: &Theme| card_container(theme)),
                    
                    Space::new().height(16),
                    
                    // Quick actions
                    container(
                        column![
                            text("Quick Actions").size(16),
                            Space::new().height(8),
                            action_button("Edit Instance", Message::EditInstance(instance.id.clone())),
                            action_button("Open Folder", Message::OpenInstanceFolder(instance.id.clone())),
                            action_button("Copy Instance", Message::CopyInstance(instance.id.clone())),
                            action_button("Export Instance", Message::ExportInstance(instance.id.clone())),
                            Space::new().height(8),
                            button(text("Delete Instance").size(14))
                                .width(Length::Fill)
                                .padding([8, 12])
                                .style(|theme: &Theme, status: button::Status| danger_button_style(theme, status))
                                .on_press(Message::DeleteInstance(instance.id.clone())),
                        ]
                        .spacing(4)
                        .padding(16)
                    )
                    .style(|theme: &Theme| card_container(theme)),
                ]
                .width(Length::Fixed(280.0))
                .spacing(8),
                
                Space::new().width(16),
                
                // Right side - tabs for content
                column![
                    // Tab bar
                    instance_tabs(app),
                    
                    Space::new().height(8),
                    
                    // Tab content
                    instance_tab_content(instance, app),
                ]
                .width(Length::Fill),
            ]
        )
        .height(Length::Fill),
    ];

    container(content.spacing(8))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Build action button for the sidebar
fn action_button<'a>(label: &'a str, message: Message) -> Element<'a, Message> {
    button(text(label).size(14))
        .width(Length::Fill)
        .padding([8, 12])
        .style(|theme: &Theme, status: button::Status| secondary_button_style(theme, status))
        .on_press(message)
        .into()
}

/// Build the instance detail tabs
fn instance_tabs<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let tabs = vec![
        ("Overview", InstanceTab::Overview),
        ("Mods", InstanceTab::Mods),
        ("Resource Packs", InstanceTab::ResourcePacks),
        ("Shader Packs", InstanceTab::ShaderPacks),
        ("Worlds", InstanceTab::Worlds),
        ("Screenshots", InstanceTab::Screenshots),
        ("Notes", InstanceTab::Notes),
        ("Settings", InstanceTab::Settings),
    ];

    let mut tab_row = row![].spacing(4);
    
    for (label, tab) in tabs {
        let is_selected = app.instance_tab == tab;
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
            .on_press(Message::SetInstanceTab(tab));
        tab_row = tab_row.push(btn);
    }

    container(tab_row)
        .width(Length::Fill)
        .style(|theme: &Theme| card_container(theme))
        .padding(8)
        .into()
}

/// Build the content for the current tab
fn instance_tab_content<'a>(instance: &'a Instance, app: &'a OxideLauncher) -> Element<'a, Message> {
    let content: Element<'a, Message> = match app.instance_tab {
        InstanceTab::Overview => overview_tab(instance),
        InstanceTab::Mods => mods_tab(instance, app),
        InstanceTab::ResourcePacks => resource_packs_tab(instance),
        InstanceTab::ShaderPacks => shader_packs_tab(instance),
        InstanceTab::Worlds => worlds_tab(instance),
        InstanceTab::Screenshots => screenshots_tab(instance),
        InstanceTab::Notes => notes_tab(instance, app),
        InstanceTab::Settings => settings_tab(instance, app),
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| card_container(theme))
        .padding(16)
        .into()
}

/// Overview tab content
fn overview_tab<'a>(instance: &'a Instance) -> Element<'a, Message> {
    let mod_loader_str = format!("{:?}", instance.mod_loader);
    let path_str = instance.path.to_string_lossy().to_string();
    
    column![
        text("Instance Overview").size(18),
        Space::new().height(16),
        
        info_row("Name".to_string(), instance.name.clone()),
        info_row("Minecraft Version".to_string(), instance.minecraft_version.clone()),
        info_row("Mod Loader".to_string(), mod_loader_str),
        info_row("Instance ID".to_string(), instance.id.clone()),
        info_row("Path".to_string(), path_str),
        
        Space::new().height(16),
        
        text("Statistics").size(16),
        Space::new().height(8),
        info_row("Total Play Time".to_string(), format_play_time(instance.total_played_seconds)),
        info_row("Last Played".to_string(), instance.last_played
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "Never".to_string())),
        info_row("Created".to_string(), instance.created_at.format("%Y-%m-%d %H:%M").to_string()),
    ]
    .spacing(8)
    .into()
}

/// Info row helper
fn info_row<'a>(label: String, value: String) -> Element<'a, Message> {
    row![
        text(format!("{}:", label))
            .size(14)
            .width(Length::Fixed(150.0)),
        text(value).size(14),
    ]
    .spacing(8)
    .into()
}

/// Mods tab content
fn mods_tab<'a>(instance: &'a Instance, app: &'a OxideLauncher) -> Element<'a, Message> {
    column![
        row![
            text("Mods").size(18),
            Space::new().width(Length::Fill),
            button(text("Add Mods").size(14))
                .padding([8, 16])
                .on_press(Message::Navigate(View::Browse)),
            button(text("Open Folder").size(14))
                .padding([8, 16])
                .style(|theme: &Theme, status: button::Status| secondary_button_style(theme, status))
                .on_press(Message::OpenInstanceFolder(instance.id.clone())),
        ]
        .align_y(Alignment::Center),
        
        Space::new().height(16),
        
        // TODO: List actual mods
        text("No mods installed")
            .size(14)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
    ]
    .spacing(8)
    .into()
}

/// Resource packs tab content
fn resource_packs_tab<'a>(instance: &'a Instance) -> Element<'a, Message> {
    column![
        row![
            text("Resource Packs").size(18),
            Space::new().width(Length::Fill),
            button(text("Add Resource Pack").size(14))
                .padding([8, 16])
                .on_press(Message::AddResourcePack(instance.id.clone())),
        ]
        .align_y(Alignment::Center),
        
        Space::new().height(16),
        
        text("No resource packs installed")
            .size(14)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
    ]
    .spacing(8)
    .into()
}

/// Shader packs tab content
fn shader_packs_tab<'a>(instance: &'a Instance) -> Element<'a, Message> {
    column![
        row![
            text("Shader Packs").size(18),
            Space::new().width(Length::Fill),
            button(text("Add Shader Pack").size(14))
                .padding([8, 16])
                .on_press(Message::AddShaderPack(instance.id.clone())),
        ]
        .align_y(Alignment::Center),
        
        Space::new().height(16),
        
        text("No shader packs installed")
            .size(14)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
    ]
    .spacing(8)
    .into()
}

/// Worlds tab content
fn worlds_tab<'a>(instance: &'a Instance) -> Element<'a, Message> {
    column![
        row![
            text("Worlds").size(18),
            Space::new().width(Length::Fill),
            button(text("Add World").size(14))
                .padding([8, 16])
                .on_press(Message::AddWorld(instance.id.clone())),
        ]
        .align_y(Alignment::Center),
        
        Space::new().height(16),
        
        text("No worlds found")
            .size(14)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
    ]
    .spacing(8)
    .into()
}

/// Screenshots tab content
fn screenshots_tab<'a>(instance: &'a Instance) -> Element<'a, Message> {
    column![
        row![
            text("Screenshots").size(18),
            Space::new().width(Length::Fill),
            button(text("Open Folder").size(14))
                .padding([8, 16])
                .style(|theme: &Theme, status: button::Status| secondary_button_style(theme, status))
                .on_press(Message::OpenScreenshotsFolder(instance.id.clone())),
        ]
        .align_y(Alignment::Center),
        
        Space::new().height(16),
        
        text("No screenshots found")
            .size(14)
            .style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
    ]
    .spacing(8)
    .into()
}

/// Notes tab content
fn notes_tab<'a>(instance: &'a Instance, app: &'a OxideLauncher) -> Element<'a, Message> {
    column![
        text("Instance Notes").size(18),
        Space::new().height(16),
        
        text_input("Add notes about this instance...", &app.instance_notes)
            .on_input(Message::InstanceNotesChanged)
            .padding(12)
            .width(Length::Fill),
    ]
    .spacing(8)
    .into()
}

/// Settings tab content  
fn settings_tab<'a>(instance: &'a Instance, app: &'a OxideLauncher) -> Element<'a, Message> {
    column![
        text("Instance Settings").size(18),
        Space::new().height(16),
        
        // Memory settings
        text("Memory").size(16),
        Space::new().height(8),
        
        row![
            text("Minimum RAM (MB):").size(14).width(Length::Fixed(150.0)),
            text_input("", &app.instance_min_ram)
                .on_input(Message::InstanceMinRamChanged)
                .padding(8)
                .width(Length::Fixed(100.0)),
        ]
        .spacing(8),
        
        row![
            text("Maximum RAM (MB):").size(14).width(Length::Fixed(150.0)),
            text_input("", &app.instance_max_ram)
                .on_input(Message::InstanceMaxRamChanged)
                .padding(8)
                .width(Length::Fixed(100.0)),
        ]
        .spacing(8),
        
        Space::new().height(16),
        
        // Java settings
        text("Java").size(16),
        Space::new().height(8),
        
        row![
            text("Java Path:").size(14).width(Length::Fixed(150.0)),
            text_input("Use default", &app.instance_java_path)
                .on_input(Message::InstanceJavaPathChanged)
                .padding(8)
                .width(Length::Fill),
            button(text("Browse").size(14))
                .padding([8, 12])
                .on_press(Message::BrowseJavaPath),
        ]
        .spacing(8),
        
        row![
            text("JVM Arguments:").size(14).width(Length::Fixed(150.0)),
            text_input("", &app.instance_jvm_args)
                .on_input(Message::InstanceJvmArgsChanged)
                .padding(8)
                .width(Length::Fill),
        ]
        .spacing(8),
        
        Space::new().height(16),
        
        // Game settings
        text("Game").size(16),
        Space::new().height(8),
        
        row![
            text("Game Resolution:").size(14).width(Length::Fixed(150.0)),
            text_input("Width", &app.instance_resolution_width)
                .on_input(Message::InstanceResolutionWidthChanged)
                .padding(8)
                .width(Length::Fixed(80.0)),
            text("x").size(14),
            text_input("Height", &app.instance_resolution_height)
                .on_input(Message::InstanceResolutionHeightChanged)
                .padding(8)
                .width(Length::Fixed(80.0)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        
        Space::new().height(16),
        
        button(text("Save Settings").size(14))
            .padding([10, 20])
            .style(|theme: &Theme, status: button::Status| primary_button_style(theme, status))
            .on_press(Message::SaveInstanceSettings),
    ]
    .spacing(8)
    .into()
}

/// Format play time in a human-readable format
fn format_play_time(seconds: u64) -> String {
    if seconds == 0 {
        return "Never played".to_string();
    }
    
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
