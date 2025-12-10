//! Create instance view - wizard for creating new instances

use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, text,
    text_input, Space, rule,
};
use iced::{Alignment, Element, Length, Theme};
use crate::app::{Message, OxideLauncher, View, CreateInstanceStep};
use crate::core::instance::ModLoader;
use crate::ui::styles::*;

/// Build the create instance view
pub fn create_instance_view<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let content = column![
        // Header
        row![
            button(text("← Back").size(14))
                .padding([8, 16])
                .on_press(Message::Navigate(View::Instances)),
            Space::new().width(Length::Fill),
            text("Create New Instance").size(24),
            Space::new().width(Length::Fill),
        ]
        .align_y(Alignment::Center),
        
        Space::new().height(16),
        
        // Step indicator
        step_indicator(app.create_instance_step),
        
        Space::new().height(16),
        
        // Step content
        scrollable(
            match app.create_instance_step {
                CreateInstanceStep::BasicInfo => step_name_and_group(app),
                CreateInstanceStep::Version => step_version_selection(app),
                CreateInstanceStep::ModLoader => step_mod_loader(app),
                CreateInstanceStep::Settings => step_options(app),
            }
        )
        .height(Length::Fill),
        
        Space::new().height(16),
        
        // Navigation buttons
        row![
            if !matches!(app.create_instance_step, CreateInstanceStep::BasicInfo) {
                button(text("← Previous").size(14))
                    .padding([10, 20])
                    .style(|theme: &Theme, status: button::Status| secondary_button_style(theme, status))
                    .on_press(Message::CreateInstancePreviousStep)
            } else {
                button(text("Cancel").size(14))
                    .padding([10, 20])
                    .style(|theme: &Theme, status: button::Status| secondary_button_style(theme, status))
                    .on_press(Message::Navigate(View::Instances))
            },
            Space::new().width(Length::Fill),
            if !matches!(app.create_instance_step, CreateInstanceStep::Settings) {
                button(text("Next →").size(14))
                    .padding([10, 20])
                    .style(|theme: &Theme, status: button::Status| primary_button_style(theme, status))
                    .on_press(Message::CreateInstanceNextStep)
            } else {
                button(text("Create Instance").size(14))
                    .padding([10, 20])
                    .style(|theme: &Theme, status: button::Status| primary_button_style(theme, status))
                    .on_press(Message::CreateInstance)
            },
        ],
    ]
    .spacing(8)
    .padding(16);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Step indicator
fn step_indicator<'a>(current_step: CreateInstanceStep) -> Element<'a, Message> {
    let steps = vec![
        ("Name", CreateInstanceStep::BasicInfo),
        ("Version", CreateInstanceStep::Version),
        ("Mod Loader", CreateInstanceStep::ModLoader),
        ("Options", CreateInstanceStep::Settings),
    ];
    
    let mut step_row = row![].spacing(4).align_y(Alignment::Center);
    
    for (idx, (step_name, step_kind)) in steps.iter().enumerate() {
        let is_active = *step_kind == current_step;
        let is_completed = steps.iter().position(|(_, s)| *s == current_step).map(|pos| idx < pos).unwrap_or(false);
        
        let step_color = if is_active {
            iced::Color::from_rgb(0.3, 0.7, 0.3)
        } else if is_completed {
            iced::Color::from_rgb(0.3, 0.6, 0.3)
        } else {
            iced::Color::from_rgb(0.5, 0.5, 0.5)
        };
        
        let step_text = if is_completed {
            format!("✓ {}", step_name)
        } else {
            format!("{}. {}", idx + 1, step_name)
        };
        
        let step_element = container(
            text(step_text)
                .size(14)
                .style(move |_theme: &Theme| iced::widget::text::Style {
                    color: Some(step_color)
                })
        )
        .padding([8, 16])
        .style(move |theme: &Theme| {
            let mut style = container::Style::default();
            if is_active {
                style.border.color = step_color;
                style.border.width = 1.0;
                style.border.radius = 4.0.into();
            }
            style
        });
        
        step_row = step_row.push(step_element);
        
        if idx < steps.len() - 1 {
            step_row = step_row.push(
                text("→")
                    .size(14)
                    .style(|_theme: &Theme| iced::widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5))
                    })
            );
        }
    }
    
    container(step_row.spacing(12))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
}

