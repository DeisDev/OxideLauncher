/*
 * Interface for Minecraft launcher implementations
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

package dev.oxide.launch.launchers;

/**
 * Interface for Minecraft launchers.
 * 
 * Each launcher implementation handles a specific type of Minecraft version
 * or modloader configuration.
 */
public interface Launcher {
    
    /**
     * Launch Minecraft.
     * 
     * This method should not return under normal circumstances - it will
     * invoke the main class which runs until the game exits.
     * 
     * @throws Throwable If launch fails
     */
    void launch() throws Throwable;
}
