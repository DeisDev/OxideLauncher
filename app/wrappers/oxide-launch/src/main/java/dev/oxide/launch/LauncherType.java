/*
 * Enumeration of supported Minecraft launcher types
 *
 * Oxide Launcher — A Rust-based Minecraft launcher
 * Copyright (C) 2025 Oxide Launcher contributors
 *
 * This file is part of Oxide Launcher.
 *
 * Oxide Launcher is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Oxide Launcher is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

package dev.oxide.launch;

/**
 * Enumeration of supported launcher types.
 * 
 * Each type handles a different category of Minecraft versions and modloaders.
 */
public enum LauncherType {
    /**
     * Standard launcher for modern versions.
     * 
     * Used for:
     * - Vanilla Minecraft 1.13+
     * - Forge 1.13+ (using BootstrapLauncher)
     * - NeoForge (all versions)
     * - Fabric (all versions)
     * - Quilt (all versions)
     * 
     * Simply invokes the main class via reflection.
     */
    STANDARD,
    
    /**
     * Tweaker-based launcher for LaunchWrapper modloaders.
     * 
     * Used for:
     * - Forge 1.6 - 1.12.2 (using net.minecraft.launchwrapper.Launch)
     * - LiteLoader
     * - Other LaunchWrapper-based modloaders
     * 
     * Passes tweaker classes as arguments to LaunchWrapper.
     */
    TWEAKER,
    
    /**
     * Legacy launcher for very old Minecraft versions.
     * 
     * Used for:
     * - Vanilla Minecraft Alpha/Beta through 1.5.2
     * - Early Forge versions
     * 
     * Handles:
     * - Applet-based launch
     * - Game directory field injection
     * - Special window handling
     */
    LEGACY;
    
    /**
     * Parse a launcher type from a string.
     * 
     * @param value The string value (case-insensitive)
     * @return The launcher type, or STANDARD if not recognized
     */
    public static LauncherType fromString(String value) {
        if (value == null) {
            return STANDARD;
        }
        
        switch (value.toLowerCase()) {
            case "tweaker":
            case "launchwrapper":
                return TWEAKER;
            case "legacy":
            case "applet":
                return LEGACY;
            case "standard":
            default:
                return STANDARD;
        }
    }
}