/// Step 1: Name and group
fn step_name_and_group<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    container(
        column![
            text("Instance Name & Group").size(18),
            Space::new().height(16),
            
            text("Instance Name").size(14),
            text_input("My Instance", &app.create_instance_name)
                .on_input(Message::CreateInstanceNameChanged)
                .padding(12)
                .width(Length::Fill),
            
            Space::new().height(8),
            
            text("Group (optional)").size(14),
            row![
                text_input("No Group", &app.create_instance_group)
                    .on_input(Message::CreateInstanceGroupChanged)
                    .padding(12)
                    .width(Length::Fill),
                pick_list(
                    get_existing_groups(app),
                    None::<String>,
                    |group| Message::CreateInstanceGroupChanged(group)
                )
                .width(Length::Fixed(150.0))
                .padding(8)
                .placeholder("Existing Groups"),
            ]
            .spacing(8),
            
            Space::new().height(16),
            
            text("Icon (optional)").size(14),
            row![
                button(text("Choose Icon").size(14))
                    .padding([8, 16])
                    .on_press(Message::CreateInstanceChooseIcon),
                if let Some(icon) = &app.create_instance_icon {
                    text(icon).size(12)
                } else {
                    text("No icon selected").size(12)
                },
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8)
        .padding(16)
    )
    .width(Length::Fill)
    .style(|theme: &Theme| card_container(theme))
    .into()
}

/// Get existing instance groups
fn get_existing_groups(app: &OxideLauncher) -> Vec<String> {
    let mut groups: Vec<String> = app.instances.iter()
        .filter_map(|i| i.group.clone())
        .collect();
    groups.sort();
    groups.dedup();
    groups
}

/// Step 2: Version selection
fn step_version_selection<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let versions = get_minecraft_versions();
    
    container(
        column![
            text("Minecraft Version").size(18),
            Space::new().height(16),
            
            // Version filter
                row![
                    checkbox(app.create_instance_show_releases)
                        .label("Releases")
                        .on_toggle(Message::CreateInstanceShowReleasesChanged),
                    checkbox(app.create_instance_show_snapshots)
                        .label("Snapshots")
                        .on_toggle(Message::CreateInstanceShowSnapshotsChanged),
                    checkbox(app.create_instance_show_old)
                        .label("Old Versions")
                        .on_toggle(Message::CreateInstanceShowOldChanged),
                ]
            .spacing(16),
            
            Space::new().height(8),
            
            // Search
            text_input("Search versions...", &app.create_instance_version_search)
                .on_input(Message::CreateInstanceVersionSearchChanged)
                .padding(12)
                .width(Length::Fill),
            
            Space::new().height(8),
            
            // Version list
            scrollable(
                version_list(&versions, &app.create_instance_version)
            )
            .height(Length::Fixed(300.0)),
        ]
        .spacing(8)
        .padding(16)
    )
    .width(Length::Fill)
    .style(|theme: &Theme| card_container(theme))
    .into()
}

/// Get available Minecraft versions
fn get_minecraft_versions() -> Vec<MinecraftVersionInfo> {
    // TODO: Fetch from version manifest
    vec![
        MinecraftVersionInfo { id: "1.21.4".to_string(), release_type: "release".to_string(), release_time: "2024-12-03".to_string() },
        MinecraftVersionInfo { id: "1.21.3".to_string(), release_type: "release".to_string(), release_time: "2024-10-22".to_string() },
        MinecraftVersionInfo { id: "1.21.2".to_string(), release_type: "release".to_string(), release_time: "2024-10-22".to_string() },
        MinecraftVersionInfo { id: "1.21.1".to_string(), release_type: "release".to_string(), release_time: "2024-08-08".to_string() },
        MinecraftVersionInfo { id: "1.21".to_string(), release_type: "release".to_string(), release_time: "2024-06-13".to_string() },
        MinecraftVersionInfo { id: "1.20.6".to_string(), release_type: "release".to_string(), release_time: "2024-04-29".to_string() },
        MinecraftVersionInfo { id: "1.20.4".to_string(), release_type: "release".to_string(), release_time: "2023-12-07".to_string() },
        MinecraftVersionInfo { id: "1.20.2".to_string(), release_type: "release".to_string(), release_time: "2023-09-21".to_string() },
        MinecraftVersionInfo { id: "1.20.1".to_string(), release_type: "release".to_string(), release_time: "2023-06-12".to_string() },
        MinecraftVersionInfo { id: "1.20".to_string(), release_type: "release".to_string(), release_time: "2023-06-07".to_string() },
        MinecraftVersionInfo { id: "1.19.4".to_string(), release_type: "release".to_string(), release_time: "2023-03-14".to_string() },
        MinecraftVersionInfo { id: "1.19.2".to_string(), release_type: "release".to_string(), release_time: "2022-08-05".to_string() },
        MinecraftVersionInfo { id: "1.18.2".to_string(), release_type: "release".to_string(), release_time: "2022-02-28".to_string() },
        MinecraftVersionInfo { id: "1.17.1".to_string(), release_type: "release".to_string(), release_time: "2021-07-06".to_string() },
        MinecraftVersionInfo { id: "1.16.5".to_string(), release_type: "release".to_string(), release_time: "2021-01-15".to_string() },
        MinecraftVersionInfo { id: "1.12.2".to_string(), release_type: "release".to_string(), release_time: "2017-09-18".to_string() },
        MinecraftVersionInfo { id: "1.8.9".to_string(), release_type: "release".to_string(), release_time: "2015-12-09".to_string() },
        MinecraftVersionInfo { id: "1.7.10".to_string(), release_type: "release".to_string(), release_time: "2014-06-26".to_string() },
    ]
}

