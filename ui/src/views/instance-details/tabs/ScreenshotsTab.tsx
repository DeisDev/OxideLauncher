// Screenshots tab component for viewing and managing instance screenshots
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

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Trash2, RefreshCw, FolderOpen } from "lucide-react";
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
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import type { ScreenshotInfo } from "../types";

interface ScreenshotsTabProps {
  instanceId: string;
}

export function ScreenshotsTab({ instanceId }: ScreenshotsTabProps) {
  const [screenshots, setScreenshots] = useState<ScreenshotInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [deleteDialog, setDeleteDialog] = useState<string | null>(null);
  // Reserved for fullscreen preview feature
  const [_selectedScreenshot, setSelectedScreenshot] = useState<string | null>(null);

  useEffect(() => {
    loadScreenshots();
  }, [instanceId]);

  const loadScreenshots = async () => {
    setLoading(true);
    try {
      const shots = await invoke<ScreenshotInfo[]>("list_screenshots", { instanceId });
      setScreenshots(shots);
    } catch (error) {
      console.error("Failed to load screenshots:", error);
    } finally {
      setLoading(false);
    }
  };

  const deleteScreenshot = async (filename: string) => {
    try {
      await invoke("delete_screenshot", { instanceId, filename });
      await loadScreenshots();
    } catch (error) {
      console.error("Failed to delete screenshot:", error);
      alert("Failed to delete screenshot: " + error);
    }
    setDeleteDialog(null);
  };

  const openScreenshotsFolder = async () => {
    try {
      await invoke("open_screenshots_folder", { instanceId });
    } catch (error) {
      console.error("Failed to open screenshots folder:", error);
      alert("Failed to open folder: " + error);
    }
  };

  return (
    <Card className="h-full">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Screenshots</CardTitle>
            <CardDescription>View and manage your screenshots</CardDescription>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={openScreenshotsFolder}>
              <FolderOpen className="h-4 w-4 mr-2" />
              Open Folder
            </Button>
            <Button variant="outline" size="sm" onClick={loadScreenshots} disabled={loading}>
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
            <span className="ml-2 text-muted-foreground">Loading screenshots...</span>
          </div>
        ) : screenshots.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            No screenshots found. Take screenshots in-game with F2.
          </div>
        ) : (
          <ScrollArea className="h-[400px]">
            <div className="grid grid-cols-3 gap-4">
              {screenshots.map((screenshot) => (
                <div
                  key={screenshot.filename}
                  className="group relative aspect-video bg-muted rounded-lg overflow-hidden cursor-pointer border border-border hover:border-primary transition-colors"
                  onClick={() => setSelectedScreenshot(screenshot.path)}
                >
                  <img
                    src={`file://${screenshot.path}`}
                    alt={screenshot.filename}
                    className="w-full h-full object-cover"
                    onError={(e) => {
                      e.currentTarget.style.display = 'none';
                    }}
                  />
                  <div className="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                    <Button
                      variant="ghost"
                      size="icon"
                      className="text-white hover:text-destructive"
                      onClick={(e) => {
                        e.stopPropagation();
                        setDeleteDialog(screenshot.filename);
                      }}
                    >
                      <Trash2 className="h-5 w-5" />
                    </Button>
                  </div>
                  <div className="absolute bottom-0 left-0 right-0 bg-black/70 px-2 py-1 text-xs text-white truncate">
                    {screenshot.timestamp || screenshot.filename}
                  </div>
                </div>
              ))}
            </div>
          </ScrollArea>
        )}
      </CardContent>

      <AlertDialog open={!!deleteDialog} onOpenChange={() => setDeleteDialog(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Screenshot</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete "{deleteDialog}"? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => deleteDialog && deleteScreenshot(deleteDialog)}
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
