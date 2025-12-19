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

import java.applet.Applet;
import java.awt.*;
import java.awt.event.WindowAdapter;
import java.awt.event.WindowEvent;
import java.io.File;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.net.MalformedURLException;
import java.net.URL;
import java.util.HashMap;
import java.util.Map;

/**
 * Legacy launcher for very old Minecraft versions.
 * 
 * This launcher handles:
 * - Applet-based Minecraft (Alpha, Beta, Release up to ~1.5.2)
 * - Game directory field injection (required for some versions)
 * - Early Forge versions
 * 
 * Old Minecraft versions have several quirks:
 * 1. Some expect to be launched as an Applet, not via main()
 * 2. The game directory needs to be set via reflection before launch
 * 3. Window handling is different (Frame vs modern LWJGL window)
 */
public final class LegacyLauncher implements Launcher {
    
    private static final String DEFAULT_APPLET_CLASS = "net.minecraft.client.MinecraftApplet";
    
    private final LaunchConfig config;
    
    public LegacyLauncher(LaunchConfig config) {
        this.config = config;
    }
    
    @Override
    public void launch() throws Throwable {
        String mainClassName = config.getMainClass();
        
        OxideLaunch.log("Legacy launch: " + mainClassName);
        
        // Set game directory via reflection if possible
        if (config.getGameDir() != null) {
            setGameDirectory(mainClassName, config.getGameDir());
        }
        
        // Try applet launch first for very old versions
        if (shouldUseApplet(mainClassName)) {
            try {
                launchApplet(mainClassName);
                return;
            } catch (Throwable e) {
                OxideLaunch.log("Applet launch failed, falling back to main(): " + e.getMessage());
            }
        }
        
        // Fall back to standard main() invocation
        launchMain(mainClassName);
    }
    
    /**
     * Check if we should attempt applet launch.
     */
    private boolean shouldUseApplet(String mainClassName) {
        // Only use applet for the standard Minecraft main class or applet class
        return mainClassName.equals("net.minecraft.client.Minecraft") 
            || mainClassName.contains("Applet");
    }
    
    /**
     * Set the game directory via reflection.
     * 
     * Old Minecraft versions store the game directory in a static field
     * that needs to be set before launch.
     */
    private void setGameDirectory(String mainClassName, String gameDir) {
        try {
            Class<?> mainClass = Class.forName(mainClassName);
            Field gameDirField = findGameDirField(mainClass);
            
            if (gameDirField != null) {
                gameDirField.setAccessible(true);
                gameDirField.set(null, new File(gameDir));
                OxideLaunch.debug("Set game directory field to: " + gameDir);
            }
        } catch (Throwable e) {
            OxideLaunch.debug("Could not set game directory field: " + e.getMessage());
        }
        
        // Also set the system property that some versions check
        System.setProperty("minecraft.applet.TargetDirectory", gameDir);
        System.setProperty("user.dir", gameDir);
    }
    
    /**
     * Find the private static File field that holds the game directory.
     */
    private Field findGameDirField(Class<?> clazz) {
        for (Field field : clazz.getDeclaredFields()) {
            if (field.getType() != File.class) {
                continue;
            }
            
            int modifiers = field.getModifiers();
            
            // Looking for: private static File (not final)
            if (Modifier.isStatic(modifiers) 
                && Modifier.isPrivate(modifiers) 
                && !Modifier.isFinal(modifiers)) {
                return field;
            }
        }
        return null;
    }
    
