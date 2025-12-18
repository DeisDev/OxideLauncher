import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open as openExternal } from "@tauri-apps/plugin-shell";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import {
  ExternalLink,
  Check,
  X,
  Loader2,
  FolderOpen,
  RefreshCw,
  AlertCircle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { cn } from "@/lib/utils";

export interface BlockedMod {
  name: string;
  website_url: string;
  hash: string | null;
  hash_algo: string | null;
  filename: string;
  project_id: number;
  file_id: number;
  target_folder: string;
  matched: boolean;
  local_path: string | null;
}

interface BlockedModsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  blockedMods: BlockedMod[];
  instanceId: string;
  onContinue: () => void;
  onSkip: () => void;
}

export function BlockedModsDialog({
  open,
  onOpenChange,
  blockedMods: initialBlockedMods,
  instanceId,
  onContinue,
  onSkip,
}: BlockedModsDialogProps) {
  const [blockedMods, setBlockedMods] = useState<BlockedMod[]>(initialBlockedMods);
  const [additionalPaths, setAdditionalPaths] = useState<string[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [isWatching, setIsWatching] = useState(false);
  const [downloadsDir, setDownloadsDir] = useState<string>("");
  const [error, setError] = useState<string | null>(null);

  const allMatched = blockedMods.every((mod) => mod.matched);
  const matchedCount = blockedMods.filter((mod) => mod.matched).length;
  const totalCount = blockedMods.length;

  // Get downloads directory on mount
  useEffect(() => {
    if (open) {
      invoke<string>("get_downloads_dir")
        .then(setDownloadsDir)
        .catch((err) => console.error("Failed to get downloads dir:", err));
    }
  }, [open]);

  // Setup file watcher when dialog opens
  useEffect(() => {
    if (!open || blockedMods.length === 0) return;

    const sessionId = `blocked-mods-${instanceId}-${Date.now()}`;

    // Start the watcher
    invoke("start_blocked_mods_watcher", {
      sessionId,
      blockedMods,
      additionalPaths,
    })
      .then(() => {
        setIsWatching(true);
        // Immediately scan for files when watcher starts
        invoke<BlockedMod[]>("scan_for_blocked_mod_files", {
          blockedMods,
          additionalPaths,
        })
          .then(setBlockedMods)
          .catch((err) => console.error("Initial scan failed:", err));
      })
      .catch((err) => {
        console.error("Failed to start watcher:", err);
        setError(`Failed to watch for files: ${err}`);
      });

    // Listen for updates
    const unlisten = listen<{
      session_id: string;
      blocked_mods: BlockedMod[];
      all_matched: boolean;
    }>("blocked-mods-updated", (event) => {
      if (event.payload.session_id === sessionId) {
        setBlockedMods(event.payload.blocked_mods);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
      setIsWatching(false);
    };
  }, [open, instanceId, additionalPaths]);

  // Update local state when props change
  useEffect(() => {
    setBlockedMods(initialBlockedMods);
  }, [initialBlockedMods]);

  const handleOpenUrl = useCallback((url: string) => {
    openExternal(url);
  }, []);

  const handleOpenAllMissing = useCallback(() => {
    blockedMods
      .filter((mod) => !mod.matched)
      .forEach((mod) => openExternal(mod.website_url));
  }, [blockedMods]);

  const handleAddFolder = useCallback(async () => {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "Select folder to watch for downloaded mods",
      });

      if (selected && typeof selected === "string") {
        setAdditionalPaths((prev) => [...prev, selected]);
        // Trigger a rescan
        handleManualScan();
      }
    } catch (err) {
      console.error("Failed to select folder:", err);
    }
  }, []);

  const handleManualScan = useCallback(async () => {
    setIsScanning(true);
    setError(null);

    try {
      const updated = await invoke<BlockedMod[]>("scan_for_blocked_mod_files", {
        blockedMods,
        additionalPaths,
      });
      setBlockedMods(updated);
    } catch (err) {
      console.error("Failed to scan for files:", err);
      setError(`Scan failed: ${err}`);
    } finally {
      setIsScanning(false);
    }
  }, [blockedMods, additionalPaths]);

  const handleContinue = useCallback(async () => {
    if (!allMatched) {
      // Skip - just close without copying
      onSkip();
      return;
    }

    try {
      await invoke("copy_blocked_mods_to_instance", {
        instanceId,
        blockedMods: blockedMods.filter((mod) => mod.matched),
      });
      onContinue();
    } catch (err) {
      console.error("Failed to copy mods:", err);
      setError(`Failed to copy mods: ${err}`);
    }
  }, [allMatched, instanceId, blockedMods, onContinue, onSkip]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent 
        className="max-w-2xl max-h-[80vh] flex flex-col"
        onPointerDownOutside={(e) => e.preventDefault()}
        onEscapeKeyDown={(e) => e.preventDefault()}
        onInteractOutside={(e) => e.preventDefault()}
      >
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <AlertCircle className="h-5 w-5 text-yellow-500" />
            Manual Download Required
          </DialogTitle>
          <DialogDescription>
            Some mods in this modpack cannot be downloaded automatically due to CurseForge restrictions.
            Please download them manually by clicking the links below.
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-hidden">
          {/* Status bar */}
          <div className="flex items-center justify-between mb-3 text-sm">
            <div className="flex items-center gap-2">
              {isWatching && (
                <span className="flex items-center gap-1 text-muted-foreground">
                  <Loader2 className="h-3 w-3 animate-spin" />
                  Watching for downloads...
                </span>
              )}
            </div>
            <div
              className={cn(
                "font-medium",
                allMatched ? "text-green-500" : "text-muted-foreground"
              )}
            >
              {allMatched ? (
                <span className="flex items-center gap-1">
                  <Check className="h-4 w-4" />
                  All mods found!
                </span>
              ) : (
                `${matchedCount} / ${totalCount} found`
              )}
            </div>
          </div>

          {/* Watched directories */}
          <div className="text-xs text-muted-foreground mb-2">
            Watching: {downloadsDir}
            {additionalPaths.length > 0 && `, ${additionalPaths.join(", ")}`}
          </div>

          {/* Mods list */}
          <ScrollArea className="h-[300px] rounded-md border">
            <div className="p-4 pr-6 space-y-3">
              {blockedMods.map((mod) => (
                <div
                  key={`${mod.project_id}-${mod.file_id}`}
                  className={cn(
                    "flex items-start gap-3 p-3 rounded-lg",
                    mod.matched ? "bg-green-500/10" : "bg-muted/50"
                  )}
                >
                  {/* Status icon */}
                  <div className="mt-0.5 shrink-0">
                    {mod.matched ? (
                      <Check className="h-5 w-5 text-green-500" />
                    ) : (
                      <X className="h-5 w-5 text-muted-foreground" />
                    )}
                  </div>

                  {/* Mod info */}
                  <div className="flex-1 min-w-0 overflow-hidden">
                    <div className="font-medium break-words [overflow-wrap:anywhere]">{mod.name}</div>
                    <div className="text-sm text-muted-foreground break-all">
                      {mod.filename}
                    </div>
                    {mod.matched && mod.local_path && (
                      <div className="text-xs text-green-600 break-all mt-1">
                        Found: {mod.local_path}
                      </div>
                    )}
                  </div>

                  {/* Actions */}
                  <div className="flex items-center gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleOpenUrl(mod.website_url)}
                      disabled={mod.matched}
                    >
                      <ExternalLink className="h-4 w-4 mr-1" />
                      Download
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </ScrollArea>

          {/* Error message */}
          {error && (
            <div className="mt-3 p-3 rounded-lg bg-destructive/10 text-destructive text-sm">
              {error}
            </div>
          )}
        </div>

        <DialogFooter className="flex-col sm:flex-row gap-2">
          <div className="flex gap-2 flex-1">
            <Button variant="outline" onClick={handleOpenAllMissing} disabled={allMatched}>
              Open All Missing
            </Button>
            <Button variant="outline" onClick={handleAddFolder}>
              <FolderOpen className="h-4 w-4 mr-1" />
              Add Folder
            </Button>
            <Button variant="outline" onClick={handleManualScan} disabled={isScanning}>
              {isScanning ? (
                <Loader2 className="h-4 w-4 mr-1 animate-spin" />
              ) : (
                <RefreshCw className="h-4 w-4 mr-1" />
              )}
              Rescan
            </Button>
          </div>
          <div className="flex gap-2">
            <Button
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </Button>
            <Button onClick={handleContinue}>
              {allMatched ? "Continue" : "Skip Missing"}
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
