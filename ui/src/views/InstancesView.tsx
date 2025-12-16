import { useEffect, useState, useMemo } from "react";
import { Link, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import {
  Plus, Play, Trash2, Info, Pencil, Folder, Copy, FileOutput, FileInput,
  Image, Settings, Square, FolderTree, Link as LinkIcon, Feather,
  ArrowUpDown, Grid, List, Clock
} from "lucide-react";

// Mod loader icon paths (relative to public or using URL constructor)
const GrassIcon = new URL("../../art/grass.svg", import.meta.url).href;
const FabricIcon = new URL("../../art/fabricmc.svg", import.meta.url).href;
const ForgeIcon = new URL("../../art/forge.svg", import.meta.url).href;
const NeoForgeIcon = new URL("../../art/neoforged.svg", import.meta.url).href;
const QuiltIcon = new URL("../../art/quiltmc.svg", import.meta.url).href;
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
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
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useConfig } from "@/hooks/useConfig";
import { ExportInstanceDialog, ImportInstanceDialog } from "@/components/dialogs";

interface InstanceInfo {
  id: string;
  name: string;
  minecraft_version: string;
  mod_loader: string;
  mod_loader_version: string | null;
  icon: string | null;
  last_played: string | null;
  total_played_seconds: number;
  group?: string | null;
  date_created?: string;
}

// Helper to get mod loader icon
function getModLoaderIcon(loader: string): { icon: string | null; isLucide: boolean } {
  switch (loader) {
    case "Fabric":
      return { icon: FabricIcon, isLucide: false };
    case "Forge":
      return { icon: ForgeIcon, isLucide: false };
    case "NeoForge":
      return { icon: NeoForgeIcon, isLucide: false };
    case "Quilt":
      return { icon: QuiltIcon, isLucide: false };
    case "LiteLoader":
      return { icon: null, isLucide: true };
    case "Vanilla":
    default:
      return { icon: GrassIcon, isLucide: false };
  }
}

// Format play time in human readable format
function formatPlayTime(seconds: number): string {
  if (seconds === 0) return "Never played";
  
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  if (minutes > 0) {
    return `${minutes}m`;
  }
  return "< 1m";
}