    /**
     * Launch Minecraft as an Applet.
     */
    private void launchApplet(String mainClassName) throws Throwable {
        OxideLaunch.log("Launching as applet...");
        
        // Determine applet class
        String appletClassName = mainClassName.contains("Applet") 
            ? mainClassName 
            : DEFAULT_APPLET_CLASS;
        
        // Create the applet instance
        Class<?> appletClass = Class.forName(appletClassName);
        MethodHandle constructor = MethodHandles.lookup()
            .findConstructor(appletClass, MethodType.methodType(void.class));
        Applet gameApplet = (Applet) constructor.invoke();
        
        // Create our wrapper
        LegacyAppletWrapper wrapper = new LegacyAppletWrapper(gameApplet, config);
        
        // Create the frame
        Frame frame = new Frame("Minecraft");
        frame.setLayout(new BorderLayout());
        frame.add(wrapper, BorderLayout.CENTER);
        frame.setSize(config.getWidth(), config.getHeight());
        
        // Handle window close
        frame.addWindowListener(new WindowAdapter() {
            @Override
            public void windowClosing(WindowEvent e) {
                wrapper.stop();
                wrapper.destroy();
                frame.dispose();
                System.exit(0);
            }
        });
        
        frame.setLocationRelativeTo(null);
        frame.setVisible(true);
        
        // Start the applet
        wrapper.init();
        wrapper.start();
    }
    
    /**
     * Launch via main() method.
     */
    private void launchMain(String mainClassName) throws Throwable {
        String[] gameArgs = config.buildGameArgs();
        
        OxideLaunch.debug("Launching main(): " + mainClassName);
        
        MethodHandle mainMethod = ReflectionUtils.findMainMethod(mainClassName);
        mainMethod.invokeExact(gameArgs);
    }
    
    /**
     * Applet wrapper for legacy Minecraft.
     * 
     * This provides the AppletStub interface that Minecraft expects,
     * including document base URLs and applet parameters.
     */
    private static class LegacyAppletWrapper extends Applet implements java.applet.AppletStub {
        
        private final Applet wrappedApplet;
        private final Map<String, String> params = new HashMap<>();
        private final URL documentBase;
        private boolean active = false;
        
        LegacyAppletWrapper(Applet applet, LaunchConfig config) throws MalformedURLException {
            setLayout(new BorderLayout());
            add(applet, BorderLayout.CENTER);
            
            this.wrappedApplet = applet;
            
            // Set up document base URL
            String pkg = applet.getClass().getPackage().getName();
            if (pkg.startsWith("com.mojang")) {
                // Classic versions
                documentBase = new URL("http://www.minecraft.net:80/game/");
            } else {
                documentBase = new URL("http://www.minecraft.net/game/");
            }
            
            // Extract parameters from game args
            String[] args = config.getGameArgs().toArray(new String[0]);
            for (int i = 0; i < args.length - 1; i++) {
                if (args[i].startsWith("--")) {
                    String key = args[i].substring(2);
                    String value = args[i + 1];
                    if (!value.startsWith("--")) {
                        params.put(key, value);
                        i++;
                    }
                }
            }
            
            // Map some parameters to expected applet param names
            if (params.containsKey("username")) {
                params.put("userName", params.get("username"));
            }
            if (params.containsKey("accessToken")) {
                params.put("sessionId", params.get("accessToken"));
            }
        }
        
        @Override
        public void init() {
            wrappedApplet.setStub(this);
            wrappedApplet.init();
        }
        
        @Override
        public void start() {
            wrappedApplet.start();
            active = true;
        }
        
        @Override
        public void stop() {
            wrappedApplet.stop();
            active = false;
        }
        
        @Override
        public void destroy() {
            wrappedApplet.destroy();
        }
        
        @Override
        public boolean isActive() {
            return active;
        }
        
        @Override
        public URL getDocumentBase() {
            return documentBase;
        }
        
        @Override
        public URL getCodeBase() {
            try {
                return new URL("http://www.minecraft.net/game/");
            } catch (MalformedURLException e) {
                return null;
            }
        }
        
        @Override
        public String getParameter(String name) {
            String value = params.get(name);
            if (value != null) {
                return value;
            }
            
            // Check alternate names
            if ("username".equals(name)) {
                return params.get("userName");
            }
            if ("sessionid".equals(name)) {
                return params.get("sessionId");
            }
            
            return null;
        }
        
        @Override
        public void appletResize(int width, int height) {
            wrappedApplet.setSize(width, height);
        }
        
        @Override
        public java.applet.AppletContext getAppletContext() {
            return null; // Not needed for Minecraft
        }
    }
}
