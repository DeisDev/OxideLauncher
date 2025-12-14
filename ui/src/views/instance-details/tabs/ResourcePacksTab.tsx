import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Trash2, RefreshCw, Package, FolderOpen } from "lucide-react";
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
import type { ResourcePackInfo } from "../types";

interface ResourcePacksTabProps {
  instanceId: string;
}

export function ResourcePacksTab({ instanceId }: ResourcePacksTabProps) {
  const [resourcePacks, setResourcePacks] = useState<ResourcePackInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [deleteDialog, setDeleteDialog] = useState<string | null>(null);

  useEffect(() => {
    loadResourcePacks();
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

  return (
    <Card className="h-full">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Resource Packs</CardTitle>
            <CardDescription>Manage resource packs for this instance</CardDescription>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={openFolder}>
              <FolderOpen className="h-4 w-4 mr-2" />
              Open Folder
            </Button>
            <Button variant="outline" size="sm" onClick={loadResourcePacks} disabled={loading}>
              <RefreshCw className={cn("h-4 w-4 mr-2", loading && "animate-spin")} />
              Refresh
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="flex items-center justify-center py-8">
            <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
            <span className="ml-2 text-muted-foreground">Loading resource packs...</span>
          </div>
        ) : resourcePacks.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            No resource packs found. Add resource packs to the resourcepacks folder.
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
    </Card>
  );
}
