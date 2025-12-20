/*
 * Tweaker-based launcher for LaunchWrapper modloaders
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
import java.util.ArrayList;
import java.util.List;

import dev.oxide.launch.LaunchConfig;
import dev.oxide.launch.OxideLaunch;
import dev.oxide.launch.util.ReflectionUtils;

/**
 * Tweaker-based launcher for LaunchWrapper modloaders.
 * 
 * This launcher handles Forge 1.6-1.12.2 and LiteLoader, which use
 * net.minecraft.launchwrapper.Launch as their main class with tweaker
 * classes passed as arguments.
 * 
 * The tweaker system works as follows:
 * 1. LaunchWrapper is invoked with --tweakClass arguments
 * 2. Each tweaker can modify the classpath, add more tweakers, etc.
 * 3. Finally, the actual Minecraft main class is invoked
 * 
 * Common tweakers:
 * - cpw.mods.fml.common.launcher.FMLTweaker (Forge 1.6-1.7)
 * - net.minecraftforge.fml.common.launcher.FMLTweaker (Forge 1.8-1.12)
 * - com.mumfrey.liteloader.launch.LiteLoaderTweaker (LiteLoader)
 */
public final class TweakerLauncher implements Launcher {
    
    // The standard LaunchWrapper main class
    private static final String LAUNCH_WRAPPER_CLASS = "net.minecraft.launchwrapper.Launch";
    
    private final LaunchConfig config;
    
    public TweakerLauncher(LaunchConfig config) {
        this.config = config;
    }
    
    @Override
    public void launch() throws Throwable {
        // Build arguments for LaunchWrapper
        List<String> launchArgs = new ArrayList<>();
        
        // Add tweaker classes
        // These MUST come before other game arguments
        for (String tweaker : config.getTweakClasses()) {
            launchArgs.add("--tweakClass");
            launchArgs.add(tweaker);
            OxideLaunch.debug("Adding tweaker: " + tweaker);
        }
        
        // Add game arguments after tweakers
        for (String arg : config.getGameArgs()) {
            launchArgs.add(arg);
        }
        
        // Add window dimensions
        launchArgs.add("--width");
        launchArgs.add(String.valueOf(config.getWidth()));
        launchArgs.add("--height");
        launchArgs.add(String.valueOf(config.getHeight()));
        
        // Set game directory if specified
        if (config.getGameDir() != null && !config.getGameDir().isEmpty()) {
            launchArgs.add("--gameDir");
            launchArgs.add(config.getGameDir());
        }
        
        // Set assets directory if specified
        if (config.getAssetsDir() != null && !config.getAssetsDir().isEmpty()) {
            launchArgs.add("--assetsDir");
            launchArgs.add(config.getAssetsDir());
        }
        
        String[] args = launchArgs.toArray(new String[0]);
        
        OxideLaunch.log("Tweaker launch: " + LAUNCH_WRAPPER_CLASS);
        OxideLaunch.log("Tweakers: " + config.getTweakClasses());
        OxideLaunch.debug("Launch arguments: " + String.join(" ", args));
        
        // Set system properties
        System.setProperty("oxide.launch.mainclass", LAUNCH_WRAPPER_CLASS);
        
        // Find and invoke LaunchWrapper's main method
        MethodHandle mainMethod = ReflectionUtils.findMainMethod(LAUNCH_WRAPPER_CLASS);
        mainMethod.invokeExact(args);
    }
}
