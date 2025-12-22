// TypeScript type definitions for the settings system
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

/**
 * Settings types for the modular settings system
 */

// Window state for position memory
export interface WindowState {
  x: number | null;
  y: number | null;
  width: number | null;
  height: number | null;
  maximized: boolean;
}

// Java installation type
export interface JavaInstallation {
  id: string;
  path: string;
  version: string;
  major_version: number;
  arch: string;
  vendor: string;
  is_64bit: boolean;
  is_managed: boolean;
  recommended: boolean;
}

export interface AvailableJavaVersion {
  major: number;
  name: string;
  is_lts: boolean;
}

// Proxy types
export type ProxyType = "Http" | "Socks5";

export interface ProxyConfig {
  proxy_type: ProxyType;
  host: string;
  port: number;
  username: string | null;
  password: string | null;
}

// Instance view mode
export type InstanceViewMode = "Grid" | "List";

// Full config interface matching Rust
export interface Config {
  data_dir: string;
  instances_dir: string | null;
  theme: string;
  java: JavaConfig;
  network: NetworkConfig;
  ui: UiConfig;
  minecraft: MinecraftConfig;
  commands: CustomCommands;
  memory: MemoryConfig;
  logging: LoggingConfig;
  api_keys: ApiKeys;
  debug: DebugConfig;
  files: FilesConfig;
}

export interface JavaConfig {
  custom_path: string | null;
  use_bundled: boolean;
  auto_detect: boolean;
  extra_args: string[];
  skip_compatibility_check: boolean;
  auto_download: boolean;
}

export interface NetworkConfig {
  proxy: ProxyConfig | null;
  max_concurrent_downloads: number;
  download_retries: number;
  timeout_seconds: number;
  user_agent: string;
  downloads_dir: string | null;
  downloads_dir_watch_recursive: boolean;
}

export interface UiConfig {
  show_news: boolean;
  instance_view: InstanceViewMode;
  instance_sort_by: string;
  instance_sort_asc: boolean;
  instance_grid_size: string;
  color_scheme: string;
  window_width: number;
  window_height: number;
  last_instance: string | null;
  rust_mode: boolean;
  remember_main_window_position: boolean;
  remember_dialog_window_positions: boolean;
  main_window_state: WindowState;
  dialog_window_states: Record<string, WindowState>;
  open_instance_after_install: boolean;
}

export interface MinecraftConfig {
  window_width: number;
  window_height: number;
  launch_maximized: boolean;
  close_after_launch: boolean;
  show_console: boolean;
  auto_close_console: boolean;
  show_console_on_error: boolean;
  record_game_time: boolean;
  show_game_time: boolean;
}

export interface CustomCommands {
  pre_launch: string | null;
  post_exit: string | null;
  wrapper_command: string | null;
}

export interface MemoryConfig {
  min_memory: number;
  max_memory: number;
  permgen: number;
}

export interface LoggingConfig {
  debug_to_file: boolean;
  max_file_size_mb: number;
  max_files: number;
}

export interface ApiKeys {
  msa_client_id: string | null;
  curseforge_api_key: string | null;
  modrinth_api_token: string | null;
}

export interface DebugConfig {
  force_java_console: boolean;
  disable_create_no_window: boolean;
  log_launch_commands: boolean;
  verbose_logging: boolean;
  keep_natives_after_launch: boolean;
  pause_before_launch: boolean;
}

export interface FilesConfig {
  /** Move deleted files to recycle bin instead of permanent deletion */
  use_recycle_bin: boolean;
}

// Settings context type
export interface SettingsContextType {
  config: Config | null;
  setConfig: React.Dispatch<React.SetStateAction<Config | null>>;
  saveConfig: () => Promise<void>;
  loading: boolean;
}
