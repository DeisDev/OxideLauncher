// Dialog window header component with close button
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

import { X } from "lucide-react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { emit } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";

interface DialogWindowHeaderProps {
  title: string;
  icon?: React.ReactNode;
  children?: React.ReactNode;
}

/**
 * A draggable header component for dialog windows with a close button.
 * Use this at the top of all dialog pages that open in separate windows.
 */
export function DialogWindowHeader({ title, icon, children }: DialogWindowHeaderProps) {
  const handleClose = () => {
    // Fire-and-forget close operation to avoid blocking UI
    (async () => {
      try {
        // Emit dialog-closed event first so main window removes overlay
        await emit("dialog-closed", {});
      } catch {
        // Ignore emit errors
      }
      
      try {
        // Then close the window
        const currentWindow = getCurrentWebviewWindow();
        // Use destroy() for immediate close if close() doesn't work
        await currentWindow.close();
      } catch (error) {
        console.error("Failed to close window:", error);
        // Try destroy as fallback
        try {
          const currentWindow = getCurrentWebviewWindow();
          await currentWindow.destroy();
        } catch {
          // Ignore
        }
      }
    })();
  };

  return (
    <div 
      className="flex items-center justify-between px-4 py-3 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60"
      data-tauri-drag-region
    >
      <div className="flex items-center gap-2" data-tauri-drag-region>
        {icon}
        <h1 className="text-lg font-semibold" data-tauri-drag-region>{title}</h1>
      </div>
      
      <div className="flex items-center gap-2">
        {children}
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 rounded-sm hover:bg-destructive/10 hover:text-destructive"
          onClick={handleClose}
        >
          <X className="h-4 w-4" />
          <span className="sr-only">Close</span>
        </Button>
      </div>
    </div>
  );
}
