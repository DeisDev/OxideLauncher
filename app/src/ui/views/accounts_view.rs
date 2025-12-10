//! Accounts view - manage Microsoft and offline accounts

use iced::widget::{
    button, column, container, row, scrollable, text, text_input,
    Space,
};
use iced::{Alignment, Element, Length, Theme};
use crate::app::{Message, OxideLauncher};
use crate::core::accounts::{Account, AccountType};
use crate::ui::styles::*;

/// Build the accounts view
pub fn accounts_view<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let content = column![
        // Header
        row![
            text("Accounts").size(24),
            Space::new().width(Length::Fill),
            button(text("Add Microsoft Account").size(14))
                .padding([10, 16])
                .style(|theme: &Theme, status: button::Status| primary_button_style(theme, status))
                .on_press(Message::AddMicrosoftAccount),
            button(text("Add Offline Account").size(14))
                .padding([10, 16])
                .style(|theme: &Theme, status: button::Status| secondary_button_style(theme, status))
                .on_press(Message::ShowAddOfflineAccount),
        ]
        .align_y(Alignment::Center),
        
        Space::new().height(16),
        
        // Account list or empty state
        if app.accounts.is_empty() {
            empty_accounts_view()
        } else {
            accounts_list(app)
        },
        
        // Add offline account dialog
        if app.show_add_offline_dialog {
            add_offline_account_dialog(app)
        } else {
            Space::new().height(0).into()
        },
        
        // Microsoft auth dialog
        if app.show_msa_auth_dialog {
            msa_auth_dialog(app)
        } else {
            Space::new().height(0).into()
        },
    ]
    .spacing(8);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Empty accounts view
fn empty_accounts_view<'a>() -> Element<'a, Message> {
    container(
        column![
            text("No accounts added").size(20),
            Space::new().height(8),
            text("Add a Microsoft account to play online, or create an offline account for testing.").size(14),
            Space::new().height(16),
            row![
                button(text("Add Microsoft Account").size(14))
                    .padding([10, 20])
                    .style(|theme: &Theme, status: button::Status| primary_button_style(theme, status))
                    .on_press(Message::AddMicrosoftAccount),
                button(text("Add Offline Account").size(14))
                    .padding([10, 20])
                    .style(|theme: &Theme, status: button::Status| secondary_button_style(theme, status))
                    .on_press(Message::ShowAddOfflineAccount),
            ]
            .spacing(8),
        ]
        .align_x(Alignment::Center)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// Accounts list
fn accounts_list<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let accounts: Vec<&Account> = app.accounts.iter().collect();
    
    let mut account_items: Vec<Element<Message>> = Vec::new();
    
    for account in accounts {
        account_items.push(account_card(account, app));
    }
    
    scrollable(
        column(account_items)
            .spacing(8)
            .width(Length::Fill)
    )
    .height(Length::Fill)
    .into()
}

/// Account card
fn account_card<'a>(account: &'a Account, app: &'a OxideLauncher) -> Element<'a, Message> {
    let is_active = account.is_active;
    
    let account_type_text = match &account.account_type {
        AccountType::Microsoft => "Microsoft Account",
        AccountType::Offline => "Offline Account",
    };
    
    let status_color = if account.is_valid() {
        iced::Color::from_rgb(0.3, 0.7, 0.3)
    } else {
        iced::Color::from_rgb(0.7, 0.3, 0.3)
    };
    
    let status_text = if account.is_valid() {
        "✓ Valid"
    } else {
        "✗ Needs refresh"
    };
    
    let card_content = row![
        // Account avatar placeholder
        container(
            text("👤").size(32)
        )
        .width(Length::Fixed(60.0))
        .height(Length::Fixed(60.0))
        .center_x(Length::Fixed(60.0))
        .center_y(Length::Fixed(60.0))
        .style(|theme: &Theme| {
            let mut style = card_container(theme);
            style.border.radius = 30.0.into();
            style
        }),
        
        Space::new().width(16),
        
        // Account info
        column![
            row![
                text(&account.username).size(18),
                if is_active {
                    text(" (Active)")
                        .size(14)
                        .style(|_theme: &Theme| iced::widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.3, 0.7, 0.3))
                        })
                } else {
                    text("")
                },
            ],
            text(account_type_text).size(14),
            text(status_text)
                .size(12)
                .style(move |_theme: &Theme| iced::widget::text::Style {
                    color: Some(status_color)
                }),
        ]
        .spacing(4),
        
        Space::new().width(Length::Fill),
        
        // Actions
        column![
            row![
                if !is_active {
                    button(text("Set Active").size(14))
                        .padding([8, 12])
                        .style(|theme: &Theme, status: button::Status| primary_button_style(theme, status))
                        .on_press(Message::SetActiveAccount(account.id.clone()))
                } else {
                    button(text("Active").size(14))
                        .padding([8, 12])
                        .style(|theme: &Theme, status: button::Status| {
                            let mut style = secondary_button_style(theme, status);
                            style.text_color = iced::Color::from_rgb(0.3, 0.7, 0.3);
                            style
                        })
                },
                {
                    let refresh_element: Element<_> = if matches!(account.account_type, AccountType::Microsoft) && !account.is_valid() {
                        button(text("Refresh").size(14))
                            .padding([8, 12])
                            .style(|theme: &Theme, status: button::Status| primary_button_style(theme, status))
                            .on_press(Message::RefreshAccount(account.id.clone()))
                            .into()
                    } else {
                        Space::new().into()
                    };
                    refresh_element
                },
                button(text("Remove").size(14))
                    .padding([8, 12])
                    .style(|theme: &Theme, status: button::Status| danger_button_style(theme, status))
                    .on_press(Message::RemoveAccount(account.id.clone())),
            ]
            .spacing(8),
        ],
    ]
    .align_y(Alignment::Center)
    .padding(16);

    container(card_content)
        .width(Length::Fill)
        .style(move |theme: &Theme| {
            let mut style = card_container(theme);
            if is_active {
                style.border.color = iced::Color::from_rgb(0.3, 0.6, 0.3);
                style.border.width = 2.0;
            }
            style
        })
        .into()
}

