//! Browse view - search and download mods, modpacks, resource packs from Modrinth/CurseForge

use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_input,
    Space,
};
use iced::{Alignment, Element, Length, Theme};
use crate::app::{Message, OxideLauncher, BrowseResourceType};
use crate::core::modplatform::types::*;
use crate::ui::styles::*;

/// Build the browse view
pub fn browse_view<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let content = column![
        // Header
        row![
            text("Browse").size(24),
            Space::new().width(Length::Fill),
        ],
        
        Space::new().height(16),
        
        // Search bar and filters
        search_bar(app),
        
        Space::new().height(16),
        
        // Resource type tabs
        resource_type_tabs(app),
        
        Space::new().height(16),
        
        // Results or loading state
        if app.browse_loading {
            loading_view()
        } else if app.browse_results.is_empty() {
            if app.browse_search_query.is_empty() {
                empty_search_view()
            } else {
                no_results_view()
            }
        } else {
            results_grid(app)
        },
    ]
    .spacing(8);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Search bar with filters
fn search_bar<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let content = row![
        // Search input
        text_input("Search mods, modpacks, resource packs...", &app.browse_search_query)
            .on_input(Message::BrowseSearchChanged)
            .on_submit(Message::BrowseSearch)
            .padding(12)
            .width(Length::Fill),
        
        // Platform filter
        pick_list(
            vec!["All Platforms", "Modrinth", "CurseForge"],
            Some(&app.browse_platform_filter as &str),
            |platform| Message::BrowsePlatformChanged(platform.to_string())
        )
        .width(Length::Fixed(150.0))
        .padding(8),
        
        // Minecraft version filter
        pick_list(
            get_minecraft_versions(),
            app.browse_version_filter.clone(),
            |version| Message::BrowseVersionChanged(Some(version))
        )
        .width(Length::Fixed(120.0))
        .padding(8)
        .placeholder("Version"),
        
        // Mod loader filter
        pick_list(
            vec!["All Loaders", "Forge", "Fabric", "Quilt", "NeoForge"],
            Some(&app.browse_loader_filter as &str),
            |loader| Message::BrowseLoaderChanged(loader.to_string())
        )
        .width(Length::Fixed(120.0))
        .padding(8),
        
        // Sort order
        pick_list(
            vec!["Relevance", "Downloads", "Follows", "Newest", "Updated"],
            Some(&app.browse_sort_order as &str),
            |sort| Message::BrowseSortChanged(sort.to_string())
        )
        .width(Length::Fixed(120.0))
        .padding(8),
        
        // Search button
        button(text("Search").size(14))
            .padding([12, 20])
            .style(|theme: &Theme, status: button::Status| primary_button_style(theme, status))
            .on_press(Message::BrowseSearch),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .style(|theme: &Theme| card_container(theme))
        .padding(12)
        .into()
}

/// Get available Minecraft versions
fn get_minecraft_versions() -> Vec<String> {
    vec![
        "1.21.4".to_string(),
        "1.21.3".to_string(),
        "1.21.2".to_string(),
        "1.21.1".to_string(),
        "1.21".to_string(),
        "1.20.6".to_string(),
        "1.20.4".to_string(),
        "1.20.2".to_string(),
        "1.20.1".to_string(),
        "1.20".to_string(),
        "1.19.4".to_string(),
        "1.19.3".to_string(),
        "1.19.2".to_string(),
        "1.19.1".to_string(),
        "1.19".to_string(),
        "1.18.2".to_string(),
        "1.18.1".to_string(),
        "1.18".to_string(),
        "1.17.1".to_string(),
        "1.16.5".to_string(),
        "1.12.2".to_string(),
        "1.8.9".to_string(),
        "1.7.10".to_string(),
    ]
}

/// Resource type tabs
fn resource_type_tabs<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let tabs = vec![
        ("Mods", BrowseResourceType::Mods),
        ("Modpacks", BrowseResourceType::Modpacks),
        ("Resource Packs", BrowseResourceType::ResourcePacks),
        ("Shader Packs", BrowseResourceType::ShaderPacks),
    ];

    let mut tab_row = row![].spacing(4);
    
    for (label, resource_type) in tabs {
        let is_selected = app.browse_resource_type == resource_type;
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
            .on_press(Message::BrowseResourceTypeChanged(resource_type));
        tab_row = tab_row.push(btn);
    }

    container(tab_row)
        .width(Length::Fill)
        .into()
}

