/*
 * OxideLaunch - Minecraft Launch Wrapper
 * Copyright (C) 2024-2025 OxideLauncher Contributors
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

package dev.oxide.launch;

import dev.oxide.launch.launchers.Launcher;
import dev.oxide.launch.launchers.LegacyLauncher;
import dev.oxide.launch.launchers.StandardLauncher;
import dev.oxide.launch.launchers.TweakerLauncher;

/**
 * Main entry point for the OxideLaunch wrapper.
 * 
 * This wrapper handles the complexities of launching different Minecraft versions
 * and modloaders by providing a unified interface that the Rust launcher can call.
 * 
 * Usage:
 *   java -cp "OxideLaunch.jar;minecraft.jar;libs/*" dev.oxide.launch.OxideLaunch [options] -- [game args]
 * 
 * Options:
 *   --launcher <type>     Launcher type: standard, tweaker, legacy (default: standard)
 *   --mainClass <class>   The main class to launch
 *   --gameDir <path>      The game directory path
 *   --assetsDir <path>    The assets directory path
 *   --tweakClass <class>  Add a tweaker class (can be specified multiple times)
 *   --width <pixels>      Window width
 *   --height <pixels>     Window height
 *   --                    Separator between wrapper args and game args
 */
public final class OxideLaunch {
    
    public static final String VERSION = "1.0.0";
    
    public static void main(String[] args) {
        try {
            // Handle --help and --version before anything else
            if (args.length > 0) {
                String first = args[0];
                if ("--help".equals(first) || "-h".equals(first)) {
                    printHelp();
                    return;
                }
                if ("--version".equals(first) || "-v".equals(first)) {
                    System.out.println("OxideLaunch v" + VERSION);
                    return;
                }
            }
            
            log("OxideLaunch v" + VERSION + " starting...");
            
            LaunchConfig config = LaunchConfig.parse(args);
            
            if (config == null) {
                System.err.println("Failed to parse launch configuration");
                System.exit(1);
                return;
            }
            
            log("Launcher type: " + config.getLauncherType());
            log("Main class: " + config.getMainClass());
            log("Game directory: " + config.getGameDir());
            
            Launcher launcher = createLauncher(config);
            
            if (launcher == null) {
                System.err.println("Unknown launcher type: " + config.getLauncherType());
                System.exit(1);
                return;
            }
            
            log("Launching game...");
            launcher.launch();
            
        } catch (Throwable t) {
            System.err.println("Fatal error during launch:");
            t.printStackTrace();
            System.exit(1);
        }
    }
    
    private static Launcher createLauncher(LaunchConfig config) {
        switch (config.getLauncherType()) {
            case STANDARD:
                return new StandardLauncher(config);
            case TWEAKER:
                return new TweakerLauncher(config);
            case LEGACY:
                return new LegacyLauncher(config);
            default:
                return null;
        }
    }
    
    private static void printHelp() {
        System.out.println("OxideLaunch v" + VERSION + " - Minecraft Launch Wrapper");
        System.out.println();
        System.out.println("Usage:");
        System.out.println("  java -cp \"OxideLaunch.jar;minecraft.jar;libs/*\" dev.oxide.launch.OxideLaunch [options] -- [game args]");
        System.out.println();
        System.out.println("Options:");
        System.out.println("  --launcher <type>     Launcher type: standard, tweaker, legacy (default: standard)");
        System.out.println("  --mainClass <class>   The main class to launch (required)");
        System.out.println("  --gameDir <path>      The game directory path");
        System.out.println("  --assetsDir <path>    The assets directory path");
        System.out.println("  --tweakClass <class>  Add a tweaker class (can be specified multiple times)");
        System.out.println("  --width <pixels>      Window width (default: 854)");
        System.out.println("  --height <pixels>     Window height (default: 480)");
        System.out.println("  --maximize            Start maximized");
        System.out.println("  --help, -h            Show this help message");
        System.out.println("  --version, -v         Show version");
        System.out.println("  --                    Separator between wrapper args and game args");
        System.out.println();
        System.out.println("Environment:");
        System.out.println("  oxide.debug=true      Enable debug logging");
    }
    
    public static void log(String message) {
        System.out.println("[OxideLaunch] " + message);
    }
    
    public static void debug(String message) {
        // Can be enabled via system property for debugging
        if (Boolean.getBoolean("oxide.debug")) {
            System.out.println("[OxideLaunch/DEBUG] " + message);
        }
    }
}
