//! Main window view - the primary layout of the application

use iced::widget::{
    button, column, container, row, scrollable, text, text_input, 
    Space,
};
use iced::{Alignment, Element, Length, Theme};
use crate::app::{Message, OxideLauncher, View};
use crate::ui::styles::*;

/// Build the main view layout
pub fn main_view<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let content = column![
        // Top toolbar
        toolbar(app),
        // Main content area with sidebar and instance grid
        row![
            // Sidebar (navigation and actions)
            sidebar(app),
            // Main content (instance grid or other views)
            main_content(app),
        ]
        .height(Length::Fill),
    ];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Build the top toolbar
fn toolbar<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let toolbar_row = row![
        // Left side - navigation buttons
        row![
            toolbar_button("Add Instance", Message::OpenCreateInstance),
            toolbar_button("Folders", Message::OpenFolders),
        ]
        .spacing(4),
        
        Space::new().width(Length::Fill),
        
        // Center - search bar
        text_input("Search instances...", &app.search_query)
            .on_input(Message::SearchChanged)
            .padding(8)
            .width(Length::Fixed(300.0)),
        
        Space::new().width(Length::Fill),
        
        // Right side - account and settings
        row![
            account_button(app),
            toolbar_button("Settings", Message::Navigate(View::Settings)),
            toolbar_button("Help", Message::OpenHelp),
        ]
        .spacing(4),
    ]
    .spacing(8)
    .padding(8)
    .align_y(Alignment::Center);

    container(toolbar_row)
        .width(Length::Fill)
        .style(|theme: &Theme| toolbar_container(theme))
        .into()
}

/// Build a toolbar button
fn toolbar_button<'a>(label: &'a str, message: Message) -> Element<'a, Message> {
    button(
        text(label).size(14)
    )
    .padding([6, 12])
    .on_press(message)
    .into()
}

/// Build the account button with current account name
fn account_button<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let account_text = app.accounts.get_active()
        .map(|a| a.username.clone())
        .unwrap_or_else(|| "No Account".to_string());
    
    button(
        row![
            text("👤").size(14),
            text(account_text).size(14),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
    )
    .padding([6, 12])
    .on_press(Message::Navigate(View::Accounts))
    .into()
}

/// Build the left sidebar
fn sidebar<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let sidebar_content = column![
        // Navigation section
        sidebar_section("Navigation", vec![
            sidebar_item("Instances", View::Instances, app.current_view == View::Instances),
            sidebar_item("Browse", View::Browse, app.current_view == View::Browse),
            sidebar_item("Accounts", View::Accounts, app.current_view == View::Accounts),
            sidebar_item("Settings", View::Settings, app.current_view == View::Settings),
        ]),
        
        Space::new().height(Length::Fill),
        
        // Instance groups section
        sidebar_section("Groups", vec![
            sidebar_group_item("All Instances", app.instances.count()),
        ]),
        
        Space::new().height(Length::Fill),
        
        // Quick actions
        sidebar_section("Actions", vec![
            sidebar_action("Add Instance", Message::OpenCreateInstance),
            sidebar_action("Check Updates", Message::CheckForUpdates),
        ]),
    ]
    .spacing(8)
    .padding(12);

    container(
        scrollable(sidebar_content)
            .height(Length::Fill)
    )
    .width(Length::Fixed(200.0))
    .height(Length::Fill)
    .style(|theme: &Theme| sidebar_container(theme))
    .into()
}

/// Build a sidebar section with title and items
fn sidebar_section<'a>(title: &'a str, items: Vec<Element<'a, Message>>) -> Element<'a, Message> {
    let mut content = column![
        text(title)
            .size(12)
            .style(|theme: &Theme| {
                let palette = theme.palette();
                text::Style { color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6)) }
            }),
    ]
    .spacing(4);
    
    for item in items {
        content = content.push(item);
    }
    
    content.into()
}

/// Build a sidebar navigation item
fn sidebar_item<'a>(label: &'a str, view: View, selected: bool) -> Element<'a, Message> {
    let style = if selected {
        |theme: &Theme, status: button::Status| {
            let mut style = secondary_button_style(theme, status);
            style.background = Some(iced::Background::Color(iced::Color::from_rgba(0.3, 0.6, 0.3, 0.3)));
            style
        }
    } else {
        |theme: &Theme, status: button::Status| icon_button_style(theme, status)
    };
    
    button(
        text(label).size(14)
    )
    .width(Length::Fill)
    .padding([8, 12])
    .style(style)
    .on_press(Message::Navigate(view))
    .into()
}

