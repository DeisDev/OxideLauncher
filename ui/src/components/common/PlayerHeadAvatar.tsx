// Player head avatar component for displaying Minecraft player heads
//
// Oxide Launcher — A Rust-based Minecraft launcher
// Copyright (C) 2025 Oxide Launcher contributors
//
// This file is part of Oxide Launcher.
//
// Oxide Launcher is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Oxide Launcher is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { cn } from "@/lib/utils";

interface PlayerHeadAvatarProps {
  /** Player UUID for loading cached skin */
  uuid: string | null;
  /** Skin URL from the account info */
  skinUrl: string | null;
  /** Size of the avatar (square) */
  size?: number;
  /** Fallback text to display if skin loading fails */
  fallbackText: string;
  /** Additional CSS classes */
  className?: string;
}

/**
 * Renders a 2D player head avatar cropped from the skin texture.
 * The face is at position (8,8) with size 8x8 in the standard Minecraft skin.
 * The hat overlay is at position (40,8) with size 8x8.
 */
export function PlayerHeadAvatar({
  uuid,
  skinUrl,
  size = 48,
  fallbackText,
  className,
}: PlayerHeadAvatarProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState(false);

  useEffect(() => {
    setLoaded(false);
    setError(false);

    if (!uuid && !skinUrl) {
      setError(true);
      return;
    }

    const loadSkin = async () => {
      try {
        // First try to get the cached skin path
        let skinDataUrl: string | null = null;
        
        if (uuid) {
          const cachedPath = await invoke<string | null>("get_cached_skin_path", { uuid });
          if (cachedPath) {
            // Use Tauri's asset protocol to load the cached file
            skinDataUrl = convertFileSrc(cachedPath);
          }
        }

        // If no cached skin but we have a URL, download it as base64
        if (!skinDataUrl && skinUrl) {
          try {
            const base64 = await invoke<string>("download_skin_image", { skinUrl });
            skinDataUrl = `data:image/png;base64,${base64}`;
            
            // Cache it for future use if we have a UUID
            if (uuid) {
              invoke("cache_skin_image", { uuid, skinUrl }).catch(console.error);
            }
          } catch (e) {
            console.error("Failed to download skin:", e);
            setError(true);
            return;
          }
        }

        if (!skinDataUrl) {
          setError(true);
          return;
        }

        // Load and crop the skin
        const img = new Image();
        img.crossOrigin = "anonymous";
        
        img.onload = () => {
          const canvas = canvasRef.current;
          if (!canvas) return;
          
          const ctx = canvas.getContext("2d");
          if (!ctx) return;

          // Clear canvas
          ctx.clearRect(0, 0, size, size);
          
          // Disable image smoothing for crisp pixel art
          ctx.imageSmoothingEnabled = false;

          // Draw the face (8x8 from position 8,8 in the skin)
          // Scale it to fill the canvas
          ctx.drawImage(
            img,
            8, 8,    // Source x, y
            8, 8,    // Source width, height
            0, 0,    // Dest x, y
            size, size // Dest width, height
          );

          // Draw the hat/overlay layer (8x8 from position 40,8 in the skin)
          // This adds the hat layer on top for skins that have it
          ctx.drawImage(
            img,
            40, 8,   // Source x, y
            8, 8,    // Source width, height
            0, 0,    // Dest x, y
            size, size // Dest width, height
          );

          setLoaded(true);
        };

        img.onerror = () => {
          console.error("Failed to load skin image");
          setError(true);
        };

        img.src = skinDataUrl;
      } catch (e) {
        console.error("Error loading player head:", e);
        setError(true);
      }
    };

    loadSkin();
  }, [uuid, skinUrl, size]);

  return (
    <div
      className={cn(
        "rounded-full bg-muted flex items-center justify-center text-lg font-semibold overflow-hidden",
        className
      )}
      style={{ width: size, height: size }}
    >
      {/* Canvas for rendering the cropped head */}
      <canvas
        ref={canvasRef}
        width={size}
        height={size}
        className={cn(
          "w-full h-full",
          (!loaded || error) && "hidden"
        )}
      />
      
      {/* Fallback text when loading fails or not ready */}
      {(error || !loaded) && (
        <span className="text-muted-foreground">
          {fallbackText}
        </span>
      )}
    </div>
  );
}