/// Minecraft version info
struct MinecraftVersionInfo {
    id: String,
    release_type: String,
    release_time: String,
}

/// Version list
fn version_list<'a>(versions: &[MinecraftVersionInfo], selected: &str) -> Element<'a, Message> {
    let mut items: Vec<Element<Message>> = Vec::new();
    
    for version in versions {
        let is_selected = version.id == selected;
        let version_id = version.id.clone();
        let release_type = version.release_type.clone();
        let release_time = version.release_time.clone();
        let item = button(
            row![
                text(version_id.clone()).size(14),
                Space::new().width(Length::Fill),
                text(release_type).size(12).style(|_theme: &Theme| iced::widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
                }),
                Space::new().width(16),
                text(release_time).size(12).style(|_theme: &Theme| iced::widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
                }),
            ]
            .align_y(Alignment::Center)
        )
        .width(Length::Fill)
        .padding([12, 16])
        .style(move |theme: &Theme, status: button::Status| {
            if is_selected {
                let mut style = secondary_button_style(theme, status);
                style.background = Some(iced::Background::Color(iced::Color::from_rgba(0.3, 0.6, 0.3, 0.3)));
                style.border.color = iced::Color::from_rgb(0.3, 0.6, 0.3);
                style
            } else {
                icon_button_style(theme, status)
            }
        })
        .on_press(Message::CreateInstanceVersionSelected(version_id));
        
        items.push(item.into());
    }
    
    column(items)
        .spacing(2)
        .width(Length::Fill)
        .into()
}

/// Step 3: Mod loader selection
fn step_mod_loader<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    container(
        column![
            text("Mod Loader").size(18),
            Space::new().height(16),
            
            text("Choose a mod loader for your instance (optional)").size(14),
            
            Space::new().height(16),
            
            // Mod loader options
            mod_loader_option("Vanilla".to_string(), "No mod loader - play vanilla Minecraft".to_string(), 
                app.create_instance_mod_loader == "vanilla", "vanilla"),
            mod_loader_option("Forge".to_string(), "The most popular mod loader with extensive mod support".to_string(), 
                app.create_instance_mod_loader == "forge", "forge"),
            mod_loader_option("Fabric".to_string(), "Lightweight and fast mod loader with modern APIs".to_string(), 
                app.create_instance_mod_loader == "fabric", "fabric"),
            mod_loader_option("Quilt".to_string(), "Fork of Fabric with additional features".to_string(), 
                app.create_instance_mod_loader == "quilt", "quilt"),
            mod_loader_option("NeoForge".to_string(), "Community fork of Forge with active development".to_string(), 
                app.create_instance_mod_loader == "neoforge", "neoforge"),
            
            // Mod loader version (if not vanilla)
            if app.create_instance_mod_loader != "vanilla" {
                column![
                    Space::new().height(16),
                    text(format!("{} Version", capitalize(&app.create_instance_mod_loader))).size(14),
                        pick_list(
                            get_loader_versions(&app.create_instance_mod_loader, &app.create_instance_version),
                            Some(app.create_instance_loader_version.clone()),
                            |version| Message::CreateInstanceLoaderVersionChanged(version)
                        )
                    .width(Length::Fixed(200.0))
                    .padding(8)
                    .placeholder("Select version"),
                ]
                .spacing(8)
            } else {
                column![]
            },
        ]
        .spacing(8)
        .padding(16)
    )
    .width(Length::Fill)
    .style(|theme: &Theme| card_container(theme))
    .into()
}