/// Build a sidebar group item with count
fn sidebar_group_item<'a>(label: &'a str, count: usize) -> Element<'a, Message> {
    button(
        row![
            text(label).size(14),
            Space::new().width(Length::Fill),
            text(count.to_string()).size(12),
        ]
        .align_y(Alignment::Center)
    )
    .width(Length::Fill)
    .padding([8, 12])
    .style(|theme: &Theme, status: button::Status| icon_button_style(theme, status))
    .on_press(Message::FilterByGroup(label.to_string()))
    .into()
}

/// Build a sidebar action button
fn sidebar_action<'a>(label: &'a str, message: Message) -> Element<'a, Message> {
    button(
        text(label).size(14)
    )
    .width(Length::Fill)
    .padding([8, 12])
    .style(|theme: &Theme, status: button::Status| icon_button_style(theme, status))
    .on_press(message)
    .into()
}

/// Build the main content area
fn main_content<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let content: Element<'a, Message> = match app.current_view {
        View::Instances => instances_grid(app),
        View::Browse => crate::ui::views::browse_view::browse_view(app),
        View::Accounts => crate::ui::views::accounts_view::accounts_view(app),
        View::Settings => crate::ui::views::settings_view::settings_view(app),
        View::InstanceDetail(ref id) => {
            if let Some(instance) = app.instances.get(id) {
                crate::ui::views::instance_view::instance_detail_view(instance, app)
            } else {
                text("Instance not found").into()
            }
        }
        View::CreateInstance => crate::ui::views::create_instance_view::create_instance_view(app),
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(16)
        .style(|theme: &Theme| content_container(theme))
        .into()
}

/// Build the instances grid view
fn instances_grid<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let instances = app.instances.get_filtered(&app.search_query);
    
    if instances.is_empty() {
        return container(
            column![
                text("No instances found").size(20),
                Space::new().height(8),
                text("Click 'Add Instance' to create one.").size(14),
                Space::new().height(16),
                button(text("Add Instance"))
                    .padding([10, 20])
                    .on_press(Message::OpenCreateInstance),
            ]
            .align_x(Alignment::Center)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();
    }

    // Build grid of instance cards
    let mut grid_rows: Vec<Element<Message>> = Vec::new();
    let mut current_row: Vec<Element<Message>> = Vec::new();
    let cards_per_row = 4;

    for (idx, instance) in instances.iter().enumerate() {
        let is_selected = app.selected_instance.as_ref() == Some(&instance.id);
        let card = instance_card(instance, is_selected);
        current_row.push(card);

        if current_row.len() >= cards_per_row || idx == instances.len() - 1 {
            // Fill remaining slots with empty space if needed
            while current_row.len() < cards_per_row {
                current_row.push(Space::new().width(Length::FillPortion(1)).into());
            }
            
            let row_element: Element<Message> = row(current_row.drain(..).collect::<Vec<_>>())
                .spacing(16)
                .into();
            grid_rows.push(row_element);
        }
    }

    scrollable(
        column(grid_rows)
            .spacing(16)
            .width(Length::Fill)
    )
    .height(Length::Fill)
    .into()
}

/// Build an instance card
fn instance_card<'a>(instance: &'a crate::core::instance::Instance, selected: bool) -> Element<'a, Message> {
    let mod_loader_text = match &instance.mod_loader {
        Some(loader) => loader.loader_type.name(),
        None => "Vanilla",
    };

    let card_content = column![
        // Instance icon placeholder
        container(
            text("🎮").size(40)
        )
        .width(Length::Fill)
        .height(Length::Fixed(80.0))
        .center_x(Length::Fill)
        .center_y(Length::Fixed(80.0)),
        
        // Instance name
        text(&instance.name)
            .size(16)
            .width(Length::Fill),
        
        // Version and mod loader
        row![
            text(&instance.minecraft_version).size(12),
            text(" • ").size(12),
            text(mod_loader_text).size(12),
        ],
        
        // Play time
        text(format_play_time(instance.total_played_seconds))
            .size(11)
            .style(|_theme: &Theme| text::Style { 
                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
            }),
    ]
    .spacing(4)
    .padding(12)
    .width(Length::Fill);

    let card_button = button(card_content)
        .width(Length::FillPortion(1))
        .style(move |theme: &Theme, status: button::Status| instance_card_style(theme, status, selected))
        .on_press(Message::SelectInstance(instance.id.clone()));

    container(card_button)
        .width(Length::FillPortion(1))
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
        format!("{}h {}m played", hours, minutes)
    } else {
        format!("{}m played", minutes)
    }
}