/// Add offline account dialog
fn add_offline_account_dialog<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let dialog = container(
        column![
            text("Add Offline Account").size(20),
            Space::new().height(16),
            
            text("Enter a username for the offline account:").size(14),
            Space::new().height(8),
            
            text_input("Username", &app.offline_username_input)
                .on_input(Message::OfflineUsernameChanged)
                .on_submit(Message::CreateOfflineAccount)
                .padding(12)
                .width(Length::Fill),
            
            Space::new().height(8),
            
            text("Note: Offline accounts cannot be used on online servers.")
                .size(12)
                .style(|_theme: &Theme| iced::widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
                }),
            
            Space::new().height(16),
            
            row![
                Space::new().width(Length::Fill),
                button(text("Cancel").size(14))
                    .padding([8, 16])
                    .style(|theme: &Theme, status: button::Status| secondary_button_style(theme, status))
                    .on_press(Message::HideAddOfflineAccount),
                button(text("Create").size(14))
                    .padding([8, 16])
                    .style(|theme: &Theme, status: button::Status| primary_button_style(theme, status))
                    .on_press(Message::CreateOfflineAccount),
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

/// Microsoft auth dialog
fn msa_auth_dialog<'a>(app: &'a OxideLauncher) -> Element<'a, Message> {
    let dialog = container(
        column![
            text("Sign in with Microsoft").size(20),
            Space::new().height(16),
            
            if let Some(ref code) = app.msa_device_code {
                column![
                    text("1. Go to:").size(14),
                    text("https://microsoft.com/link")
                        .size(16)
                        .style(|_theme: &Theme| iced::widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.3, 0.6, 0.9))
                        }),
                    
                    Space::new().height(8),
                    
                    text("2. Enter this code:").size(14),
                    text(code)
                        .size(24)
                        .style(|_theme: &Theme| iced::widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.3, 0.7, 0.3))
                        }),
                    
                    Space::new().height(16),
                    
                    text("Waiting for you to sign in...")
                        .size(14)
                        .style(|_theme: &Theme| iced::widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
                        }),
                    
                    Space::new().height(8),
                    
                    row![
                        button(text("Copy Code").size(14))
                            .padding([8, 16])
                            .on_press(Message::CopyDeviceCode),
                        button(text("Open Link").size(14))
                            .padding([8, 16])
                            .on_press(Message::OpenUrl("https://microsoft.com/link".to_string())),
                    ]
                    .spacing(8),
                ]
                .spacing(4)
            } else {
                column![
                    text("Starting Microsoft authentication...")
                        .size(14)
                        .style(|_theme: &Theme| iced::widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
                        }),
                ]
            },
            
            Space::new().height(16),
            
            row![
                Space::new().width(Length::Fill),
                button(text("Cancel").size(14))
                    .padding([8, 16])
                    .style(|theme: &Theme, status: button::Status| secondary_button_style(theme, status))
                    .on_press(Message::CancelMicrosoftAuth),
            ],
        ]
        .spacing(4)
        .padding(24)
        .width(Length::Fixed(400.0))
        .align_x(Alignment::Center)
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
