// Utility functions for instance creation including version parsing
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

// Utility functions for Create Instance view

// Parse Minecraft version to compare
export const parseMinecraftVersion = (versionStr: string): { major: number; minor: number; patch: number } | null => {
  const match = versionStr.match(/^(\d+)\.(\d+)(?:\.(\d+))?/);
  if (match) {
    return {
      major: parseInt(match[1], 10),
      minor: parseInt(match[2], 10),
      patch: match[3] ? parseInt(match[3], 10) : 0,
    };
  }
  return null;
};

// Compare versions: returns true if v1 >= v2
export const isVersionAtLeast = (versionStr: string, minMajor: number, minMinor: number, minPatch = 0): boolean => {
  const parsed = parseMinecraftVersion(versionStr);
  if (!parsed) return false;
  
  if (parsed.major > minMajor) return true;
  if (parsed.major < minMajor) return false;
  if (parsed.minor > minMinor) return true;
  if (parsed.minor < minMinor) return false;
  return parsed.patch >= minPatch;
};

// Check mod loader compatibility with selected Minecraft version
export const isLoaderCompatible = (loader: string, version: string): { compatible: boolean; reason?: string } => {
  if (!version || loader === "None") {
    return { compatible: true };
  }

  const parsed = parseMinecraftVersion(version);
  if (!parsed) {
    // For snapshots or unusual version strings, allow all loaders
    return { compatible: true };
  }

  switch (loader) {
    case "Fabric":
    case "Quilt":
      if (!isVersionAtLeast(version, 1, 14)) {
        return { compatible: false, reason: "Requires 1.14+" };
      }
      return { compatible: true };

    case "NeoForge":
      if (!isVersionAtLeast(version, 1, 20, 2)) {
        return { compatible: false, reason: "Requires 1.20.2+" };
      }
      return { compatible: true };

    case "Forge":
      return { compatible: true };

    case "LiteLoader":
      if (isVersionAtLeast(version, 1, 13)) {
        return { compatible: false, reason: "Only supports up to 1.12.2" };
      }
      if (!isVersionAtLeast(version, 1, 5, 2)) {
        return { compatible: false, reason: "Requires 1.5.2+" };
      }
      return { compatible: true };

    default:
      return { compatible: true };
  }
};

export const getVersionTypeLabel = (type: string): string => {
  switch (type) {
    case "release": return "release";
    case "snapshot": return "snapshot";
    case "old_beta": return "beta";
    case "old_alpha": return "alpha";
    default: return type;
  }
};

export const getVersionTypeColor = (type: string): string => {
  switch (type) {
    case "release": return "text-emerald-600 dark:text-emerald-400";
    case "snapshot": return "text-amber-600 dark:text-amber-400";
    case "old_beta": return "text-purple-600 dark:text-purple-400";
    case "old_alpha": return "text-red-600 dark:text-red-400";
    default: return "text-muted-foreground";
  }
};

export const getLoaderColor = (loader: string): string => {
  switch (loader.toLowerCase()) {
    case "fabric": return "text-amber-600 dark:text-amber-400";
    case "forge": return "text-blue-600 dark:text-blue-400";
    case "neoforge": return "text-orange-600 dark:text-orange-400";
    case "quilt": return "text-purple-600 dark:text-purple-400";
    case "liteloader": return "text-cyan-600 dark:text-cyan-400";
    default: return "";
  }
};

// Validate if a string is a valid URL
export const isValidUrl = (str: string): boolean => {
  try {
    const url = new URL(str);
    return url.protocol === "http:" || url.protocol === "https:" || url.protocol === "curseforge:";
  } catch {
    return false;
  }
};

// Get filename from path
export const getFilenameFromPath = (path: string): string => {
  return path.split(/[/\\]/).pop() || path;
};
