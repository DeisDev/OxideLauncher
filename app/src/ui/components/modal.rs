//! Modal dialog component

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length, Theme};
use crate::app::Message;
use crate::ui::styles::*;

/// Modal configuration
pub struct ModalConfig {
    pub title: String,
    pub message: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub confirm_message: Message,
    pub cancel_message: Message,
    pub is_danger: bool,
}

/// Build a modal dialog
pub fn modal<'a>(config: ModalConfig) -> Element<'a, Message> {
    let is_danger = config.is_danger;
    let dialog = container(
        column![
            text(config.title).size(20),
            Space::new().height(16),
            
            text(config.message).size(14),
            
            Space::new().height(24),
            
            row![
                Space::new().width(Length::Fill),
                button(text(config.cancel_text).size(14))
                    .padding([8, 16])
                    .style(|theme: &Theme, status: button::Status| secondary_button_style(theme, status))
                    .on_press(config.cancel_message),
                button(text(config.confirm_text).size(14))
                    .padding([8, 16])
                    .style(move |theme: &Theme, status: button::Status| {
                        if is_danger {
                            danger_button_style(theme, status)
                        } else {
                            primary_button_style(theme, status)
                        }
                    })
                    .on_press(config.confirm_message),
            ]
            .spacing(8),
        ]
        .spacing(4)
        .padding(24)
        .width(Length::Fixed(400.0))
    )
    .style(|theme: &Theme| card_container(theme));

    // Overlay
    container(
        container(dialog)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
        ..Default::default()
    })
    .into()
}

/// Confirm delete modal
pub fn confirm_delete_modal<'a>(item_name: String, confirm_message: Message, cancel_message: Message) -> Element<'a, Message> {
    modal(ModalConfig {
        title: "Confirm Delete".to_string(),
        message: format!("Are you sure you want to delete '{}'? This action cannot be undone.", item_name),
        confirm_text: "Delete".to_string(),
        cancel_text: "Cancel".to_string(),
        confirm_message,
        cancel_message,
        is_danger: true,
    })
}

/// Info modal
pub fn info_modal<'a>(title: String, message: String, close_message: Message) -> Element<'a, Message> {
    let dialog = container(
        column![
            text(title).size(20),
            Space::new().height(16),
            
            text(message).size(14),
            
            Space::new().height(24),
            
            row![
                Space::new().width(Length::Fill),
                button(text("OK").size(14))
                    .padding([8, 16])
                    .style(|theme: &Theme, status: button::Status| primary_button_style(theme, status))
                    .on_press(close_message),
            ],
        ]
        .spacing(4)
        .padding(24)
        .width(Length::Fixed(400.0))
    )
    .style(|theme: &Theme| card_container(theme));

    // Overlay
    container(
        container(dialog)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
        ..Default::default()
    })
    .into()
}
