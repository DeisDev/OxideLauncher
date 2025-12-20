// Resource packs tab component for managing instance resource packs
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

import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Trash2, RefreshCw, Package, FolderOpen, Download, Plus, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { openDialogWindow, WINDOW_LABELS } from "@/lib/windowManager";
import type { ResourcePackInfo, InstanceInfo } from "../types";

interface ResourcePacksTabProps {
  instanceId: string;
  instance?: InstanceInfo | null;
}

export function ResourcePacksTab({ instanceId, instance }: ResourcePacksTabProps) {
  const [resourcePacks, setResourcePacks] = useState<ResourcePackInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [deleteDialog, setDeleteDialog] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const dropZoneRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadResourcePacks();
    
    // Listen for resourcepacks-changed event from resource browser window
    const unlistenPromise = listen<{ instanceId: string }>("resourcepacks-changed", (event) => {
      if (event.payload.instanceId === instanceId) {
        loadResourcePacks();
      }
    });
    
    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, [instanceId]);

  const loadResourcePacks = async () => {
    setLoading(true);
    try {
      const packs = await invoke<ResourcePackInfo[]>("list_resource_packs", { instanceId });
      setResourcePacks(packs);
    } catch (error) {
      console.error("Failed to load resource packs:", error);
    } finally {
      setLoading(false);
    }
  };

  const deleteResourcePack = async (filename: string) => {
    try {
      await invoke("delete_resource_pack", { instanceId, filename });
      await loadResourcePacks();
    } catch (error) {
      console.error("Failed to delete resource pack:", error);
      alert("Failed to delete resource pack: " + error);
    }
    setDeleteDialog(null);
  };

  const openFolder = async () => {
    try {
      await invoke("open_resourcepacks_folder", { instanceId });
    } catch (error) {
      console.error("Failed to open folder:", error);
      alert("Failed to open folder: " + error);
    }
  };

  const handleAddFile = async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [
          { name: "Resource Packs", extensions: ["zip"] },
        ],
      });
      
      if (selected) {
        const files = Array.isArray(selected) ? selected : [selected];
        for (const file of files) {
          await invoke("add_local_resource_pack", { instanceId, filePath: file });
        }
        await loadResourcePacks();
      }
    } catch (error) {
      console.error("Failed to add resource pack:", error);
      alert("Failed to add resource pack: " + error);
    }
  };

  // Helper function to convert ArrayBuffer to base64 (handles large files)
  const arrayBufferToBase64 = (buffer: ArrayBuffer): string => {
    const uint8Array = new Uint8Array(buffer);
    const chunkSize = 8192;
    let result = '';
    for (let i = 0; i < uint8Array.length; i += chunkSize) {
      const chunk = uint8Array.subarray(i, Math.min(i + chunkSize, uint8Array.length));
      result += String.fromCharCode.apply(null, Array.from(chunk));
    }
    return btoa(result);
  };

  // Process dropped file paths from Tauri's drag-drop event
  const processDroppedPaths = useCallback(async (paths: string[]) => {
    const zipFiles = paths.filter(p => p.toLowerCase().endsWith('.zip'));
    if (zipFiles.length === 0) {
      alert('Only .zip files are supported for resource packs');
      return;
    }
    
    for (const filePath of zipFiles) {
      try {
        await invoke("add_local_resource_pack", { instanceId, filePath });
      } catch (error) {
        console.error("Failed to add resource pack:", error);
        alert(`Failed to add resource pack: ${error}`);
      }
    }
    await loadResourcePacks();
  }, [instanceId]);

  // Set up Tauri drag-drop event listener
  useEffect(() => {
    const webview = getCurrentWebviewWindow();
    let unlisten: (() => void) | undefined;

    webview.onDragDropEvent((event) => {
      if (event.payload.type === 'over' || event.payload.type === 'enter') {
        setIsDragging(true);
      } else if (event.payload.type === 'drop') {
        setIsDragging(false);
        if (event.payload.paths && event.payload.paths.length > 0) {
          processDroppedPaths(event.payload.paths);
        }
      } else if (event.payload.type === 'leave') {
        setIsDragging(false);
      }
    }).then(fn => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [processDroppedPaths]);

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.currentTarget === dropZoneRef.current && !dropZoneRef.current?.contains(e.relatedTarget as Node)) {
      setIsDragging(false);
    }
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
    
    const files = Array.from(e.dataTransfer.files);
    const zipFiles = files.filter(file => 
      file.name.toLowerCase().endsWith('.zip')
    );
    
    if (zipFiles.length === 0) {
      // Files might come from Tauri event instead
      return;
    }
    
    // Process each file (fallback for HTML5 drag-drop)
    for (const file of zipFiles) {
      try {
        const arrayBuffer = await file.arrayBuffer();
        const base64 = arrayBufferToBase64(arrayBuffer);
        
        await invoke("add_local_resource_pack_from_bytes", {
          instanceId,
          filename: file.name,
          data: base64,
        });
      } catch (error) {
        console.error("Failed to add resource pack:", error);
        alert(`Failed to add ${file.name}: ${error}`);
      }
    }
    
    await loadResourcePacks();
  }, [instanceId]);

  return (
    <>
      <Card 
        ref={dropZoneRef}
        className={cn(
          "h-full transition-colors",
          isDragging && "border-primary border-2 bg-primary/5"
        )}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
      >
        <CardHeader className="pb-3">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
            <div>
              <CardTitle>Resource Packs</CardTitle>
              <CardDescription>Manage resource packs for this instance</CardDescription>
            </div>
            <div className="flex flex-wrap gap-2">
              <Button 
                variant="default" 
                size="sm" 
                onClick={() => instance && openDialogWindow(WINDOW_LABELS.RESOURCE_BROWSER, {
                  instanceId,
                  instanceName: instance.name,
                  minecraftVersion: instance.minecraft_version,
                  resourceType: "resourcepack",
                })}
                disabled={!instance}
              >
                <Download className="h-4 w-4 sm:mr-2" />
                <span className="hidden sm:inline">Download</span>
              </Button>
              <Button variant="outline" size="sm" onClick={handleAddFile}>
                <Plus className="h-4 w-4 sm:mr-2" />
                <span className="hidden sm:inline">Add File</span>
              </Button>
              <Button variant="outline" size="sm" onClick={openFolder}>
                <FolderOpen className="h-4 w-4 sm:mr-2" />
                <span className="hidden sm:inline">Open Folder</span>
              </Button>
              <Button variant="outline" size="sm" onClick={loadResourcePacks} disabled={loading}>
                <RefreshCw className={cn("h-4 w-4 sm:mr-2", loading && "animate-spin")} />
                <span className="hidden sm:inline">Refresh</span>
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {isDragging ? (
            <div className="flex flex-col items-center justify-center py-16 border-2 border-dashed border-primary rounded-lg">
              <Upload className="h-12 w-12 text-primary mb-4" />
              <p className="text-lg font-medium text-primary">Drop resource packs here</p>
              <p className="text-sm text-muted-foreground">.zip files only</p>
            </div>
          ) : loading ? (
          <div className="flex items-center justify-center py-8">
            <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
            <span className="ml-2 text-muted-foreground">Loading resource packs...</span>
          </div>
        ) : resourcePacks.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <Package className="h-12 w-12 mx-auto mb-4 opacity-50" />
              <p>No resource packs found.</p>
              <p className="text-sm mt-2">Download from Modrinth/CurseForge or add files manually.</p>
            </div>
        ) : (
          <ScrollArea className="h-[400px]">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Size</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {resourcePacks.map((pack) => (
                  <TableRow key={pack.filename}>
                    <TableCell className="font-medium">
                      <div className="flex items-center gap-3">
                        {pack.icon_path ? (
                          <img
                            src={convertFileSrc(pack.icon_path)}
                            alt=""
                            className="h-8 w-8 rounded object-cover flex-shrink-0"
                            onError={(e) => {
                              // Replace with fallback icon on error
                              (e.target as HTMLImageElement).style.display = 'none';
                              (e.target as HTMLImageElement).nextElementSibling?.classList.remove('hidden');
                            }}
                          />
                        ) : null}
                        <Package className={cn("h-8 w-8 text-muted-foreground flex-shrink-0", pack.icon_path && "hidden")} />
                        <div className="min-w-0">
                          <div className="truncate">{pack.name}</div>
                          {pack.description && (
                            <TooltipProvider>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <div className="text-xs text-muted-foreground truncate max-w-[300px]">
                                    {pack.description}
                                  </div>
                                </TooltipTrigger>
                                <TooltipContent side="bottom" className="max-w-[400px]">
                                  <p>{pack.description}</p>
                                </TooltipContent>
                              </Tooltip>
                            </TooltipProvider>
                          )}
                        </div>
                      </div>
                    </TableCell>
                    <TableCell>{pack.size}</TableCell>
                    <TableCell className="text-right">
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setDeleteDialog(pack.filename)}
                        title="Delete resource pack"
                      >
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </ScrollArea>
        )}
        </CardContent>
      </Card>

      <AlertDialog open={!!deleteDialog} onOpenChange={() => setDeleteDialog(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Resource Pack</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete "{deleteDialog}"? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => deleteDialog && deleteResourcePack(deleteDialog)}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
