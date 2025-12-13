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

export const TABS: { id: TabType; label: string }[] = [
  { id: "log", label: "Minecraft Log" },
  { id: "version", label: "Version" },
  { id: "mods", label: "Mods" },
  { id: "resourcepacks", label: "Resource Packs" },
  { id: "shaderpacks", label: "Shader Packs" },
  { id: "notes", label: "Notes" },
  { id: "worlds", label: "Worlds" },
  { id: "screenshots", label: "Screenshots" },
  { id: "settings", label: "Settings" },
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
  major_version: number;
  version: string;
  lts: boolean;
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

export interface InstanceSettings {
  java_path: string | null;
  memory_min_mb: number;
  memory_max_mb: number;
  java_args: string;
  game_args: string;
  window_width: number;
  window_height: number;
  start_maximized: boolean;
  console_mode: "always" | "on_error" | "never";
  pre_launch_hook: string | null;
  post_exit_hook: string | null;
  enable_analytics: boolean;
  enable_logging: boolean;
  game_dir_override: string | null;
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
