/*
 * OxideLaunch - Minecraft Launch Wrapper
 * Copyright (C) 2024-2025 OxideLauncher Contributors
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 */

package dev.oxide.launch.launchers;

import dev.oxide.launch.LaunchConfig;
import dev.oxide.launch.OxideLaunch;
import dev.oxide.launch.util.ReflectionUtils;
import java.lang.invoke.MethodHandle;

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
