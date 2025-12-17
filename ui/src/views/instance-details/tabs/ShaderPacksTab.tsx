import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
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
import { cn } from "@/lib/utils";
import { ResourceDownloadDialog } from "@/components/dialogs/ResourceDownloadDialog";
import type { ShaderPackInfo, InstanceInfo } from "../types";

interface ShaderPacksTabProps {
  instanceId: string;
  instance?: InstanceInfo | null;
}

export function ShaderPacksTab({ instanceId, instance }: ShaderPacksTabProps) {
  const [shaderPacks, setShaderPacks] = useState<ShaderPackInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [deleteDialog, setDeleteDialog] = useState<string | null>(null);
  const [showDownloadDialog, setShowDownloadDialog] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const dropZoneRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadShaderPacks();
  }, [instanceId]);

  const loadShaderPacks = async () => {
    setLoading(true);
    try {
      const packs = await invoke<ShaderPackInfo[]>("list_shader_packs", { instanceId });
      setShaderPacks(packs);
    } catch (error) {
      console.error("Failed to load shader packs:", error);
    } finally {
      setLoading(false);
    }
  };

  const deleteShaderPack = async (filename: string) => {
    try {
      await invoke("delete_shader_pack", { instanceId, filename });
      await loadShaderPacks();
    } catch (error) {
      console.error("Failed to delete shader pack:", error);
      alert("Failed to delete shader pack: " + error);
    }
    setDeleteDialog(null);
  };

  const openFolder = async () => {
    try {
      await invoke("open_shaderpacks_folder", { instanceId });
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
          { name: "Shader Packs", extensions: ["zip"] },
        ],
      });
      
      if (selected) {
        const files = Array.isArray(selected) ? selected : [selected];
        for (const file of files) {
          await invoke("add_local_shader_pack", { instanceId, filePath: file });
        }
        await loadShaderPacks();
      }
    } catch (error) {
      console.error("Failed to add shader pack:", error);
      alert("Failed to add shader pack: " + error);
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
      alert('Only .zip files are supported for shader packs');
      return;
    }
    
    for (const filePath of zipFiles) {
      try {
        await invoke("add_local_shader_pack", { instanceId, filePath });
      } catch (error) {
        console.error("Failed to add shader pack:", error);
        alert(`Failed to add shader pack: ${error}`);
      }
    }
    await loadShaderPacks();
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
        
        await invoke("add_local_shader_pack_from_bytes", {
          instanceId,
          filename: file.name,
          data: base64,
        });
      } catch (error) {
        console.error("Failed to add shader pack:", error);
        alert(`Failed to add ${file.name}: ${error}`);
      }
    }
    
    await loadShaderPacks();
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
              <CardTitle>Shader Packs</CardTitle>
              <CardDescription>Manage shader packs for this instance</CardDescription>
            </div>
            <div className="flex flex-wrap gap-2">
              <Button 
                variant="default" 
                size="sm" 
                onClick={() => setShowDownloadDialog(true)}
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
              <Button variant="outline" size="sm" onClick={loadShaderPacks} disabled={loading}>
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
              <p className="text-lg font-medium text-primary">Drop shader packs here</p>
              <p className="text-sm text-muted-foreground">.zip files only</p>
            </div>
          ) : loading ? (
          <div className="flex items-center justify-center py-8">
            <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
            <span className="ml-2 text-muted-foreground">Loading shader packs...</span>
          </div>
        ) : shaderPacks.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <Package className="h-12 w-12 mx-auto mb-4 opacity-50" />
              <p>No shader packs found.</p>
              <p className="text-sm mt-2">Install a shader mod (like Iris or OptiFine), then download from Modrinth/CurseForge or add files manually.</p>
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
                {shaderPacks.map((pack) => (
                  <TableRow key={pack.filename}>
                    <TableCell className="font-medium">
                      <div className="flex items-center gap-2">
                        <Package className="h-4 w-4 text-muted-foreground" />
                        <span>{pack.name}</span>
                      </div>
                    </TableCell>
                    <TableCell>{pack.size}</TableCell>
                    <TableCell className="text-right">
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setDeleteDialog(pack.filename)}
                        title="Delete shader pack"
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
            <AlertDialogTitle>Delete Shader Pack</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete "{deleteDialog}"? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => deleteDialog && deleteShaderPack(deleteDialog)}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {instance && (
        <ResourceDownloadDialog
          open={showDownloadDialog}
          onOpenChange={setShowDownloadDialog}
          instanceId={instanceId}
          instanceName={instance.name}
          minecraftVersion={instance.minecraft_version}
          resourceType="shaderpack"
          onResourcesInstalled={loadShaderPacks}
        />
      )}
    </>
  );
}
