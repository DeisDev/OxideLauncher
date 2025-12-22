// Centralized TypeScript type definitions for the OxideLauncher frontend.
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

// Instance Types
export interface InstanceInfo {
  id: string;
  name: string;
  minecraft_version: string;
  mod_loader: string;
  mod_loader_version: string | null;
  icon?: string | null;
  last_played?: string | null;
  total_played_seconds?: number;
}

// Mod Types
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
}

// Java Types
export interface JavaInstallation {
  id?: string;
  path: string;
  version: string;
  major_version?: number;
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

// World Types
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

// Resource Types
export interface ResourcePackInfo {
  filename: string;
  name: string;
  description: string | null;
  size: string;
  enabled: boolean;
}

export interface ShaderPackInfo {
  filename: string;
  name: string;
  size: string;
}

export interface ScreenshotInfo {
  filename: string;
  path: string;
  modified: string;
  size: string;
}

// Account Types
export interface AccountInfo {
  id: string;
  username: string;
  uuid: string;
  account_type: "Microsoft" | "Offline";
  is_active: boolean;
  is_valid: boolean;
  needs_refresh: boolean;
  skin_url: string | null;
  added_at: string;
  last_used: string | null;
}

// Skin Management Types
export interface SkinInfoResponse {
  id: string;
  url: string;
  variant: "slim" | "classic";
  is_active: boolean;
}

export interface CapeInfoResponse {
  id: string;
  url: string;
  alias: string | null;
  is_active: boolean;
}

export interface PlayerProfileResponse {
  id: string;
  name: string;
  skins: SkinInfoResponse[];
  capes: CapeInfoResponse[];
  active_skin: SkinInfoResponse | null;
  active_cape: CapeInfoResponse | null;
}

export interface FetchedSkinResponse {
  uuid: string;
  username: string;
  skin_url: string | null;
  skin_variant: "slim" | "classic";
  cape_url: string | null;
}

export interface DeviceCodeInfo {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

export type AuthProgressEventType =
  | { type: "StepStarted"; data: { step: string; description: string } }
  | { type: "DeviceCodeReady"; data: { user_code: string; verification_uri: string; expires_in: number } }
  | { type: "PollingForAuth"; data: { message: string } }
  | { type: "StepCompleted"; data: { step: string } }
  | { type: "Failed"; data: { step: string; error: string } }
  | { type: "Completed"; data: { username: string } };

// Config Types
export interface JavaConfig {
  custom_path: string | null;
  extra_args: string;
  auto_download: boolean;
  skip_compatibility_check: boolean;
  auto_detect?: boolean;
}

export interface MemoryConfig {
  min_memory: number;
  max_memory: number;
}

export interface LoggingConfig {
  debug_to_file: boolean;
  max_file_size_mb: number;
  max_files: number;
}

export interface Config {
  java: JavaConfig;
  memory: MemoryConfig;
  logging: LoggingConfig;
}

// Version Types
export interface MinecraftVersionInfo {
  id: string;
  version_type: string;
  release_time: string;
}

export interface LoaderVersionInfo {
  version: string;
  is_stable: boolean;
  minecraft_version?: string;
}

// Tab Types
export type InstanceTabType =
  | "log"
  | "version"
  | "mods"
  | "resourcepacks"
  | "shaderpacks"
  | "notes"
  | "worlds"
  | "screenshots"
  | "settings";

// Log Types
export type LogSource = "launcher" | "game" | "stderr";

export type LogLevel = "trace" | "debug" | "info" | "warning" | "error" | "fatal";

export interface LogEntry {
  timestamp: number;
  source: LogSource;
  level: LogLevel;
  content: string;
}
