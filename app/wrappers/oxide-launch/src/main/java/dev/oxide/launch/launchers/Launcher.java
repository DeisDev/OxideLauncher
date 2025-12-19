/*
 * OxideLaunch - Minecraft Launch Wrapper
 * Copyright (C) 2024-2025 OxideLauncher Contributors
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
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
