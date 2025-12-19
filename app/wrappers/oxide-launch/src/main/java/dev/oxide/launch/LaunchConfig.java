/*
 * OxideLaunch - Minecraft Launch Wrapper
 * Copyright (C) 2024-2025 OxideLauncher Contributors
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
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
