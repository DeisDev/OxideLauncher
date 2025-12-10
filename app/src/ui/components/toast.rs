//! Toast notification component

use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length, Theme};
use crate::app::Message;
use crate::ui::styles::*;

/// Toast notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Info,
    Warning,
    Error,
}

impl ToastType {
    pub fn icon(&self) -> &'static str {
        match self {
            ToastType::Success => "✓",
            ToastType::Info => "ℹ",
            ToastType::Warning => "⚠",
            ToastType::Error => "✗",
        }
    }
    
    pub fn color(&self) -> iced::Color {
        match self {
            ToastType::Success => iced::Color::from_rgb(0.3, 0.7, 0.3),
            ToastType::Info => iced::Color::from_rgb(0.2, 0.5, 0.8),
            ToastType::Warning => iced::Color::from_rgb(0.9, 0.6, 0.1),
            ToastType::Error => iced::Color::from_rgb(0.8, 0.3, 0.3),
        }
    }
}

/// Toast notification data
#[derive(Debug, Clone)]
pub struct Toast {
    pub id: usize,
    pub toast_type: ToastType,
    pub message: String,
    pub duration_ms: u64,
}

impl Toast {
    pub fn new(toast_type: ToastType, message: impl Into<String>) -> Self {
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        Self {
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            toast_type,
            message: message.into(),
            duration_ms: 5000,
        }
    }
    
    pub fn success(message: impl Into<String>) -> Self {
        Self::new(ToastType::Success, message)
    }
    
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(ToastType::Info, message)
    }
    
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(ToastType::Warning, message)
    }
    
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(ToastType::Error, message)
    }
    
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

/// Build a toast notification element
pub fn toast_notification<'a>(toast: &'a Toast) -> Element<'a, Message> {
    let color = toast.toast_type.color();
    let icon = toast.toast_type.icon();
    
    container(
        row![
            text(icon)
                .size(16)
                .style(move |_theme: &Theme| iced::widget::text::Style {
                    color: Some(color)
                }),
            text(&toast.message).size(14),
            button(text("×").size(14))
                .padding([4, 8])
                .style(|theme: &Theme, status: button::Status| icon_button_style(theme, status))
                .on_press(Message::DismissToast(toast.id)),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .padding(12)
    )
    .style(move |theme: &Theme| {
        let mut style = card_container(theme);
        style.border.color = color;
        style.border.width = 2.0;
        style
    })
    .width(Length::Fixed(350.0))
    .into()
}

/// Build a toast container with multiple toasts
pub fn toast_container<'a>(toasts: &'a [Toast]) -> Element<'a, Message> {
    let mut toast_column = iced::widget::column![].spacing(8);
    
    for toast in toasts {
        toast_column = toast_column.push(toast_notification(toast));
    }
    
    container(toast_column)
        .padding(16)
        .into()
}
