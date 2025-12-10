//! Oxide Launcher - A Rust-based Minecraft Launcher
//! 
//! This launcher is inspired by Prism Launcher and aims to provide
//! a modern, fast, and feature-rich experience for managing Minecraft instances.

mod app;
mod core;
mod ui;

use app::OxideLauncher;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Oxide Launcher");

    // Run the application using Iced 0.14 API
    // The boot function can return just the state (IntoBoot is implemented for State)
    // or a tuple (State, Task<Message>)
    iced::application(OxideLauncher::new, OxideLauncher::update, OxideLauncher::view)
        .title("Oxide Launcher")
        .theme(OxideLauncher::theme)
        .subscription(OxideLauncher::subscription)
        .window_size((1200.0, 800.0))
        .run()
}
