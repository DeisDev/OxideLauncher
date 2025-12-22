// TypeScript type definitions and constants for instance creation
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

// Types and constants for Create Instance view

// Source types for create instance tabs
export type SourceType = 
  | "custom"
  | "import"
  | "modpacks";

// Source configuration
export interface Source {
  id: SourceType;
  label: string;
}

// Available sources for creating instances
export const SOURCES: Source[] = [
  { id: "custom", label: "Custom" },
  { id: "import", label: "Import" },
  { id: "modpacks", label: "Modpacks" },
];

// Minecraft version info
export interface MinecraftVersion {
  id: string;
  version_type: string;
  release_time: string;
}

// Loader version info
export interface LoaderVersion {
  version: string;
  recommended: boolean;
}

// Modpack search result
export interface ModpackResult {
  id: string;
  slug: string;
  name: string;
  description: string;
  author: string;
  downloads: number;
  follows: number;
  icon_url: string | null;
  categories: string[];
  versions: string[];
  loaders: string[];
  date_created: string;
  date_modified: string;
  platform: string;
}

// Modpack version info
export interface ModpackVersion {
  id: string;
  project_id: string;
  name: string;
  version_number: string;
  changelog: string | null;
  game_versions: string[];
  loaders: string[];
  download_url: string | null;
  filename: string;
  size: number;
  downloads: number;
  date_published: string;
  version_type: string;
  platform: string;
}

// Supported import format
export interface ImportFormat {
  id: string;
  name: string;
  description: string;
  extensions: string[];
  category: "native" | "popular" | "legacy";
}

// Import formats supported by the launcher
export const IMPORT_FORMATS: ImportFormat[] = [
  {
    id: "oxide",
    name: "Oxide Launcher",
    description: "Native Oxide Launcher instances",
    extensions: [".oxide"],
    category: "native",
  },
  {
    id: "modrinth",
    name: "Modrinth",
    description: "Modrinth modpack format",
    extensions: [".mrpack"],
    category: "popular",
  },
  {
    id: "curseforge",
    name: "CurseForge",
    description: "CurseForge modpack format",
    extensions: [".zip"],
    category: "popular",
  },
  {
    id: "prism",
    name: "Prism/MultiMC/PolyMC",
    description: "Prism Launcher and compatible instances",
    extensions: [".zip", ".mrpack"],
    category: "popular",
  },
  {
    id: "technic",
    name: "Technic",
    description: "Technic platform modpacks (via URL)",
    extensions: [".zip"],
    category: "legacy",
  },
  {
    id: "atlauncher",
    name: "ATLauncher",
    description: "ATLauncher instances",
    extensions: [".zip"],
    category: "legacy",
  },
  {
    id: "ftb",
    name: "FTB App",
    description: "Feed The Beast app instances",
    extensions: [".zip"],
    category: "legacy",
  },
];
