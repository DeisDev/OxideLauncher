/*
 * Standard launcher for modern Minecraft versions and modloaders
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

import java.lang.invoke.MethodHandle;

import dev.oxide.launch.LaunchConfig;
import dev.oxide.launch.OxideLaunch;
import dev.oxide.launch.util.ReflectionUtils;

/**
 * Standard launcher for modern Minecraft versions and modloaders.
 * 
 * This launcher simply invokes the main class via reflection, passing
 * the game arguments. It works for:
 * 
 * - Vanilla Minecraft 1.13+
 * - Forge 1.13+ (BootstrapLauncher)
 * - NeoForge
 * - Fabric
 * - Quilt
 * 
 * These modern modloaders handle their own setup internally and don't
 * require special handling from the launch wrapper.
 */
public final class StandardLauncher implements Launcher {
    
    private final LaunchConfig config;
    
    public StandardLauncher(LaunchConfig config) {
        this.config = config;
    }
    
    @Override
    public void launch() throws Throwable {
        String mainClassName = config.getMainClass();
        String[] gameArgs = config.buildGameArgs();
        
        OxideLaunch.log("Standard launch: " + mainClassName);
        OxideLaunch.debug("Game arguments: " + String.join(" ", gameArgs));
        
        // Set system properties that mods/launchers might expect
        System.setProperty("oxide.launch.mainclass", mainClassName);
        
        // Find and invoke the main method
        MethodHandle mainMethod = ReflectionUtils.findMainMethod(mainClassName);
        mainMethod.invokeExact(gameArgs);
    }
}
