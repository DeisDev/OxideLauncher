import { useEffect, useRef, useState } from "react";
import * as skinview3d from "skinview3d";
import { Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";

interface SkinViewer3DProps {
  /** Base64 data URL or HTTP URL for the skin */
  skinUrl: string | null;
  /** Base64 data URL or HTTP URL for the cape */
  capeUrl?: string | null;
  /** Skin model variant */
  variant?: "slim" | "classic";
  /** Width of the viewer */
  width?: number;
  /** Height of the viewer */
  height?: number;
  /** Enable auto-rotation */
  autoRotate?: boolean;
  /** Auto-rotation speed (degrees per second) */
  autoRotateSpeed?: number;
  /** Enable walking animation */
  walking?: boolean;
  /** Show name tag above character */
  nameTag?: string;
  /** CSS class name */
  className?: string;
  /** Enable zoom controls */
  enableZoom?: boolean;
  /** Initial zoom level (0.5 = zoomed out, 1 = normal, 2 = zoomed in) */
  zoom?: number;
  /** Fallback content when no skin is available */
  fallbackText?: string;
  /** Show cape on back equipment slot (default) or as elytra */
  backEquipment?: "cape" | "elytra";
}

export function SkinViewer3D({
  skinUrl,
  capeUrl,
  variant = "classic",
  width = 200,
  height = 300,
  autoRotate = true,
  autoRotateSpeed = 1.5,
  walking = false,
  nameTag,
  className,
  enableZoom = true,
  zoom = 0.9,
  fallbackText,
  backEquipment = "cape",
}: SkinViewer3DProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const viewerRef = useRef<skinview3d.SkinViewer | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  // Initialize the viewer
  useEffect(() => {
    if (!canvasRef.current) return;

    // Create the skin viewer
    const viewer = new skinview3d.SkinViewer({
      canvas: canvasRef.current,
      width,
      height,
      preserveDrawingBuffer: true,
    });

    // Configure initial settings
    viewer.autoRotate = autoRotate;
    viewer.autoRotateSpeed = autoRotateSpeed;
    viewer.zoom = zoom;
    viewer.fov = 50;

    // Enable orbit controls for drag-to-rotate
    viewer.controls.enableRotate = true;
    viewer.controls.enableZoom = enableZoom;
    viewer.controls.enablePan = false;

    // Set fullbright lighting (no shadows)
    viewer.globalLight.intensity = 1.0;
    viewer.cameraLight.intensity = 0.0;

    // Set a transparent/dark background
    viewer.background = null;

    viewerRef.current = viewer;

    return () => {
      viewer.dispose();
      viewerRef.current = null;
    };
  }, [width, height]);

  // Update auto-rotate settings
  useEffect(() => {
    if (viewerRef.current) {
      viewerRef.current.autoRotate = autoRotate;
      viewerRef.current.autoRotateSpeed = autoRotateSpeed;
    }
  }, [autoRotate, autoRotateSpeed]);

  // Update zoom
  useEffect(() => {
    if (viewerRef.current) {
      viewerRef.current.zoom = zoom;
    }
  }, [zoom]);

  // Update animation
  useEffect(() => {
    if (viewerRef.current) {
      if (walking) {
        viewerRef.current.animation = new skinview3d.WalkingAnimation();
        viewerRef.current.animation.speed = 0.8;
      } else {
        viewerRef.current.animation = new skinview3d.IdleAnimation();
      }
    }
  }, [walking]);

  // Update name tag
  useEffect(() => {
    if (viewerRef.current) {
      viewerRef.current.nameTag = nameTag || null;
    }
  }, [nameTag]);

  // Load skin when URL changes
  useEffect(() => {
    const viewer = viewerRef.current;
    if (!viewer) return;

    if (!skinUrl) {
      setLoading(false);
      setError(true);
      return;
    }

    setLoading(true);
    setError(false);

    // Determine model type
    const modelType = variant === "slim" ? "slim" : "default";

    viewer
      .loadSkin(skinUrl, { model: modelType })
      .then(() => {
        setLoading(false);
        setError(false);
      })
      .catch((err) => {
        console.error("Failed to load skin:", err);
        setLoading(false);
        setError(true);
      });
  }, [skinUrl, variant]);

  // Load cape when URL changes
  useEffect(() => {
    const viewer = viewerRef.current;
    if (!viewer) return;

    if (capeUrl) {
      viewer
        .loadCape(capeUrl, { backEquipment })
        .catch((err) => {
          console.error("Failed to load cape:", err);
        });
    } else {
      // Hide cape if no URL
      viewer.loadCape(null);
    }
  }, [capeUrl, backEquipment]);

  return (
    <div
      className={cn(
        "relative rounded-lg bg-muted/50 overflow-hidden",
        className
      )}
      style={{ width, height }}
    >
      <canvas
        ref={canvasRef}
        className={cn(
          "w-full h-full",
          (loading || error) && "opacity-0"
        )}
      />

      {/* Loading overlay */}
      {loading && (
        <div className="absolute inset-0 flex items-center justify-center bg-muted/80">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      )}

      {/* Error/Fallback state */}
      {error && !loading && (
        <div className="absolute inset-0 flex items-center justify-center bg-muted text-4xl font-bold text-muted-foreground">
          {fallbackText || "?"}
        </div>
      )}
    </div>
  );
}

/**
 * A smaller skin viewer optimized for showing just the face/head
 */
export function SkinFaceViewer({
  skinUrl,
  size = 64,
  className,
  fallbackText,
}: {
  skinUrl: string | null;
  size?: number;
  className?: string;
  fallbackText?: string;
}) {
  return (
    <SkinViewer3D
      skinUrl={skinUrl}
      width={size}
      height={size}
      autoRotate={false}
      enableZoom={false}
      zoom={2.5}
      className={cn("rounded-md", className)}
      fallbackText={fallbackText}
    />
  );
}

/**
 * Cape preview component - shows cape in 3D context
 */
export function CapeViewer3D({
  skinUrl,
  capeUrl,
  variant = "classic",
  width = 120,
  height = 180,
  className,
  backEquipment = "cape",
}: {
  skinUrl: string | null;
  capeUrl: string | null;
  variant?: "slim" | "classic";
  width?: number;
  height?: number;
  className?: string;
  backEquipment?: "cape" | "elytra";
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const viewerRef = useRef<skinview3d.SkinViewer | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  // Initialize viewer facing backwards to show cape
  useEffect(() => {
    if (!canvasRef.current) return;

    const viewer = new skinview3d.SkinViewer({
      canvas: canvasRef.current,
      width,
      height,
      preserveDrawingBuffer: true,
    });

    // Face backwards to show cape
    viewer.autoRotate = false;
    viewer.zoom = 0.85;
    viewer.fov = 50;

    // Rotate to show back
    viewer.playerObject.rotation.y = Math.PI; // 180 degrees

    viewer.controls.enableRotate = true;
    viewer.controls.enableZoom = false;
    viewer.controls.enablePan = false;

    // Fullbright lighting - no shadows
    viewer.globalLight.intensity = 1.0;
    viewer.cameraLight.intensity = 0.0;

    viewer.background = null;

    viewerRef.current = viewer;

    return () => {
      viewer.dispose();
      viewerRef.current = null;
    };
  }, [width, height]);

  // Load skin
  useEffect(() => {
    const viewer = viewerRef.current;
    if (!viewer || !skinUrl) {
      setLoading(false);
      return;
    }

    const modelType = variant === "slim" ? "slim" : "default";
    viewer.loadSkin(skinUrl, { model: modelType }).catch(console.error);
  }, [skinUrl, variant]);

  // Load cape
  useEffect(() => {
    const viewer = viewerRef.current;
    if (!viewer) return;

    if (capeUrl) {
      setLoading(true);
      viewer
        .loadCape(capeUrl, { backEquipment })
        .then(() => {
          setLoading(false);
          setError(false);
        })
        .catch((err) => {
          console.error("Failed to load cape:", err);
          setLoading(false);
          setError(true);
        });
    } else {
      setLoading(false);
      setError(true);
    }
  }, [capeUrl, backEquipment]);

  return (
    <div
      className={cn(
        "relative rounded-lg bg-muted/50 overflow-hidden",
        className
      )}
      style={{ width, height }}
    >
      <canvas
        ref={canvasRef}
        className={cn("w-full h-full", (loading || error) && "opacity-0")}
      />

      {loading && (
        <div className="absolute inset-0 flex items-center justify-center bg-muted/80">
          <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        </div>
      )}

      {error && !loading && (
        <div className="absolute inset-0 flex items-center justify-center bg-muted text-muted-foreground text-sm">
          No Cape
        </div>
      )}
    </div>
  );
}
