/*
 * Utility class for reflection operations
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

package dev.oxide.launch.util;

import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;

/**
 * Utility class for reflection operations.
 */
public final class ReflectionUtils {
    
    private ReflectionUtils() {}
    
    /**
     * Find the main method of a class.
     * 
     * @param className The fully qualified class name
     * @return A MethodHandle to the main method
     * @throws Throwable If the class or method cannot be found
     */
    public static MethodHandle findMainMethod(String className) throws Throwable {
        Class<?> clazz = Class.forName(className);
        return findMainMethod(clazz);
    }
    
    /**
     * Find the main method of a class.
     * 
     * @param clazz The class to search
     * @return A MethodHandle to the main method
     * @throws Throwable If the method cannot be found
     */
    public static MethodHandle findMainMethod(Class<?> clazz) throws Throwable {
        MethodHandles.Lookup lookup = MethodHandles.lookup();
        MethodType mainType = MethodType.methodType(void.class, String[].class);
        
        return lookup.findStatic(clazz, "main", mainType);
    }
    
    /**
     * Get a class by name, or null if not found.
     * 
     * @param className The fully qualified class name
     * @return The class, or null if not found
     */
    public static Class<?> getClassOrNull(String className) {
        try {
            return Class.forName(className);
        } catch (ClassNotFoundException e) {
            return null;
        }
    }
    
    /**
     * Check if a class exists on the classpath.
     * 
     * @param className The fully qualified class name
     * @return True if the class exists
     */
    public static boolean classExists(String className) {
        return getClassOrNull(className) != null;
    }
}
