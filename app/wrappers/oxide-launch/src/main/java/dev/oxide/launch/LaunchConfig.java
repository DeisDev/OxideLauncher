/*
 * Configuration class for Minecraft launch parameters
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

import java.util.ArrayList;
import java.util.List;

/**
 * Configuration for launching Minecraft.
 * 
 * Parsed from command-line arguments passed to OxideLaunch.
 */
public final class LaunchConfig {
    
    private LauncherType launcherType = LauncherType.STANDARD;
    private String mainClass;
    private String gameDir;
    private String assetsDir;
    private int width = 854;
    private int height = 480;
    private boolean maximize = false;
    private final List<String> tweakClasses = new ArrayList<>();
    private final List<String> gameArgs = new ArrayList<>();
    
    private LaunchConfig() {}
    
    /**
     * Parse launch configuration from command-line arguments.
     * 
     * @param args The command-line arguments
     * @return The parsed configuration, or null if parsing failed
     */
    public static LaunchConfig parse(String[] args) {
        LaunchConfig config = new LaunchConfig();
        boolean parsingGameArgs = false;
        
        for (int i = 0; i < args.length; i++) {
            String arg = args[i];
            
            // Everything after "--" is passed to the game
            if ("--".equals(arg)) {
                parsingGameArgs = true;
                continue;
            }
            
            if (parsingGameArgs) {
                config.gameArgs.add(arg);
                continue;
            }
            
            // Parse wrapper arguments
            switch (arg) {
                case "--launcher":
                    if (i + 1 < args.length) {
                        config.launcherType = LauncherType.fromString(args[++i]);
                    }
                    break;
                    
                case "--mainClass":
                    if (i + 1 < args.length) {
                        config.mainClass = args[++i];
                    }
                    break;
                    
                case "--gameDir":
                    if (i + 1 < args.length) {
                        config.gameDir = args[++i];
                    }
                    break;
                    
                case "--assetsDir":
                    if (i + 1 < args.length) {
                        config.assetsDir = args[++i];
                    }
                    break;
                    
                case "--width":
                    if (i + 1 < args.length) {
                        try {
                            config.width = Integer.parseInt(args[++i]);
                        } catch (NumberFormatException e) {
                            // Use default
                        }
                    }
                    break;
                    
                case "--height":
                    if (i + 1 < args.length) {
                        try {
                            config.height = Integer.parseInt(args[++i]);
                        } catch (NumberFormatException e) {
                            // Use default
                        }
                    }
                    break;
                    
                case "--maximize":
                    config.maximize = true;
                    break;
                    
                case "--tweakClass":
                    if (i + 1 < args.length) {
                        config.tweakClasses.add(args[++i]);
                    }
                    break;
                    
                default:
                    // Unknown argument - could be a game argument without separator
                    // For safety, just skip it
                    OxideLaunch.debug("Unknown argument: " + arg);
                    break;
            }
        }
        
        // Validate required fields
        if (config.mainClass == null || config.mainClass.isEmpty()) {
            System.err.println("Error: --mainClass is required");
            return null;
        }
        
        return config;
    }
    
    // Getters
    
    public LauncherType getLauncherType() {
        return launcherType;
    }
    
    public String getMainClass() {
        return mainClass;
    }
    
    public String getGameDir() {
        return gameDir;
    }
    
    public String getAssetsDir() {
        return assetsDir;
    }
    
    public int getWidth() {
        return width;
    }
    
    public int getHeight() {
        return height;
    }
    
    public boolean isMaximize() {
        return maximize;
    }
    
    public List<String> getTweakClasses() {
        return tweakClasses;
    }
    
    public List<String> getGameArgs() {
        return gameArgs;
    }
    
    /**
     * Build the final game arguments including window size.
     * 
     * @return The complete list of game arguments
     */
    public String[] buildGameArgs() {
        List<String> args = new ArrayList<>(gameArgs);
        
        // Add window dimensions (most modern versions support these)
        args.add("--width");
        args.add(String.valueOf(width));
        args.add("--height");
        args.add(String.valueOf(height));
        
        return args.toArray(new String[0]);
    }
}
