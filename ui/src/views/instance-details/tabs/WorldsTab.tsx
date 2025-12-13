import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Copy, Trash2, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
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
import type { WorldInfo } from "../types";

interface WorldsTabProps {
  instanceId: string;
}

export function WorldsTab({ instanceId }: WorldsTabProps) {
  const [worlds, setWorlds] = useState<WorldInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [deleteDialog, setDeleteDialog] = useState<string | null>(null);
  const [copyingWorld, setCopyingWorld] = useState<string | null>(null);

  useEffect(() => {
    loadWorlds();
  }, [instanceId]);

  const loadWorlds = async () => {
    setLoading(true);
    try {
      const worldList = await invoke<WorldInfo[]>("list_worlds", { instanceId });
      setWorlds(worldList);
    } catch (error) {
      console.error("Failed to load worlds:", error);
    } finally {
      setLoading(false);
    }
  };

  const deleteWorld = async (folderName: string) => {
    try {
      await invoke("delete_world", { instanceId, folderName });
      await loadWorlds();
    } catch (error) {
      console.error("Failed to delete world:", error);
      alert("Failed to delete world: " + error);
    }
    setDeleteDialog(null);
  };

  const copyWorld = async (folderName: string) => {
    const newName = prompt("Enter name for the copied world:", `${folderName}_copy`);
    if (!newName) return;
    
    setCopyingWorld(folderName);
    try {
      await invoke("copy_world", { instanceId, folderName, newName });
      await loadWorlds();
    } catch (error) {
      console.error("Failed to copy world:", error);
      alert("Failed to copy world: " + error);
    } finally {
      setCopyingWorld(null);
    }
  };

  return (
    <Card className="h-full">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Worlds</CardTitle>
            <CardDescription>Manage your saved worlds</CardDescription>
          </div>
          <Button variant="outline" size="sm" onClick={loadWorlds} disabled={loading}>
            <RefreshCw className={cn("h-4 w-4 mr-2", loading && "animate-spin")} />
            Refresh
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="flex items-center justify-center py-8">
            <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
            <span className="ml-2 text-muted-foreground">Loading worlds...</span>
          </div>
        ) : worlds.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            No worlds found. Create a world in-game to see it here.
          </div>
        ) : (
          <ScrollArea className="h-[400px]">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Game Mode</TableHead>
                  <TableHead>Last Played</TableHead>
                  <TableHead>Size</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {worlds.map((world) => (
                  <TableRow key={world.folder_name}>
                    <TableCell className="font-medium">
                      <div className="flex items-center gap-2">
                        <span>{world.name}</span>
                        {world.hardcore && (
                          <Badge variant="destructive" className="text-xs">Hardcore</Badge>
                        )}
                      </div>
                    </TableCell>
                    <TableCell>{world.game_type}</TableCell>
                    <TableCell>{world.last_played || "Unknown"}</TableCell>
                    <TableCell>{world.size}</TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => copyWorld(world.folder_name)}
                          disabled={copyingWorld === world.folder_name}
                          title="Copy world"
                        >
                          {copyingWorld === world.folder_name ? (
                            <RefreshCw className="h-4 w-4 animate-spin" />
                          ) : (
                            <Copy className="h-4 w-4" />
                          )}
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => setDeleteDialog(world.folder_name)}
                          title="Delete world"
                        >
                          <Trash2 className="h-4 w-4 text-destructive" />
                        </Button>
                      </div>
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
            <AlertDialogTitle>Delete World</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete the world "{worlds.find(w => w.folder_name === deleteDialog)?.name || deleteDialog}"? 
              This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => deleteDialog && deleteWorld(deleteDialog)}
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
