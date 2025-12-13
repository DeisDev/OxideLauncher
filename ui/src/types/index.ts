/**
 * Centralized TypeScript type definitions for OxideLauncher
 * 
 * This file contains all shared interfaces used across the frontend.
 * Import from '@/types' instead of defining locally.
 */

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
  major_version: number;
  version: string;
  lts: boolean;
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
  account_type: string;
  is_active: boolean;
}

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

export interface Config {
  java: JavaConfig;
  memory: MemoryConfig;
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
