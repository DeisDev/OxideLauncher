//! Progress bar component

use iced::widget::{column, container, row, text, progress_bar as iced_progress_bar, Space};
use iced::{Element, Length, Theme};
use crate::app::Message;
use crate::ui::styles::*;

/// Progress bar with label and percentage
pub fn progress_bar_with_label<'a>(
    label: String,
    progress: f32,
    total: Option<String>,
) -> Element<'a, Message> {
    let percentage = (progress * 100.0) as u8;
    
    column![
        row![
            text(label).size(14),
            Space::new().width(Length::Fill),
            if let Some(total_text) = total {
                text(total_text).size(12)
            } else {
                text(format!("{}%", percentage)).size(12)
            },
        ],
        container(iced_progress_bar(0.0..=1.0, progress))
            .width(Length::Fill)
            .height(Length::Fixed(8.0)),
    ]
    .spacing(4)
    .into()
}

/// Download progress component
pub fn download_progress<'a>(
    filename: String,
    downloaded_bytes: u64,
    total_bytes: u64,
    speed: Option<f64>,
) -> Element<'a, Message> {
    let progress = if total_bytes > 0 {
        downloaded_bytes as f32 / total_bytes as f32
    } else {
        0.0
    };
    
    let size_text = format!(
        "{} / {}",
        format_bytes(downloaded_bytes),
        format_bytes(total_bytes)
    );
    
    let speed_text = speed.map(|s| format!(" - {}/s", format_bytes(s as u64)));
    
    container(
        column![
            row![
                text(filename).size(14),
                Space::new().width(Length::Fill),
                text(size_text).size(12),
                if let Some(speed) = speed_text {
                    text(speed).size(12)
                } else {
                    text("")
                },
            ],
            container(iced_progress_bar(0.0..=1.0, progress))
                .width(Length::Fill)
                .height(Length::Fixed(6.0)),
        ]
        .spacing(4)
        .padding(8)
    )
    .style(|theme: &Theme| card_container(theme))
    .into()
}

/// Multi-file download progress
pub fn multi_download_progress<'a>(
    title: String,
    completed: usize,
    total: usize,
    current_file: Option<String>,
) -> Element<'a, Message> {
    let progress = if total > 0 {
        completed as f32 / total as f32
    } else {
        0.0
    };
    
    container(
        column![
            row![
                text(title).size(16),
                Space::new().width(Length::Fill),
                text(format!("{} / {}", completed, total)).size(14),
            ],
            container(iced_progress_bar(0.0..=1.0, progress))
                .width(Length::Fill)
                .height(Length::Fixed(8.0)),
            if let Some(file) = current_file {
                text(format!("Downloading: {}", file))
                    .size(12)
                    .style(|_theme: &Theme| iced::widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6))
                    })
            } else {
                text("")
            },
        ]
        .spacing(4)
        .padding(12)
    )
    .style(|theme: &Theme| card_container(theme))
    .into()
}

/// Format bytes to human readable
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