/// Loading view
fn loading_view<'a>() -> Element<'a, Message> {
    container(
        column![
            text("🔄").size(48),
            Space::new().height(8),
            text("Searching...").size(18),
        ]
        .align_x(Alignment::Center)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// Empty search view
fn empty_search_view<'a>() -> Element<'a, Message> {
    container(
        column![
            text("🔍").size(48),
            Space::new().height(8),
            text("Start searching").size(20),
            Space::new().height(8),
            text("Search for mods, modpacks, resource packs, and more from Modrinth and CurseForge.").size(14),
        ]
        .align_x(Alignment::Center)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// No results view
fn no_results_view<'a>() -> Element<'a, Message> {
    container(
        column![
            text("😕").size(48),
            Space::new().height(8),
            text("No results found").size(20),
            Space::new().height(8),
            text("Try adjusting your search terms or filters.").size(14),
        ]
        .align_x(Alignment::Center)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// Results grid
fn results_grid<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let mut items: Vec<Element<Message>> = Vec::new();
    
    for result in &app.browse_results {
        items.push(result_card(result, app));
    }
    
    let results_column = column(items)
        .spacing(12)
        .width(Length::Fill);
    
    column![
        // Results count and pagination
        row![
            text(format!("Showing {} results", app.browse_results.len())).size(14),
            Space::new().width(Length::Fill),
            if app.browse_page > 0 {
                button(text("← Previous").size(14))
                    .padding([8, 12])
                    .on_press(Message::BrowsePreviousPage)
            } else {
                button(text("← Previous").size(14))
                    .padding([8, 12])
            },
            button(text("Next →").size(14))
                .padding([8, 12])
                .on_press(Message::BrowseNextPage),
        ]
        .align_y(Alignment::Center),
        
        Space::new().height(8),
        
        scrollable(results_column)
            .height(Length::Fill),
    ]
    .spacing(8)
    .into()
}

/// Result card
fn result_card<'a>(result: &'a SearchHit, app: &'a OxideLauncher) -> Element<'a, Message> {
    let platform_icon = match result.platform {
        Platform::Modrinth => "🟢",
        Platform::CurseForge => "🟠",
    };
    
    let card_content = row![
        // Icon placeholder
        container(
            text("📦").size(32)
        )
        .width(Length::Fixed(64.0))
        .height(Length::Fixed(64.0))
        .center_x(Length::Fixed(64.0))
        .center_y(Length::Fixed(64.0))
        .style(|theme: &Theme| card_container(theme)),
        
        Space::new().width(16),
        
        // Info
        column![
            row![
                text(&result.title).size(18),
                Space::new().width(8),
                text(platform_icon).size(14),
            ],
            text(format!("by {}", result.author)).size(12),
            text(&result.description).size(14),
            row![
                text(format!("📥 {}", format_downloads(result.downloads))).size(12),
                Space::new().width(16),
                text(format!("❤ {}", result.follows)).size(12),
                Space::new().width(16),
                text(format!("Updated: {}", result.date_modified.format("%Y-%m-%d"))).size(12),
            ],
            row![
                text("Versions: ").size(11),
                text(result.versions.iter().take(5).cloned().collect::<Vec<_>>().join(", "))
                    .size(11)
                    .style(|_theme: &Theme| iced::widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
                    }),
            ],
        ]
        .spacing(2)
        .width(Length::Fill),
        
        Space::new().width(16),
        
        // Actions
        column![
            button(text("View").size(14))
                .padding([8, 16])
                .style(|theme: &Theme, status: button::Status| primary_button_style(theme, status))
                .on_press(Message::ViewProject(result.id.clone(), result.platform)),
            
            if let Some(instance_id) = &app.selected_instance {
                button(text("Install").size(14))
                    .padding([8, 16])
                    .on_press(Message::InstallToInstance(result.id.clone(), result.platform, instance_id.clone()))
            } else {
                button(text("Install").size(14))
                    .padding([8, 16])
                    .style(|theme: &Theme, status: button::Status| secondary_button_style(theme, status))
                    .on_press(Message::ShowInstallDialog(result.id.clone(), result.platform))
            },
        ]
        .spacing(8)
        .align_x(Alignment::End),
    ]
    .align_y(Alignment::Center)
    .padding(16);

    container(card_content)
        .width(Length::Fill)
        .style(|theme: &Theme| card_container(theme))
        .into()
}

/// Format download count
fn format_downloads(downloads: u64) -> String {
    if downloads >= 1_000_000 {
        format!("{:.1}M", downloads as f64 / 1_000_000.0)
    } else if downloads >= 1_000 {
        format!("{:.1}K", downloads as f64 / 1_000.0)
    } else {
        downloads.to_string()
    }
}