/// Mod loader option button
fn mod_loader_option<'a>(name: String, description: String, selected: bool, loader_id: &str) -> Element<'a, Message> {
    let loader_id_owned = loader_id.to_string();
    button(
        column![
            row![
                text(name).size(16),
                Space::new().width(Length::Fill),
                if selected {
                    text("✓").size(16).style(|_theme: &Theme| iced::widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.3, 0.7, 0.3))
                    })
                } else {
                    text("")
                },
            ],
            text(description).size(12).style(|_theme: &Theme| iced::widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
        ]
        .spacing(4)
    )
    .width(Length::Fill)
    .padding([12, 16])
    .style(move |theme: &Theme, status: button::Status| {
        if selected {
            let mut style = secondary_button_style(theme, status);
            style.background = Some(iced::Background::Color(iced::Color::from_rgba(0.3, 0.6, 0.3, 0.2)));
            style.border.color = iced::Color::from_rgb(0.3, 0.6, 0.3);
            style
        } else {
            secondary_button_style(theme, status)
        }
    })
    .on_press(Message::CreateInstanceModLoaderChanged(loader_id_owned))
    .into()
}

/// Get available loader versions
fn get_loader_versions(_loader: &str, _mc_version: &str) -> Vec<String> {
    // TODO: Fetch actual versions from APIs
    vec![
        "Latest".to_string(),
        "Recommended".to_string(),
    ]
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Step 4: Additional options
fn step_options<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    container(
        column![
            text("Additional Options").size(18),
            Space::new().height(16),
            
            // Memory settings
            text("Memory").size(16),
            Space::new().height(8),
            
            row![
                text("Minimum RAM (MB):").size(14).width(Length::Fixed(200.0)),
                text_input("512", &app.create_instance_min_ram)
                    .on_input(Message::CreateInstanceMinRamChanged)
                    .padding(8)
                    .width(Length::Fixed(100.0)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            
            row![
                text("Maximum RAM (MB):").size(14).width(Length::Fixed(200.0)),
                text_input("4096", &app.create_instance_max_ram)
                    .on_input(Message::CreateInstanceMaxRamChanged)
                    .padding(8)
                    .width(Length::Fixed(100.0)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            
            Space::new().height(16),
            rule::horizontal(1),
            Space::new().height(16),
            
            // Java settings
            text("Java").size(16),
            Space::new().height(8),
            
            row![
                text("Java Path:").size(14).width(Length::Fixed(200.0)),
                text_input("Use default", &app.create_instance_java_path)
                    .on_input(Message::CreateInstanceJavaPathChanged)
                    .padding(8)
                    .width(Length::Fill),
                button(text("Browse").size(14))
                    .padding([8, 12])
                    .on_press(Message::CreateInstanceBrowseJava),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            
            Space::new().height(16),
            rule::horizontal(1),
            Space::new().height(16),
            
            // Window settings
            text("Window").size(16),
            Space::new().height(8),
            
            row![
                text("Resolution:").size(14).width(Length::Fixed(200.0)),
                text_input("854", &app.create_instance_resolution_width)
                    .on_input(Message::CreateInstanceResolutionWidthChanged)
                    .padding(8)
                    .width(Length::Fixed(80.0)),
                text("x").size(14),
                text_input("480", &app.create_instance_resolution_height)
                    .on_input(Message::CreateInstanceResolutionHeightChanged)
                    .padding(8)
                    .width(Length::Fixed(80.0)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8)
        .padding(16)
    )
    .width(Length::Fill)
    .style(|theme: &Theme| card_container(theme))
    .into()
}

/// Step 5: Confirmation
fn step_confirm<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let mod_loader_text = match app.create_instance_mod_loader.as_str() {
        "vanilla" => "Vanilla".to_string(),
        loader => format!("{} {}", capitalize(loader), &app.create_instance_loader_version),
    };
    
    container(
        column![
            text("Review & Create").size(18),
            Space::new().height(16),
            
            text("Please review your instance settings:").size(14),
            
            Space::new().height(16),
            
            info_row("Name".to_string(), app.create_instance_name.clone()),
            info_row("Group".to_string(), if app.create_instance_group.is_empty() { "None".to_string() } else { app.create_instance_group.clone() }),
            info_row("Minecraft Version".to_string(), app.create_instance_version.clone()),
            info_row("Mod Loader".to_string(), mod_loader_text),
            info_row("Memory".to_string(), format!("{} - {} MB", app.create_instance_min_ram, app.create_instance_max_ram)),
            if app.create_instance_java_path.is_empty() {
                info_row("Java".to_string(), "Default".to_string())
            } else {
                info_row("Java".to_string(), app.create_instance_java_path.clone())
            },
            info_row("Resolution".to_string(), format!("{}x{}", app.create_instance_resolution_width, app.create_instance_resolution_height)),
            
            Space::new().height(24),
            
            text("Click 'Create Instance' to create your new instance.")
                .size(14)
                .style(|_theme: &Theme| iced::widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
                }),
        ]
        .spacing(8)
        .padding(16)
    )
    .width(Length::Fill)
    .style(|theme: &Theme| card_container(theme))
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
