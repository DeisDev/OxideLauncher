//! Create game folders step.
//!
//! Oxide Launcher — A Rust-based Minecraft launcher
//! Copyright (C) 2025 Oxide Launcher contributors
//!
//! This file is part of Oxide Launcher.
//!
//! Oxide Launcher is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! Oxide Launcher is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program. If not, see <https://www.gnu.org/licenses/>.

use async_trait::async_trait;
use std::fs;
use tracing::{debug, info};

use crate::core::launch::{LaunchContext, LaunchStep, LaunchStepResult};

/// Step that creates required game folders
pub struct CreateGameFoldersStep {
    status: Option<String>,
    progress: f32,
}

impl CreateGameFoldersStep {
    pub fn new() -> Self {
        Self {
            status: None,
            progress: 0.0,
        }
    }
}

#[async_trait]
impl LaunchStep for CreateGameFoldersStep {
    fn name(&self) -> &'static str {
        "Create Game Folders"
    }
    
    fn description(&self) -> &'static str {
        "Creates required game directories"
    }
    
    async fn execute(&mut self, context: &mut LaunchContext) -> LaunchStepResult {
        self.status = Some("Creating game directories...".to_string());
        self.progress = 0.0;
        
        let game_dir = context.instance.game_dir();
        
        // Create main game directory
        if let Err(e) = fs::create_dir_all(&game_dir) {
            return LaunchStepResult::Failed(format!(
                "Couldn't create the main game folder: {}\nPath: {:?}",
                e, game_dir
            ));
        }
        debug!("Created game directory: {:?}", game_dir);
        self.progress = 0.2;
        
        // Create natives directory
        let natives_dir = &context.natives_dir;
        if let Err(e) = fs::create_dir_all(natives_dir) {
            return LaunchStepResult::Failed(format!(
                "Couldn't create the natives folder: {}\nPath: {:?}",
                e, natives_dir
            ));
        }
        debug!("Created natives directory: {:?}", natives_dir);
        self.progress = 0.4;
        
        // Create server-resource-packs folder (MCL-3732 workaround)
        let server_resource_packs = game_dir.join("server-resource-packs");
        if let Err(e) = fs::create_dir_all(&server_resource_packs) {
            // Non-fatal - just log a warning
            tracing::warn!(
                "Couldn't create the 'server-resource-packs' folder: {}\nPath: {:?}",
                e, server_resource_packs
            );
        }
        self.progress = 0.6;
        
        // Create other common directories
        let directories = [
            "mods",
            "resourcepacks",
            "shaderpacks",
            "saves",
            "screenshots",
            "logs",
            "config",
        ];
        
        for (i, dir) in directories.iter().enumerate() {
            let dir_path = game_dir.join(dir);
            if let Err(e) = fs::create_dir_all(&dir_path) {
                // Non-fatal for optional directories
                tracing::warn!("Couldn't create '{}' folder: {}", dir, e);
            }
            self.progress = 0.6 + (0.4 * (i as f32 + 1.0) / directories.len() as f32);
        }
        
        info!("Game directories created successfully");
        self.status = Some("Game directories ready".to_string());
        self.progress = 1.0;
        
        LaunchStepResult::Success
    }
    
    fn progress(&self) -> f32 {
        self.progress
    }
    
    fn status(&self) -> Option<String> {
        self.status.clone()
    }
}

impl Default for CreateGameFoldersStep {
    fn default() -> Self {
        Self::new()
    }
}
