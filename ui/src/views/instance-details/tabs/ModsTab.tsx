import { useRef, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Search,
  X,
  Download,
  Plus,
  Check,
  XCircle,
  FolderOpen,
  FileText,
  RefreshCw,
  Package,
  Trash2,
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
import { cn } from "@/lib/utils";
import type { InstanceInfo, ModSearchResult, InstalledMod } from "../types";
import { formatFileSize } from "../utils";

interface ModsTabProps {
  instanceId: string;
  instance: InstanceInfo;
}

export function ModsTab({ instanceId, instance }: ModsTabProps) {
  const [modSearchQuery, setModSearchQuery] = useState("");
  const [modSearchResults, setModSearchResults] = useState<ModSearchResult[]>([]);
  const [installedMods, setInstalledMods] = useState<InstalledMod[]>([]);
  const [filteredMods, setFilteredMods] = useState<InstalledMod[]>([]);
  const [modFilter, setModFilter] = useState("");
  const [selectedMods, setSelectedMods] = useState<Set<string>>(new Set());
  const [searchingMods, setSearchingMods] = useState(false);
  const [downloadingMod, setDownloadingMod] = useState<string | null>(null);
  const [deleteModDialog, setDeleteModDialog] = useState<string | null>(null);
  const [showModSearch, setShowModSearch] = useState(false);
  const [searchPlatform, setSearchPlatform] = useState<"modrinth" | "curseforge">("modrinth");
  const fileInputRef = useRef<HTMLInputElement>(null);

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

  const searchMods = async () => {
    if (!modSearchQuery.trim()) return;

    setSearchingMods(true);
    try {
      const results = await invoke<ModSearchResult[]>("search_mods", {
        query: modSearchQuery,
        minecraftVersion: instance.minecraft_version,
        modLoader: instance.mod_loader.toLowerCase(),
        platform: searchPlatform,
      });
      setModSearchResults(results);
    } catch (error) {
      console.error("Failed to search mods:", error);
      alert("Failed to search mods: " + error);
    } finally {
      setSearchingMods(false);
    }
  };

  const downloadMod = async (modId: string, platform: string) => {
    setDownloadingMod(modId);
    try {
      await invoke("download_mod", {
        instanceId,
        modId: modId,
        platform: platform.toLowerCase(),
      });
      await loadInstalledMods();
    } catch (error) {
      console.error("Failed to download mod:", error);
      alert("Failed to download mod: " + error);
    } finally {
      setDownloadingMod(null);
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

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files || files.length === 0) return;
    
    for (const file of Array.from(files)) {
      if (!file.name.endsWith('.jar')) {
        alert(`Skipping ${file.name}: Only .jar files are supported`);
        continue;
      }
      try {
        await invoke("add_local_mod", {
          instanceId,
          filePath: file.name,
        });
      } catch (error) {
        console.error("Failed to add mod:", error);
        alert(`Failed to add ${file.name}: ${error}`);
      }
    }
    await loadInstalledMods();
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
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
      {/* Hidden file input for adding local mods */}
      <input
        type="file"
        ref={fileInputRef}
        onChange={handleFileSelect}
        accept=".jar"
        multiple
        className="hidden"
      />

      {/* Action Toolbar */}
      <div className="flex items-center gap-2 pb-4 border-b flex-wrap">
        <Button 
          variant="default" 
          size="sm" 
          onClick={() => setShowModSearch(!showModSearch)}
        >
          <Download className="mr-2 h-4 w-4" />
          Download Mods
        </Button>
        <Button 
          variant="outline" 
          size="sm" 
          onClick={() => fileInputRef.current?.click()}
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

      {/* Mod Search Panel (collapsible) */}
      {showModSearch && (
        <div className="py-4 border-b">
          {/* Platform Selection */}
          <div className="flex items-center gap-2 mb-3">
            <span className="text-sm text-muted-foreground">Search on:</span>
            <div className="flex gap-1">
              <Button
                variant={searchPlatform === "modrinth" ? "default" : "outline"}
                size="sm"
                onClick={() => setSearchPlatform("modrinth")}
                className="h-7 px-3 text-xs"
              >
                Modrinth
              </Button>
              <Button
                variant={searchPlatform === "curseforge" ? "default" : "outline"}
                size="sm"
                onClick={() => setSearchPlatform("curseforge")}
                className="h-7 px-3 text-xs"
              >
                CurseForge
              </Button>
            </div>
          </div>
          
          <div className="flex items-center gap-4 mb-4">
            <Input
              placeholder={`Search mods on ${searchPlatform === "modrinth" ? "Modrinth" : "CurseForge"}...`}
              value={modSearchQuery}
              onChange={(e) => setModSearchQuery(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && searchMods()}
              className="flex-1"
            />
            <Button onClick={searchMods} disabled={searchingMods}>
              <Search className="mr-2 h-4 w-4" />
              {searchingMods ? "Searching..." : "Search"}
            </Button>
            <Button 
              variant="ghost" 
              size="sm" 
              onClick={() => {
                setShowModSearch(false);
                setModSearchResults([]);
              }}
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
          
          {modSearchResults.length > 0 && (
            <ScrollArea className="h-64">
              <div className="grid gap-2">
                {modSearchResults.map((mod) => (
                  <div 
                    key={mod.id} 
                    className="flex items-center gap-3 p-2 rounded-md border bg-card hover:bg-muted/50"
                  >
                    {mod.icon_url ? (
                      <img
                        src={mod.icon_url}
                        alt={mod.name}
                        className="w-10 h-10 rounded object-cover"
                      />
                    ) : (
                      <div className="w-10 h-10 rounded bg-muted flex items-center justify-center">
                        <Package className="h-5 w-5 text-muted-foreground" />
                      </div>
                    )}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <p className="font-medium truncate">{mod.name}</p>
                        <Badge variant="outline" className="text-[10px] px-1.5 py-0">
                          {mod.platform === "curseforge" ? "CF" : "MR"}
                        </Badge>
                      </div>
                      <p className="text-xs text-muted-foreground truncate">
                        by {mod.author} • {mod.downloads.toLocaleString()} downloads
                      </p>
                    </div>
                    <Button
                      onClick={() => downloadMod(mod.id, mod.platform)}
                      disabled={downloadingMod === mod.id}
                      size="sm"
                    >
                      {downloadingMod === mod.id ? (
                        <>
                          <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                          Installing...
                        </>
                      ) : (
                        <>
                          <Download className="mr-2 h-4 w-4" />
                          Install
                        </>
                      )}
                    </Button>
                  </div>
                ))}
              </div>
            </ScrollArea>
          )}
        </div>
      )}

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
      <div className="flex-1 overflow-hidden border rounded-md">
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
                <TableHead className="w-20 text-right">Size</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredMods.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={8} className="text-center py-8 text-muted-foreground">
                    {installedMods.length === 0 
                      ? "No mods installed. Click 'Download Mods' to get started."
                      : "No mods match your filter."
                    }
                  </TableCell>
                </TableRow>
              ) : (
                filteredMods.map((mod) => (
                  <TableRow 
                    key={mod.filename}
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
                    <TableCell className="text-right text-muted-foreground text-sm">
                      {formatFileSize(mod.size)}
                    </TableCell>
                  </TableRow>
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
    </div>
  );
}
