import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import {
  Plus, Play, Trash2, Info, Pencil, Folder, Copy, FileOutput,
  Image, Settings, Square, FolderTree, Link as LinkIcon, Feather
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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

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
      return { icon: null, isLucide: true }; // Use Lucide Feather icon
    case "Vanilla":
    default:
      return { icon: GrassIcon, isLucide: false };
  }
}

export function InstancesView() {
  const navigate = useNavigate();
  const [instances, setInstances] = useState<InstanceInfo[]>([]);
  const [loading, setLoading] = useState(true);
  
  // Dialog states
  const [renameDialogOpen, setRenameDialogOpen] = useState(false);
  const [groupDialogOpen, setGroupDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [selectedInstance, setSelectedInstance] = useState<string | null>(null);
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

  if (loading) {
    return <div className="loading">Loading instances...</div>;
  }

  return (
    <div className="max-w-[1600px] mx-auto">
      <div className="flex justify-between items-center mb-8 pb-5 border-b border-border">
        <h1 className="text-3xl font-bold bg-gradient-to-r from-foreground to-muted-foreground bg-clip-text text-transparent">
          Minecraft Instances
        </h1>
        <Link to="/create-instance">
          <Button>
            <Plus className="mr-2 h-4 w-4" /> Create Instance
          </Button>
        </Link>
      </div>

      {instances.length === 0 ? (
        <div className="empty-state">
          <p className="mb-4">No instances found. Create your first instance to get started!</p>
          <Link to="/create-instance">
            <Button>
              <Plus className="mr-2 h-4 w-4" /> Create Instance
            </Button>
          </Link>
        </div>
      ) : (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-4">
          {instances.map((instance) => {
            const loaderInfo = getModLoaderIcon(instance.mod_loader);
            return (
            <ContextMenu key={instance.id}>
              <ContextMenuTrigger>
                <Card className="overflow-hidden cursor-pointer transition-all hover:-translate-y-1 hover:shadow-xl hover:border-primary/50">
                  <div className="h-28 flex items-center justify-center bg-gradient-to-br from-muted to-card overflow-hidden">
                    {instance.icon ? (
                      <img
                        src={instance.icon}
                        alt={instance.name}
                        className="w-full h-full object-cover transition-transform hover:scale-105"
                      />
                    ) : (
                      <img
                        src={GrassIcon}
                        alt="Minecraft"
                        className="h-16 w-16 object-contain"
                      />
                    )}
                  </div>
                  <CardContent className="p-3">
                    <h3 className="font-semibold text-sm mb-1 truncate">{instance.name}</h3>
                    <p className="text-xs text-muted-foreground">
                      {instance.minecraft_version}
                    </p>
                    <div className="flex items-center gap-1 text-xs text-muted-foreground mb-2">
                      {loaderInfo.isLucide ? (
                        <Feather className="h-3 w-3" />
                      ) : loaderInfo.icon && instance.mod_loader !== "Vanilla" ? (
                        <img src={loaderInfo.icon} alt={instance.mod_loader} className="h-3 w-3" />
                      ) : null}
                      <span>
                        {instance.mod_loader !== "Vanilla" 
                          ? `${instance.mod_loader}${instance.mod_loader_version ? ` ${instance.mod_loader_version}` : ""}`
                          : "Vanilla"}
                      </span>
                    </div>
                    <div className="flex gap-1 mt-2">
                      <Button
                        size="sm"
                        className="h-7 text-xs px-2"
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
                <ContextMenuItem>
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
            </ContextMenu>
          );
          })}
        </div>
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
    </div>
  );
}
