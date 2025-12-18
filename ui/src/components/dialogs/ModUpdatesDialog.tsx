/**
 * Mod Updates Dialog
 * 
 * Displays available mod updates and allows users to update mods
 * Uses the RustWiz metadata system for update checking
 */

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  ArrowUpCircle,
  Check,
  Download,
  Loader2,
  PackageCheck,
  RefreshCw,
  AlertCircle,
  FileWarning,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Checkbox } from "@/components/ui/checkbox";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { cn } from "@/lib/utils";

export interface UpdateCheckResult {
  filename: string;
  current_version: string;
  latest_version: string | null;
  latest_version_id: string | null;
  update_available: boolean;
  platform: string;
  changelog: string | null;
}

export interface BatchUpdateResult {
  updates_available: UpdateCheckResult[];
  up_to_date: string[];
  unchecked: string[];
  errors: string[];
}

interface ModUpdatesDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  instanceId: string;
  instanceName: string;
  onModsUpdated?: () => void;
}

export function ModUpdatesDialog({
  open,
  onOpenChange,
  instanceId,
  instanceName,
  onModsUpdated,
}: ModUpdatesDialogProps) {
  const [checking, setChecking] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [results, setResults] = useState<BatchUpdateResult | null>(null);
  const [selectedMods, setSelectedMods] = useState<Set<string>>(new Set());
  const [updateProgress, setUpdateProgress] = useState<Map<string, "pending" | "updating" | "success" | "error">>(new Map());
  const [error, setError] = useState<string | null>(null);

  const checkForUpdates = async () => {
    setChecking(true);
    setError(null);
    setResults(null);
    setSelectedMods(new Set());
    setUpdateProgress(new Map());

    try {
      const result = await invoke<BatchUpdateResult>("check_mod_updates", {
        instanceId,
      });
      setResults(result);
      
      // Auto-select all mods with available updates
      const modsWithUpdates = new Set(result.updates_available.map(u => u.filename));
      setSelectedMods(modsWithUpdates);
    } catch (err) {
      setError(`Failed to check for updates: ${err}`);
    } finally {
      setChecking(false);
    }
  };

  const toggleModSelection = (filename: string) => {
    setSelectedMods(prev => {
      const next = new Set(prev);
      if (next.has(filename)) {
        next.delete(filename);
      } else {
        next.add(filename);
      }
      return next;
    });
  };

  const toggleAllMods = () => {
    if (!results) return;
    
    const allMods = results.updates_available.map(u => u.filename);
    if (selectedMods.size === allMods.length) {
      setSelectedMods(new Set());
    } else {
      setSelectedMods(new Set(allMods));
    }
  };

  const updateSelectedMods = async () => {
    if (!results || selectedMods.size === 0) return;

    setUpdating(true);
    
    // Initialize progress for all selected mods
    const initialProgress = new Map<string, "pending" | "updating" | "success" | "error">();
    selectedMods.forEach(filename => initialProgress.set(filename, "pending"));
    setUpdateProgress(initialProgress);

    // Update mods one by one
    for (const update of results.updates_available) {
      if (!selectedMods.has(update.filename)) continue;
      
      // Set as updating
      setUpdateProgress(prev => new Map(prev).set(update.filename, "updating"));
      
      try {
        // Download the updated mod version
        await invoke("download_mod_version", {
          instanceId,
          platform: update.platform.toLowerCase(),
          versionId: update.latest_version_id,
          filename: update.filename,
        });
        
        setUpdateProgress(prev => new Map(prev).set(update.filename, "success"));
      } catch (err) {
        console.error(`Failed to update ${update.filename}:`, err);
        setUpdateProgress(prev => new Map(prev).set(update.filename, "error"));
      }
    }

    setUpdating(false);
    
    // Notify parent to refresh mods list
    if (onModsUpdated) {
      onModsUpdated();
    }
  };

  const handleClose = () => {
    if (!updating) {
      onOpenChange(false);
      // Reset state when closing
      setResults(null);
      setError(null);
      setSelectedMods(new Set());
      setUpdateProgress(new Map());
    }
  };

  // Calculate summary stats
  const successCount = Array.from(updateProgress.values()).filter(s => s === "success").length;
  const errorCount = Array.from(updateProgress.values()).filter(s => s === "error").length;

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[600px] max-h-[80vh] flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <ArrowUpCircle className="h-5 w-5 text-primary" />
            Mod Updates
          </DialogTitle>
          <DialogDescription>
            Check and update mods for {instanceName}
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 min-h-0 py-4">
          {/* Initial state - no check performed yet */}
          {!checking && !results && !error && (
            <div className="flex flex-col items-center justify-center py-8 text-center">
              <RefreshCw className="h-12 w-12 text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2">Check for Updates</h3>
              <p className="text-sm text-muted-foreground mb-4 max-w-sm">
                Check if any of your mods have newer versions available on Modrinth or CurseForge.
              </p>
              <Button onClick={checkForUpdates}>
                <RefreshCw className="mr-2 h-4 w-4" />
                Check for Updates
              </Button>
            </div>
          )}

          {/* Loading state */}
          {checking && (
            <div className="flex flex-col items-center justify-center py-8">
              <Loader2 className="h-8 w-8 animate-spin text-primary mb-4" />
              <p className="text-sm text-muted-foreground">
                Checking for mod updates...
              </p>
            </div>
          )}

          {/* Error state */}
          {error && (
            <div className="flex flex-col items-center justify-center py-8 text-center">
              <AlertCircle className="h-12 w-12 text-destructive mb-4" />
              <h3 className="text-lg font-medium mb-2">Error</h3>
              <p className="text-sm text-muted-foreground mb-4">{error}</p>
              <Button variant="outline" onClick={checkForUpdates}>
                <RefreshCw className="mr-2 h-4 w-4" />
                Try Again
              </Button>
            </div>
          )}

          {/* Results state */}
          {results && !error && (
            <div className="flex flex-col h-full">
              {/* Summary badges */}
              <div className="flex flex-wrap gap-2 mb-4">
                <Badge variant="default" className="flex items-center gap-1">
                  <ArrowUpCircle className="h-3 w-3" />
                  {results.updates_available.length} updates available
                </Badge>
                <Badge variant="secondary" className="flex items-center gap-1">
                  <Check className="h-3 w-3" />
                  {results.up_to_date.length} up to date
                </Badge>
                {results.unchecked.length > 0 && (
                  <Badge variant="outline" className="flex items-center gap-1">
                    <FileWarning className="h-3 w-3" />
                    {results.unchecked.length} unchecked
                  </Badge>
                )}
                {results.errors.length > 0 && (
                  <Badge variant="destructive" className="flex items-center gap-1">
                    <AlertCircle className="h-3 w-3" />
                    {results.errors.length} errors
                  </Badge>
                )}
              </div>

              {/* Update progress summary */}
              {updateProgress.size > 0 && (
                <div className="flex flex-wrap gap-2 mb-4">
                  {successCount > 0 && (
                    <Badge variant="default" className="bg-green-500">
                      {successCount} updated
                    </Badge>
                  )}
                  {errorCount > 0 && (
                    <Badge variant="destructive">
                      {errorCount} failed
                    </Badge>
                  )}
                </div>
              )}

              {/* Updates list */}
              {results.updates_available.length > 0 ? (
                <ScrollArea className="flex-1">
                  <div className="space-y-2 pr-4">
                    {/* Select all header */}
                    <div className="flex items-center gap-2 p-2 border-b">
                      <Checkbox
                        checked={selectedMods.size === results.updates_available.length}
                        onCheckedChange={toggleAllMods}
                        disabled={updating}
                      />
                      <span className="text-sm font-medium">Select All</span>
                    </div>

                    {results.updates_available.map((update) => {
                      const status = updateProgress.get(update.filename);
                      
                      return (
                        <Accordion key={update.filename} type="single" collapsible>
                          <AccordionItem value={update.filename} className="border rounded-md px-2">
                            <div className="flex items-center gap-2 py-2">
                              <Checkbox
                                checked={selectedMods.has(update.filename)}
                                onCheckedChange={() => toggleModSelection(update.filename)}
                                disabled={updating || status === "success"}
                              />
                              
                              {/* Status indicator */}
                              {status === "updating" && (
                                <Loader2 className="h-4 w-4 animate-spin text-primary" />
                              )}
                              {status === "success" && (
                                <Check className="h-4 w-4 text-green-500" />
                              )}
                              {status === "error" && (
                                <X className="h-4 w-4 text-destructive" />
                              )}
                              
                              <AccordionTrigger className="flex-1 hover:no-underline py-0">
                                <div className="flex items-center gap-2 text-left">
                                  <span className={cn(
                                    "font-medium",
                                    status === "success" && "line-through text-muted-foreground"
                                  )}>
                                    {update.filename.replace(".jar", "")}
                                  </span>
                                  <Badge variant="outline" className="text-xs">
                                    {update.platform}
                                  </Badge>
                                </div>
                              </AccordionTrigger>
                              
                              <div className="flex items-center gap-2 text-xs text-muted-foreground mr-4">
                                <span>{update.current_version}</span>
                                <span>→</span>
                                <span className="text-primary font-medium">
                                  {update.latest_version}
                                </span>
                              </div>
                            </div>
                            
                            {update.changelog && (
                              <AccordionContent>
                                <div className="pl-8 pb-2">
                                  <p className="text-xs text-muted-foreground font-medium mb-1">
                                    Changelog:
                                  </p>
                                  <p className="text-xs text-muted-foreground whitespace-pre-wrap line-clamp-4">
                                    {update.changelog}
                                  </p>
                                </div>
                              </AccordionContent>
                            )}
                          </AccordionItem>
                        </Accordion>
                      );
                    })}
                  </div>
                </ScrollArea>
              ) : (
                <div className="flex flex-col items-center justify-center py-8 text-center">
                  <PackageCheck className="h-12 w-12 text-green-500 mb-4" />
                  <h3 className="text-lg font-medium mb-2">All mods are up to date!</h3>
                  <p className="text-sm text-muted-foreground">
                    No updates available for your mods.
                  </p>
                </div>
              )}

              {/* Errors accordion */}
              {results.errors.length > 0 && (
                <Accordion type="single" collapsible className="mt-4">
                  <AccordionItem value="errors" className="border-destructive/50">
                    <AccordionTrigger className="text-destructive">
                      <div className="flex items-center gap-2">
                        <AlertCircle className="h-4 w-4" />
                        {results.errors.length} errors occurred
                      </div>
                    </AccordionTrigger>
                    <AccordionContent>
                      <ul className="text-xs text-muted-foreground space-y-1 pl-6">
                        {results.errors.map((err, idx) => (
                          <li key={idx}>• {err}</li>
                        ))}
                      </ul>
                    </AccordionContent>
                  </AccordionItem>
                </Accordion>
              )}
            </div>
          )}
        </div>

        <DialogFooter className="flex gap-2">
          {results && results.updates_available.length > 0 && (
            <>
              <Button 
                variant="outline" 
                onClick={checkForUpdates}
                disabled={checking || updating}
              >
                <RefreshCw className="mr-2 h-4 w-4" />
                Re-check
              </Button>
              <Button
                onClick={updateSelectedMods}
                disabled={selectedMods.size === 0 || updating}
              >
                {updating ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Updating...
                  </>
                ) : (
                  <>
                    <Download className="mr-2 h-4 w-4" />
                    Update Selected ({selectedMods.size})
                  </>
                )}
              </Button>
            </>
          )}
          <Button variant="ghost" onClick={handleClose} disabled={updating}>
            Close
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