// Format last played date
function formatLastPlayed(dateString: string | null): string {
  if (!dateString) return "Never";
  
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
  
  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "Yesterday";
  if (diffDays < 7) return `${diffDays} days ago`;
  if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
  if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`;
  return `${Math.floor(diffDays / 365)} years ago`;
}

export function InstancesView() {
  const navigate = useNavigate();
  const { config, updateUiConfig } = useConfig();
  const [instances, setInstances] = useState<InstanceInfo[]>([]);
  const [loading, setLoading] = useState(true);
  
  // Use config values directly
  const sortBy = config?.ui.instance_sort_by || "name";
  const sortAsc = config?.ui.instance_sort_asc ?? true;
  const gridSize = config?.ui.instance_grid_size || "medium";
  const viewMode = config?.ui.instance_view || "Grid";
  const showGameTime = config?.minecraft.show_game_time ?? true;
  
  // Dialog states
  const [renameDialogOpen, setRenameDialogOpen] = useState(false);
  const [groupDialogOpen, setGroupDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [exportDialogOpen, setExportDialogOpen] = useState(false);
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const [selectedInstance, setSelectedInstance] = useState<string | null>(null);
  const [selectedInstanceName, setSelectedInstanceName] = useState<string>("");
  const [renameName, setRenameName] = useState("");
  const [groupName, setGroupName] = useState("");

  useEffect(() => {
    loadInstances();
  }, []);

  const loadInstances = async () => {
    try {
      const data = await invoke<InstanceInfo[]>("get_instances");
      setInstances(data);
    } catch (error) {
      console.error("Failed to load instances:", error);
    } finally {
      setLoading(false);
    }
  };

  const launchInstance = async (id: string) => {
    try {
      await invoke("launch_instance", { instanceId: id });
      // Navigate to instance details with log tab if show_console is enabled
      if (config?.minecraft.show_console) {
        navigate(`/instance/${id}?tab=log`);
      }
    } catch (error) {
      console.error("Failed to launch instance:", error);
    }
  };

  const handleDelete = async () => {
    if (!selectedInstance) return;
    try {
      await invoke("delete_instance", { instanceId: selectedInstance });
      loadInstances();
    } catch (error) {
      console.error("Failed to delete instance:", error);
    } finally {
      setDeleteDialogOpen(false);
      setSelectedInstance(null);
    }
  };

  const handleRename = async () => {
    if (!selectedInstance || !renameName.trim()) return;
    try {
      await invoke("rename_instance", {
        instanceId: selectedInstance,
        newName: renameName,
      });
      loadInstances();
    } catch (error) {
      console.error("Failed to rename instance:", error);
    } finally {
      setRenameDialogOpen(false);
      setSelectedInstance(null);
      setRenameName("");
    }
  };

  const handleCopy = async (instanceId: string) => {
    try {
      await invoke("copy_instance", { instanceId });
      loadInstances();
    } catch (error) {
      console.error("Failed to copy instance:", error);
    }
  };

  const handleChangeGroup = async () => {
    if (!selectedInstance) return;
    try {
      await invoke("change_instance_group", {
        instanceId: selectedInstance,
        group: groupName || null,
      });
      loadInstances();
    } catch (error) {
      console.error("Failed to change group:", error);
    } finally {
      setGroupDialogOpen(false);
      setSelectedInstance(null);
      setGroupName("");
    }
  };

  const handleOpenFolder = async (instanceId: string) => {
    try {
      await invoke("open_instance_folder", { instanceId });
    } catch (error) {
      console.error("Failed to open folder:", error);
    }
  };

  const handleCreateShortcut = async (instanceId: string) => {
    try {
      await invoke("create_instance_shortcut", { instanceId });
    } catch (error) {
      console.error("Failed to create shortcut:", error);
    }
  };

  const handleKill = async (instanceId: string) => {
    try {
      await invoke("kill_instance", { instanceId });
    } catch (error) {
      console.error("Failed to kill instance:", error);
    }
  };

  const openRenameDialog = (instance: InstanceInfo) => {
    setSelectedInstance(instance.id);
    setRenameName(instance.name);
    setRenameDialogOpen(true);
  };

  const openGroupDialog = (instance: InstanceInfo) => {
    setSelectedInstance(instance.id);
    setGroupName(instance.group || "");
    setGroupDialogOpen(true);
  };

  const openDeleteDialog = (instanceId: string) => {
    setSelectedInstance(instanceId);
    setDeleteDialogOpen(true);
  };

  const openExportDialog = (instance: InstanceInfo) => {
    setSelectedInstance(instance.id);
    setSelectedInstanceName(instance.name);
    setExportDialogOpen(true);
  };

  // Grid size classes based on settings
  const gridClasses = useMemo(() => {
    switch (gridSize) {
      case "small":
        return "grid-cols-[repeat(auto-fill,minmax(160px,1fr))]";
      case "large":
        return "grid-cols-[repeat(auto-fill,minmax(280px,1fr))]";
      case "medium":
      default:
        return "grid-cols-[repeat(auto-fill,minmax(200px,1fr))]";
    }
  }, [gridSize]);

  // Sorted instances based on settings
  const sortedInstances = useMemo(() => {
    const sorted = [...instances].sort((a, b) => {
      let comparison = 0;
      
      switch (sortBy) {
        case "name":
          comparison = a.name.localeCompare(b.name);
          break;
        case "last_played":
          const aTime = a.last_played ? new Date(a.last_played).getTime() : 0;
          const bTime = b.last_played ? new Date(b.last_played).getTime() : 0;
          comparison = bTime - aTime; // More recent first by default
          break;
        case "date_created":
          const aCreated = a.date_created ? new Date(a.date_created).getTime() : 0;
          const bCreated = b.date_created ? new Date(b.date_created).getTime() : 0;
          comparison = bCreated - aCreated; // More recent first by default
          break;
        case "minecraft_version":
          comparison = a.minecraft_version.localeCompare(b.minecraft_version, undefined, { numeric: true });
          break;
        default:
          comparison = a.name.localeCompare(b.name);
      }
      
      return sortAsc ? comparison : -comparison;
    });
    
    return sorted;
  }, [instances, sortBy, sortAsc]);

  const getSortLabel = (sort: string) => {
    switch (sort) {
      case "name": return "Name";
      case "last_played": return "Last Played";
      case "date_created": return "Date Created";
      case "minecraft_version": return "Version";
      default: return "Name";
    }
  };

  const handleSortChange = (newSortBy: string) => {
    if (newSortBy === sortBy) {
      // Toggle direction if clicking same sort option
      updateUiConfig({ instance_sort_asc: !sortAsc });
    } else {
      updateUiConfig({ instance_sort_by: newSortBy, instance_sort_asc: true });
    }
  };

  const handleViewModeChange = (mode: "Grid" | "List") => {
    updateUiConfig({ instance_view: mode });
  };

  const handleGridSizeChange = (size: string) => {
    updateUiConfig({ instance_grid_size: size });
  };

  // Skeleton loading component that maintains layout consistency
  const LoadingSkeleton = () => (
    <div className="w-full">
      <div className="flex flex-wrap justify-between items-center gap-4 mb-8 pb-5 border-b border-border">
        <div className="skeleton h-9 w-64" />
        <div className="skeleton h-10 w-40" />
      </div>
      <div className="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-4">
        {[1, 2, 3, 4, 5, 6].map((i) => (
          <Card key={i} className="overflow-hidden">
            <div className="skeleton h-28" />
            <CardContent className="p-3 space-y-2">
              <div className="skeleton h-5 w-3/4" />
              <div className="skeleton h-4 w-1/2" />
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );

  // Instance card component for grid view
  const InstanceCard = ({ instance }: { instance: InstanceInfo }) => {
    const loaderInfo = getModLoaderIcon(instance.mod_loader);
    return (
      <ContextMenu>
        <ContextMenuTrigger>
          <Card className="overflow-hidden cursor-pointer transition-all hover:-translate-y-1 hover:shadow-xl hover:border-primary/50 h-full flex flex-col">
            {/* Fixed height image container */}
            <div className="h-28 flex items-center justify-center bg-gradient-to-br from-muted to-card overflow-hidden flex-shrink-0">
              {instance.icon && instance.icon !== "default" ? (
                <img
                  src={instance.icon}
                  alt={instance.name}
                  className="w-full h-full object-cover transition-transform hover:scale-105"
                />
              ) : (
                <div className="w-full h-full flex items-center justify-center bg-gradient-to-br from-primary/10 to-primary/5">
                  <img
                    src={GrassIcon}
                    alt="Minecraft"
                    className="h-14 w-14 object-contain opacity-80"
                  />
                </div>
              )}
            </div>
            {/* Fixed height content container */}
            <CardContent className="p-3 flex flex-col flex-1">
              <h3 className="font-semibold text-sm mb-1 truncate" title={instance.name}>
                {instance.name}
              </h3>
              <p className="text-xs text-muted-foreground">
                {instance.minecraft_version}
              </p>
              <div className="flex items-center gap-1 text-xs text-muted-foreground">
                {loaderInfo.isLucide ? (
                  <Feather className="h-3 w-3 flex-shrink-0" />
                ) : loaderInfo.icon && instance.mod_loader !== "Vanilla" ? (
                  <img src={loaderInfo.icon} alt={instance.mod_loader} className="h-3 w-3 flex-shrink-0" />
                ) : null}
                <span className="truncate">
                  {instance.mod_loader !== "Vanilla" 
                    ? `${instance.mod_loader}${instance.mod_loader_version ? ` ${instance.mod_loader_version}` : ""}`
                    : "Vanilla"}
                </span>
              </div>
              {/* Play time */}
              {showGameTime && (
                <div className="flex items-center gap-1 text-xs text-muted-foreground mt-1">
                  <Clock className="h-3 w-3 flex-shrink-0" />
                  <span>{formatPlayTime(instance.total_played_seconds)}</span>
                </div>
              )}
              {/* Buttons always at bottom */}
              <div className="flex gap-1 mt-auto pt-2">
                <Button
                  size="sm"
                  className="h-7 text-xs px-2 flex-1"
                  onClick={(e) => {
                    e.stopPropagation();
                    launchInstance(instance.id);
                  }}
                >
                  <Play className="mr-1 h-3 w-3" /> Play
                </Button>
                <Link to={`/instance/${instance.id}`} onClick={(e) => e.stopPropagation()}>
                  <Button size="sm" variant="secondary" className="h-7 px-2">
                    <Info className="h-3 w-3" />
                  </Button>
                </Link>
              </div>
            </CardContent>
          </Card>
        </ContextMenuTrigger>
        <InstanceContextMenu instance={instance} />
      </ContextMenu>
    );
  };

  // Instance row component for list view
  const InstanceRow = ({ instance }: { instance: InstanceInfo }) => {
    const loaderInfo = getModLoaderIcon(instance.mod_loader);
    return (
      <ContextMenu>
        <ContextMenuTrigger>
          <div className="flex items-center gap-4 p-3 rounded-lg border bg-card hover:bg-accent/50 transition-colors cursor-pointer">
            {/* Icon */}
            <div className="h-12 w-12 flex-shrink-0 rounded-lg overflow-hidden bg-gradient-to-br from-muted to-card flex items-center justify-center">
              {instance.icon && instance.icon !== "default" ? (
                <img
                  src={instance.icon}
                  alt={instance.name}
                  className="w-full h-full object-cover"
                />
              ) : (
                <img
                  src={GrassIcon}
                  alt="Minecraft"
                  className="h-8 w-8 object-contain opacity-80"
                />
              )}
            </div>
            
            {/* Info */}
            <div className="flex-1 min-w-0">
              <h3 className="font-semibold text-sm truncate">{instance.name}</h3>
              <div className="flex items-center gap-3 text-xs text-muted-foreground">
                <span>{instance.minecraft_version}</span>
                <span className="flex items-center gap-1">
                  {loaderInfo.isLucide ? (
                    <Feather className="h-3 w-3" />
                  ) : loaderInfo.icon && instance.mod_loader !== "Vanilla" ? (
                    <img src={loaderInfo.icon} alt={instance.mod_loader} className="h-3 w-3" />
                  ) : null}
                  {instance.mod_loader !== "Vanilla" ? instance.mod_loader : "Vanilla"}
                </span>
                {showGameTime && (
                  <span className="flex items-center gap-1">
                    <Clock className="h-3 w-3" />
                    {formatPlayTime(instance.total_played_seconds)}
                  </span>
                )}
                <span>Last played: {formatLastPlayed(instance.last_played)}</span>
              </div>
            </div>
            
            {/* Actions */}
            <div className="flex gap-2 flex-shrink-0">
              <Button
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  launchInstance(instance.id);
                }}
              >
                <Play className="mr-1 h-4 w-4" /> Play
              </Button>
              <Link to={`/instance/${instance.id}`} onClick={(e) => e.stopPropagation()}>
                <Button size="sm" variant="outline">
                  <Info className="h-4 w-4" />
                </Button>
              </Link>
            </div>
          </div>
        </ContextMenuTrigger>
        <InstanceContextMenu instance={instance} />
      </ContextMenu>
    );
  };

  // Shared context menu for instances
  const InstanceContextMenu = ({ instance }: { instance: InstanceInfo }) => (
    <ContextMenuContent className="w-56">
      <ContextMenuItem onClick={() => launchInstance(instance.id)}>
        <Play className="mr-2 h-4 w-4" />
        Launch
      </ContextMenuItem>
      <ContextMenuItem onClick={() => handleKill(instance.id)}>
        <Square className="mr-2 h-4 w-4" />
        Kill
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem onClick={() => openRenameDialog(instance)}>
        <Pencil className="mr-2 h-4 w-4" />
        Rename
      </ContextMenuItem>
      <ContextMenuItem>
        <Image className="mr-2 h-4 w-4" />
        Change Icon
      </ContextMenuItem>
      <ContextMenuItem onClick={() => navigate(`/instance/${instance.id}`)}>
        <Settings className="mr-2 h-4 w-4" />
        Edit...
      </ContextMenuItem>
      <ContextMenuItem onClick={() => openGroupDialog(instance)}>
        <FolderTree className="mr-2 h-4 w-4" />
        Change Group...
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem onClick={() => handleOpenFolder(instance.id)}>
        <Folder className="mr-2 h-4 w-4" />
        Folder
      </ContextMenuItem>
      <ContextMenuItem onClick={() => openExportDialog(instance)}>
        <FileOutput className="mr-2 h-4 w-4" />
        Export
      </ContextMenuItem>
      <ContextMenuItem onClick={() => handleCopy(instance.id)}>
        <Copy className="mr-2 h-4 w-4" />
        Copy
      </ContextMenuItem>
      <ContextMenuItem onClick={() => handleCreateShortcut(instance.id)}>
        <LinkIcon className="mr-2 h-4 w-4" />
        Create Shortcut
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem
        className="text-destructive focus:text-destructive"
        onClick={() => openDeleteDialog(instance.id)}
      >
        <Trash2 className="mr-2 h-4 w-4" />
        Delete
      </ContextMenuItem>
    </ContextMenuContent>
  );

  if (loading) {
    return <LoadingSkeleton />;
  }

  return (
    <div className="w-full h-full flex flex-col">
      <div className="flex flex-wrap justify-between items-center gap-4 mb-6 pb-5 border-b border-border">
        <h1 className="text-3xl font-bold bg-gradient-to-r from-foreground to-muted-foreground bg-clip-text text-transparent">
          Minecraft Instances
        </h1>
        <div className="flex items-center gap-2">
          {/* View Mode Toggle */}
          <div className="flex border rounded-lg overflow-hidden">
            <Button
              variant={viewMode === "Grid" ? "default" : "ghost"}
              size="sm"
              className="rounded-none h-9"
              onClick={() => handleViewModeChange("Grid")}
            >
              <Grid className="h-4 w-4" />
            </Button>
            <Button
              variant={viewMode === "List" ? "default" : "ghost"}
              size="sm"
              className="rounded-none h-9"
              onClick={() => handleViewModeChange("List")}
            >
              <List className="h-4 w-4" />
            </Button>
          </div>

          {/* Grid Size (only in grid view) */}
          {viewMode === "Grid" && (
            <Select value={gridSize} onValueChange={handleGridSizeChange}>
              <SelectTrigger className="w-28 h-9">
                <SelectValue placeholder="Size" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="small">Small</SelectItem>
                <SelectItem value="medium">Medium</SelectItem>
                <SelectItem value="large">Large</SelectItem>
              </SelectContent>
            </Select>
          )}

          {/* Sort Dropdown */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm" className="h-9">
                <ArrowUpDown className="mr-2 h-4 w-4" />
                {getSortLabel(sortBy)} {sortAsc ? "↑" : "↓"}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => handleSortChange("name")}>
                Name {sortBy === "name" && (sortAsc ? "↑" : "↓")}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleSortChange("last_played")}>
                Last Played {sortBy === "last_played" && (sortAsc ? "↑" : "↓")}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleSortChange("date_created")}>
                Date Created {sortBy === "date_created" && (sortAsc ? "↑" : "↓")}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleSortChange("minecraft_version")}>
                Minecraft Version {sortBy === "minecraft_version" && (sortAsc ? "↑" : "↓")}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          <Button 
            variant="outline" 
            className="h-9"
            onClick={() => setImportDialogOpen(true)}
          >
            <FileInput className="mr-2 h-4 w-4" /> Import
          </Button>

          <Link to="/create-instance">
            <Button className="h-9">
              <Plus className="mr-2 h-4 w-4" /> Create Instance
            </Button>
          </Link>
        </div>
      </div>

      {sortedInstances.length === 0 ? (
        <ContextMenu>
          <ContextMenuTrigger asChild>
            <div className="flex-1 flex flex-col items-center justify-center">
              <p className="mb-4">No instances found. Create your first instance to get started!</p>
              <Link to="/create-instance">
                <Button>
                  <Plus className="mr-2 h-4 w-4" /> Create Instance
                </Button>
              </Link>
            </div>
          </ContextMenuTrigger>
          <ContextMenuContent className="w-48">
            <ContextMenuItem onClick={() => navigate("/create-instance")}>
              <Plus className="mr-2 h-4 w-4" />
              Create Instance
            </ContextMenuItem>
            <ContextMenuItem onClick={() => setImportDialogOpen(true)}>
              <FileInput className="mr-2 h-4 w-4" />
              Import Instance
            </ContextMenuItem>
          </ContextMenuContent>
        </ContextMenu>
      ) : viewMode === "Grid" ? (
        <ContextMenu>
          <ContextMenuTrigger asChild>
            <div className={`flex-1 grid ${gridClasses} gap-4 content-start`}>
              {sortedInstances.map((instance) => (
                <InstanceCard key={instance.id} instance={instance} />
              ))}
            </div>
          </ContextMenuTrigger>
          <ContextMenuContent className="w-48">
            <ContextMenuItem onClick={() => navigate("/create-instance")}>
              <Plus className="mr-2 h-4 w-4" />
              Create Instance
            </ContextMenuItem>
            <ContextMenuItem onClick={() => setImportDialogOpen(true)}>
              <FileInput className="mr-2 h-4 w-4" />
              Import Instance
            </ContextMenuItem>
          </ContextMenuContent>
        </ContextMenu>
      ) : (
        <ContextMenu>
          <ContextMenuTrigger asChild>
            <div className="flex-1 space-y-2">
              {sortedInstances.map((instance) => (
                <InstanceRow key={instance.id} instance={instance} />
              ))}
            </div>
          </ContextMenuTrigger>
          <ContextMenuContent className="w-48">
            <ContextMenuItem onClick={() => navigate("/create-instance")}>
              <Plus className="mr-2 h-4 w-4" />
              Create Instance
            </ContextMenuItem>
            <ContextMenuItem onClick={() => setImportDialogOpen(true)}>
              <FileInput className="mr-2 h-4 w-4" />
              Import Instance
            </ContextMenuItem>
          </ContextMenuContent>
        </ContextMenu>
      )}

      {/* Rename Dialog */}
      <Dialog open={renameDialogOpen} onOpenChange={setRenameDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Rename Instance</DialogTitle>
            <DialogDescription>
              Enter a new name for this instance.
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Label htmlFor="name">Name</Label>
            <Input
              id="name"
              value={renameName}
              onChange={(e) => setRenameName(e.target.value)}
              placeholder="Enter new name"
              onKeyDown={(e) => {
                if (e.key === "Enter") handleRename();
              }}
            />
          </div>
          <DialogFooter>
            <Button variant="secondary" onClick={() => setRenameDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleRename}>Rename</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Group Dialog */}
      <Dialog open={groupDialogOpen} onOpenChange={setGroupDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Change Group</DialogTitle>
            <DialogDescription>
              Enter a group name or leave empty to ungroup.
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Label htmlFor="group">Group Name</Label>
            <Input
              id="group"
              value={groupName}
              onChange={(e) => setGroupName(e.target.value)}
              placeholder="Enter group name (or leave empty)"
              onKeyDown={(e) => {
                if (e.key === "Enter") handleChangeGroup();
              }}
            />
          </div>
          <DialogFooter>
            <Button variant="secondary" onClick={() => setGroupDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleChangeGroup}>Change Group</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Instance?</AlertDialogTitle>
            <AlertDialogDescription>
              This action cannot be undone. This will permanently delete the instance
              and all its data.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Export Instance Dialog */}
      {selectedInstance && (
        <ExportInstanceDialog
          open={exportDialogOpen}
          onOpenChange={setExportDialogOpen}
          instanceId={selectedInstance}
          instanceName={selectedInstanceName}
        />
      )}

      {/* Import Instance Dialog */}
      <ImportInstanceDialog
        open={importDialogOpen}
        onOpenChange={setImportDialogOpen}
        onImportComplete={loadInstances}
      />
    </div>
  );
}
