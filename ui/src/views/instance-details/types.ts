// TypeScript type definitions for instance details components
//
// Oxide Launcher — A Rust-based Minecraft launcher
// Copyright (C) 2025 Oxide Launcher contributors
//
// This file is part of Oxide Launcher.
//
// Oxide Launcher is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Oxide Launcher is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

// Types used across instance details tabs

export type TabType =
  | "log"
  | "version"
  | "mods"
  | "resourcepacks"
  | "shaderpacks"
  | "notes"
  | "worlds"
  | "screenshots"
  | "settings";

export const TABS: { id: TabType; label: string; shortLabel: string }[] = [
  { id: "log", label: "Minecraft Log", shortLabel: "Log" },
  { id: "version", label: "Version", shortLabel: "Ver" },
  { id: "mods", label: "Mods", shortLabel: "Mods" },
  { id: "resourcepacks", label: "Resource Packs", shortLabel: "Res" },
  { id: "shaderpacks", label: "Shader Packs", shortLabel: "Shd" },
  { id: "notes", label: "Notes", shortLabel: "Notes" },
  { id: "worlds", label: "Worlds", shortLabel: "Worlds" },
  { id: "screenshots", label: "Screenshots", shortLabel: "Shots" },
  { id: "settings", label: "Settings", shortLabel: "Set" },
];

export interface InstanceInfo {
  id: string;
  name: string;
  minecraft_version: string;
  mod_loader: string;
  mod_loader_version: string | null;
}

export interface ModSearchResult {
  id: string;
  name: string;
  description: string;
  author: string;
  downloads: number;
  icon_url: string | null;
  project_type: string;
  platform: string;
}

export interface InstalledMod {
  filename: string;
  name: string;
  version: string | null;
  enabled: boolean;
  size: number;
  modified: string | null;
  provider: string | null;
  icon_url: string | null;
  homepage: string | null;
  issues_url: string | null;
  source_url: string | null;
}

export interface JavaInstallation {
  path: string;
  version: string;
  vendor: string;
  arch: string;
  is_64bit: boolean;
  recommended: boolean;
  is_managed: boolean;
}

export interface AvailableJavaVersion {
  major: number;
  name: string;
  is_lts: boolean;
}

export interface WorldInfo {
  folder_name: string;
  name: string;
  seed: number | null;
  game_type: string;
  hardcore: boolean;
  last_played: string | null;
  size: string;
  has_icon: boolean;
}

export interface ResourcePackInfo {
  filename: string;
  name: string;
  description: string | null;
  size: string;
  enabled: boolean;
  /** Path to the cached pack icon (extracted from pack.png inside the archive) */
  icon_path: string | null;
}

export interface ShaderPackInfo {
  filename: string;
  name: string;
  size: string;
}

export interface ScreenshotInfo {
  filename: string;
  path: string;
  timestamp: string | null;
  size: string;
}

// =============================================================================
// RustWiz Update Types
// =============================================================================

export interface UpdateCheckResult {
  filename: string;
  current_version: string;
  latest_version: string | null;
  latest_version_id: string | null;
  update_available: boolean;
  platform: string;
  changelog: string | null;
}

export interface BatchUpdateResult {
  updates_available: UpdateCheckResult[];
  up_to_date: string[];
  unchecked: string[];
  errors: string[];
}

export interface InstanceSettings {
  java_path: string | null;
  memory_min_mb: number;
  memory_max_mb: number;
  java_args: string;
  game_args: string;
  window_width: number;
  window_height: number;
  start_maximized: boolean;
  fullscreen: boolean;
  console_mode: "always" | "on_error" | "never";
  pre_launch_hook: string | null;
  post_exit_hook: string | null;
  enable_analytics: boolean;
  enable_logging: boolean;
  game_dir_override: string | null;
  skip_java_compatibility_check: boolean;
  wrapper_command: string | null;
  // Debug settings
  use_java_console: boolean;
  disable_create_no_window: boolean;
  log_launch_command: boolean;
}

export interface JavaInfo {
  path: string;
  version: string;
  major_version: number;
  vendor: string;
  architecture: string;
  is_64bit: boolean;
}

export interface JavaDownloadInfo {
  vendor: string;
  version: string;
  architecture: string;
  url: string;
}
