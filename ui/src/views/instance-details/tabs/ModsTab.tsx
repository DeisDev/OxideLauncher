import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
  Search,
  Download,
  Plus,
  Check,
  XCircle,
  FolderOpen,
  FileText,
  RefreshCw,
  Package,
  Trash2,
  Globe,
  Bug,
  Code,
  Link as LinkIcon,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Switch } from "@/components/ui/switch";
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
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
} from "@/components/ui/context-menu";
import { cn } from "@/lib/utils";
import type { InstanceInfo, InstalledMod } from "../types";
import { formatFileSize } from "../utils";
import { ModDownloadDialog } from "@/components/dialogs/ModDownloadDialog";

interface ModsTabProps {
  instanceId: string;
  instance: InstanceInfo;
}

export function ModsTab({ instanceId, instance }: ModsTabProps) {
  const [installedMods, setInstalledMods] = useState<InstalledMod[]>([]);
  const [filteredMods, setFilteredMods] = useState<InstalledMod[]>([]);
  const [modFilter, setModFilter] = useState("");
  const [selectedMods, setSelectedMods] = useState<Set<string>>(new Set());
  const [deleteModDialog, setDeleteModDialog] = useState<string | null>(null);
  const [showModDownloadDialog, setShowModDownloadDialog] = useState(false);
  const [isDragging, setIsDragging] = useState(false);

  useEffect(() => {
    loadInstalledMods();
  }, [instanceId]);

  useEffect(() => {
    if (modFilter) {
      setFilteredMods(installedMods.filter(m => 
        m.name.toLowerCase().includes(modFilter.toLowerCase()) ||
        m.filename.toLowerCase().includes(modFilter.toLowerCase())
      ));
    } else {
      setFilteredMods(installedMods);
    }
  }, [modFilter, installedMods]);

  const loadInstalledMods = async () => {
    try {
      const mods = await invoke<InstalledMod[]>("get_installed_mods", {
        instanceId,
      });
      setInstalledMods(mods);
    } catch (error) {
      console.error("Failed to load installed mods:", error);
    }
  };

  const toggleMod = async (filename: string, currentEnabled: boolean) => {
    try {
      await invoke("toggle_mod", {
        instanceId,
        filename: filename,
        enabled: !currentEnabled,
      });
      await loadInstalledMods();
    } catch (error) {
      console.error("Failed to toggle mod:", error);
      alert("Failed to toggle mod: " + error);
    }
  };

  const deleteMod = async (filename: string) => {
    try {
      await invoke("delete_mod", {
        instanceId,
        filename: filename,
      });
      await loadInstalledMods();
      setSelectedMods(prev => {
        const next = new Set(prev);
        next.delete(filename);
        return next;
      });
    } catch (error) {
      console.error("Failed to delete mod:", error);
      alert("Failed to delete mod: " + error);
    } finally {
      setDeleteModDialog(null);
    }
  };

  const deleteSelectedMods = async () => {
    if (selectedMods.size === 0) return;
    try {
      await invoke("delete_mods", {
        instanceId,
        filenames: Array.from(selectedMods),
      });
      setSelectedMods(new Set());
      await loadInstalledMods();
    } catch (error) {
      console.error("Failed to delete mods:", error);
      alert("Failed to delete mods: " + error);
    }
  };

  const enableSelectedMods = async () => {
    if (selectedMods.size === 0) return;
    try {
      await invoke("enable_mods", {
        instanceId,
        filenames: Array.from(selectedMods),
      });
      await loadInstalledMods();
    } catch (error) {
      console.error("Failed to enable mods:", error);
      alert("Failed to enable mods: " + error);
    }
  };

  const disableSelectedMods = async () => {
    if (selectedMods.size === 0) return;
    try {
      await invoke("disable_mods", {
        instanceId,
        filenames: Array.from(selectedMods),
      });
      await loadInstalledMods();
    } catch (error) {
      console.error("Failed to disable mods:", error);
      alert("Failed to disable mods: " + error);
    }
  };

  const openModsFolder = async () => {
    try {
      await invoke("open_mods_folder", { instanceId });
    } catch (error) {
      console.error("Failed to open mods folder:", error);
      alert("Failed to open folder: " + error);
    }
  };

  const openConfigsFolder = async () => {
    try {
      await invoke("open_configs_folder", { instanceId });
    } catch (error) {
      console.error("Failed to open configs folder:", error);
      alert("Failed to open folder: " + error);
    }
  };

  const handleFileSelect = async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: "Jar Files", extensions: ["jar"] }],
      });
      
      if (selected && Array.isArray(selected)) {
        await processFilePaths(selected);
      } else if (selected) {
        await processFilePaths([selected]);
      }
    } catch (error) {
      console.error("Failed to select files:", error);
    }
  };

  const processFilePaths = async (filePaths: string[]) => {
    for (const filePath of filePaths) {
      if (!filePath.endsWith('.jar')) {
        alert(`Skipping ${filePath}: Only .jar files are supported`);
        continue;
      }
      try {
        await invoke("add_local_mod", {
          instanceId,
          filePath,
        });
      } catch (error) {
        console.error("Failed to add mod:", error);
        alert(`Failed to add mod: ${error}`);
      }
    }
    await loadInstalledMods();
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

  const processFiles = async (files: File[]) => {
    // For drag and drop, we need to read the file and save it
    for (const file of files) {
      if (!file.name.endsWith('.jar')) {
        alert(`Skipping ${file.name}: Only .jar files are supported`);
        continue;
      }
      try {
        // Convert File to ArrayBuffer, then save via Tauri
        const arrayBuffer = await file.arrayBuffer();
        const base64 = arrayBufferToBase64(arrayBuffer);
        
        await invoke("add_local_mod_from_bytes", {
          instanceId,
          filename: file.name,
          data: base64,
        });
      } catch (error) {
        console.error("Failed to add mod:", error);
        alert(`Failed to add ${file.name}: ${error}`);
      }
    }
    await loadInstalledMods();
  };

  // Process file paths from Tauri's drag-drop event
  const processDroppedPaths = useCallback(async (paths: string[]) => {
    const jarFiles = paths.filter(p => p.toLowerCase().endsWith('.jar'));
    if (jarFiles.length === 0) {
      alert('Only .jar files are supported for mods');
      return;
    }
    
    for (const filePath of jarFiles) {
      try {
        await invoke("add_local_mod", {
          instanceId,
          filePath,
        });
      } catch (error) {
        console.error("Failed to add mod:", error);
        alert(`Failed to add mod: ${error}`);
      }
    }
    await loadInstalledMods();
  }, [instanceId]);

  // Set up Tauri drag-drop event listener
  useEffect(() => {
    const webview = getCurrentWebviewWindow();
    let unlisten: (() => void) | undefined;

    webview.onDragDropEvent((event) => {
      if (event.payload.type === 'over') {
        setIsDragging(true);
      } else if (event.payload.type === 'drop') {
        setIsDragging(false);
        if (event.payload.paths && event.payload.paths.length > 0) {
          processDroppedPaths(event.payload.paths);
        }
      } else if (event.payload.type === 'leave' || event.payload.type === 'cancel') {
        setIsDragging(false);
      }
    }).then(fn => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [processDroppedPaths]);

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const files = Array.from(e.dataTransfer.files);
    if (files.length === 0) return;

    await processFiles(files);
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
    const displayedMods = modFilter 
      ? installedMods.filter(m => m.name.toLowerCase().includes(modFilter.toLowerCase()))
      : installedMods;
    
    if (selectedMods.size === displayedMods.length) {
      setSelectedMods(new Set());
    } else {
      setSelectedMods(new Set(displayedMods.map(m => m.filename)));
    }
  };

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Action Toolbar */}
      <div className="flex items-center gap-2 pb-4 border-b flex-wrap">
        <Button 
          variant="default" 
          size="sm" 
          onClick={() => setShowModDownloadDialog(true)}
        >
          <Download className="mr-2 h-4 w-4" />
          Download Mods
        </Button>
        <Button 
          variant="outline" 
          size="sm" 
          onClick={handleFileSelect}
        >
          <Plus className="mr-2 h-4 w-4" />
          Add File
        </Button>
        
        <div className="h-6 w-px bg-border mx-1" />
        
        <Button 
          variant="outline" 
          size="sm" 
          onClick={deleteSelectedMods}
          disabled={selectedMods.size === 0}
        >
          <Trash2 className="mr-2 h-4 w-4" />
          Remove
        </Button>
        <Button 
          variant="outline" 
          size="sm" 
          onClick={enableSelectedMods}
          disabled={selectedMods.size === 0}
        >
          <Check className="mr-2 h-4 w-4" />
          Enable
        </Button>
        <Button 
          variant="outline" 
          size="sm" 
          onClick={disableSelectedMods}
          disabled={selectedMods.size === 0}
        >
          <XCircle className="mr-2 h-4 w-4" />
          Disable
        </Button>
        
        <div className="flex-1" />
        
        <Button 
          variant="ghost" 
          size="sm" 
          onClick={loadInstalledMods}
        >
          <RefreshCw className="h-4 w-4" />
        </Button>
      </div>

      {/* Filter */}
      <div className="py-3">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Filter installed mods..."
            value={modFilter}
            onChange={(e) => setModFilter(e.target.value)}
            className="pl-9"
          />
        </div>
      </div>

      {/* Mods Table */}
      <div 
        className={cn(
          "flex-1 overflow-hidden border rounded-md transition-colors",
          isDragging && "border-primary border-2 bg-primary/5"
        )}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        {isDragging && (
          <div className="absolute inset-0 flex items-center justify-center bg-background/80 z-10 pointer-events-none">
            <div className="text-center">
              <Package className="h-12 w-12 mx-auto mb-2 text-primary" />
              <p className="text-lg font-semibold">Drop mod files here</p>
              <p className="text-sm text-muted-foreground">Only .jar files will be accepted</p>
            </div>
          </div>
        )}
        <ScrollArea className="h-full">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-10">
                  <Checkbox
                    checked={selectedMods.size === filteredMods.length && filteredMods.length > 0}
                    onCheckedChange={toggleAllMods}
                  />
                </TableHead>
                <TableHead className="w-10">On</TableHead>
                <TableHead className="w-10"></TableHead>
                <TableHead>Name</TableHead>
                <TableHead className="w-32">Version</TableHead>
                <TableHead className="w-36">Last Modified</TableHead>
                <TableHead className="w-24">Provider</TableHead>
                <TableHead className="w-24">Links</TableHead>
                <TableHead className="w-20 text-right">Size</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredMods.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={9} className="text-center py-8 text-muted-foreground">
                    {installedMods.length === 0 
                      ? "No mods installed. Click 'Download Mods' or drag & drop .jar files here."
                      : "No mods match your filter."
                    }
                  </TableCell>
                </TableRow>
              ) : (
                filteredMods.map((mod) => (
                  <ContextMenu key={mod.filename}>
                    <ContextMenuTrigger asChild>
                      <TableRow 
                        className={cn(selectedMods.has(mod.filename) && "bg-muted/50")}
                      >
                        <TableCell>
                          <Checkbox
                            checked={selectedMods.has(mod.filename)}
                            onCheckedChange={() => toggleModSelection(mod.filename)}
                          />
                        </TableCell>
                        <TableCell>
                          <Switch
                            checked={mod.enabled}
                            onCheckedChange={() => toggleMod(mod.filename, mod.enabled)}
                          />
                        </TableCell>
                        <TableCell>
                          {mod.icon_url ? (
                            <img
                              src={mod.icon_url}
                              alt=""
                              className="w-8 h-8 rounded object-cover"
                            />
                          ) : (
                            <div className="w-8 h-8 rounded bg-muted flex items-center justify-center">
                              <Package className="h-4 w-4 text-muted-foreground" />
                            </div>
                          )}
                        </TableCell>
                        <TableCell>
                          <div className={cn(!mod.enabled && "text-muted-foreground")}>
                            <p className="font-medium">{mod.name}</p>
                            <p className="text-xs text-muted-foreground truncate max-w-xs">
                              {mod.filename}
                            </p>
                          </div>
                        </TableCell>
                        <TableCell className={cn(!mod.enabled && "text-muted-foreground")}>
                          {mod.version || "-"}
                        </TableCell>
                        <TableCell className="text-muted-foreground text-sm">
                          {mod.modified || "-"}
                        </TableCell>
                        <TableCell>
                          {mod.provider && (
                            <Badge variant="secondary" className="text-xs">
                              {mod.provider}
                            </Badge>
                          )}
                        </TableCell>
                        <TableCell>
                          <div className="flex items-center gap-1">
                            {mod.homepage && (
                              <Button
                                variant="ghost"
                                size="icon"
                                className="h-6 w-6"
                                title="Homepage"
                                onClick={() => window.open(mod.homepage!, "_blank")}
                              >
                                <Globe className="h-3.5 w-3.5" />
                              </Button>
                            )}
                            {mod.issues_url && (
                              <Button
                                variant="ghost"
                                size="icon"
                                className="h-6 w-6"
                                title="Issue Tracker"
                                onClick={() => window.open(mod.issues_url!, "_blank")}
                              >
                                <Bug className="h-3.5 w-3.5" />
                              </Button>
                            )}
                            {mod.source_url && (
                              <Button
                                variant="ghost"
                                size="icon"
                                className="h-6 w-6"
                                title="Source Code"
                                onClick={() => window.open(mod.source_url!, "_blank")}
                              >
                                <Code className="h-3.5 w-3.5" />
                              </Button>
                            )}
                          </div>
                        </TableCell>
                        <TableCell className="text-right text-muted-foreground text-sm">
                          {formatFileSize(mod.size)}
                        </TableCell>
                      </TableRow>
                    </ContextMenuTrigger>
                    <ContextMenuContent>
                      <ContextMenuItem onClick={() => toggleMod(mod.filename, mod.enabled)}>
                        {mod.enabled ? <XCircle className="mr-2 h-4 w-4" /> : <Check className="mr-2 h-4 w-4" />}
                        {mod.enabled ? "Disable" : "Enable"}
                      </ContextMenuItem>
                      <ContextMenuItem 
                        onClick={() => setDeleteModDialog(mod.filename)}
                        className="text-destructive focus:text-destructive"
                      >
                        <Trash2 className="mr-2 h-4 w-4" />
                        Delete
                      </ContextMenuItem>
                      {(mod.homepage || mod.issues_url || mod.source_url) && (
                        <>
                          <ContextMenuSeparator />
                          <ContextMenuSub>
                            <ContextMenuSubTrigger>
                              <LinkIcon className="mr-2 h-4 w-4" />
                              Open URL
                            </ContextMenuSubTrigger>
                            <ContextMenuSubContent>
                              {mod.homepage && (
                                <ContextMenuItem onClick={() => window.open(mod.homepage!, "_blank")}>
                                  <Globe className="mr-2 h-4 w-4" />
                                  Homepage
                                </ContextMenuItem>
                              )}
                              {mod.issues_url && (
                                <ContextMenuItem onClick={() => window.open(mod.issues_url!, "_blank")}>
                                  <Bug className="mr-2 h-4 w-4" />
                                  Issue Tracker
                                </ContextMenuItem>
                              )}
                              {mod.source_url && (
                                <ContextMenuItem onClick={() => window.open(mod.source_url!, "_blank")}>
                                  <Code className="mr-2 h-4 w-4" />
                                  Source Code
                                </ContextMenuItem>
                              )}
                            </ContextMenuSubContent>
                          </ContextMenuSub>
                        </>
                      )}
                    </ContextMenuContent>
                  </ContextMenu>
                ))
              )}
            </TableBody>
          </Table>
        </ScrollArea>
      </div>

      {/* Footer Actions */}
      <div className="flex items-center gap-2 pt-4 border-t">
        <Button variant="outline" size="sm" onClick={openConfigsFolder}>
          <FileText className="mr-2 h-4 w-4" />
          View Configs
        </Button>
        <Button variant="outline" size="sm" onClick={openModsFolder}>
          <FolderOpen className="mr-2 h-4 w-4" />
          View Folder
        </Button>
        <div className="flex-1" />
        <span className="text-sm text-muted-foreground">
          {installedMods.length} mod{installedMods.length !== 1 ? 's' : ''} installed
          {selectedMods.size > 0 && ` • ${selectedMods.size} selected`}
        </span>
      </div>

      {/* Delete Mod Dialog */}
      <AlertDialog open={!!deleteModDialog} onOpenChange={() => setDeleteModDialog(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Mod</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete "{deleteModDialog}"? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => deleteModDialog && deleteMod(deleteModDialog)}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Mod Download Dialog */}
      <ModDownloadDialog
        open={showModDownloadDialog}
        onOpenChange={setShowModDownloadDialog}
        instanceId={instanceId}
        instanceName={instance.name}
        minecraftVersion={instance.minecraft_version}
        modLoader={instance.mod_loader}
        onModsInstalled={loadInstalledMods}
      />
    </div>
  );
}
