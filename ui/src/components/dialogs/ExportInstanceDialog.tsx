// Export instance dialog with customizable content options
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

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import {
  Loader2,
  Package,
  Save,
  FolderOpen,
  Camera,
  FileText,
  AlertTriangle,
  Paintbrush,
  Settings,
  Gamepad,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Progress } from "@/components/ui/progress";
import { Separator } from "@/components/ui/separator";

interface ExportInstanceDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  instanceId: string;
  instanceName: string;
}

interface ExportOptions {
  include_saves: boolean;
  include_screenshots: boolean;
  include_logs: boolean;
  include_crash_reports: boolean;
  include_resource_packs: boolean;
  include_shader_packs: boolean;
  include_mods: boolean;
  include_configs: boolean;
  include_game_settings: boolean;
}

const defaultOptions: ExportOptions = {
  include_saves: true,
  include_screenshots: false,
  include_logs: false,
  include_crash_reports: false,
  include_resource_packs: true,
  include_shader_packs: true,
  include_mods: true,
  include_configs: true,
  include_game_settings: true,
};

export function ExportInstanceDialog({
  open,
  onOpenChange,
  instanceId,
  instanceName,
}: ExportInstanceDialogProps) {
  const [options, setOptions] = useState<ExportOptions>(defaultOptions);
  const [exporting, setExporting] = useState(false);
  const [progress, setProgress] = useState(0);
  const [status, setStatus] = useState("");
  const [error, setError] = useState<string | null>(null);

  const handleOptionChange = (key: keyof ExportOptions, value: boolean) => {
    setOptions(prev => ({ ...prev, [key]: value }));
  };

  const handleExport = async () => {
    try {
      setError(null);
      setExporting(true);
      setProgress(0);
      setStatus("Selecting destination...");

      // Open save dialog
      const filePath = await save({
        defaultPath: `${instanceName.replace(/[^a-zA-Z0-9-_]/g, "_")}.oxide`,
        filters: [
          { name: "OxideLauncher Instance", extensions: ["oxide"] },
          { name: "All Files", extensions: ["*"] },
        ],
      });

      if (!filePath) {
        setExporting(false);
        setStatus("");
        return;
      }

      setProgress(10);
      setStatus("Preparing export...");

      // Call the export command
      await invoke("export_instance_to_file", {
        instanceId,
        outputPath: filePath,
        options,
      });

      setProgress(100);
      setStatus("Export complete!");
      
      // Close dialog after a short delay
      setTimeout(() => {
        onOpenChange(false);
        setExporting(false);
        setProgress(0);
        setStatus("");
      }, 1500);
    } catch (err) {
      console.error("Export failed:", err);
      setError(String(err));
      setExporting(false);
      setProgress(0);
      setStatus("");
    }
  };

  const handleClose = () => {
    if (!exporting) {
      setError(null);
      setProgress(0);
      setStatus("");
      onOpenChange(false);
    }
  };

  const optionGroups = [
    {
      title: "Game Content",
      options: [
        { key: "include_mods" as const, label: "Mods", icon: Package, description: "All installed mods" },
        { key: "include_configs" as const, label: "Config Files", icon: Settings, description: "Mod configuration files" },
        { key: "include_game_settings" as const, label: "Game Settings", icon: Gamepad, description: "options.txt and other game settings" },
      ],
    },
    {
      title: "Resource Content",
      options: [
        { key: "include_resource_packs" as const, label: "Resource Packs", icon: Paintbrush, description: "Texture and resource packs" },
        { key: "include_shader_packs" as const, label: "Shader Packs", icon: Paintbrush, description: "Shader packs (Iris/Optifine)" },
      ],
    },
    {
      title: "User Data",
      options: [
        { key: "include_saves" as const, label: "World Saves", icon: Save, description: "All world saves" },
        { key: "include_screenshots" as const, label: "Screenshots", icon: Camera, description: "In-game screenshots" },
      ],
    },
    {
      title: "Logs & Reports",
      options: [
        { key: "include_logs" as const, label: "Logs", icon: FileText, description: "Game logs" },
        { key: "include_crash_reports" as const, label: "Crash Reports", icon: AlertTriangle, description: "Crash reports" },
      ],
    },
  ];

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FolderOpen className="h-5 w-5" />
            Export Instance
          </DialogTitle>
          <DialogDescription>
            Export "{instanceName}" to a portable .oxide file
          </DialogDescription>
        </DialogHeader>

        {exporting ? (
          <div className="py-6 space-y-4">
            <div className="flex items-center gap-3">
              <Loader2 className="h-5 w-5 animate-spin text-primary" />
              <span className="text-sm">{status}</span>
            </div>
            <Progress value={progress} className="h-2" />
          </div>
        ) : (
          <div className="py-2 space-y-4 max-h-[400px] overflow-y-auto">
            {error && (
              <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-sm">
                {error}
              </div>
            )}

            {optionGroups.map((group, groupIndex) => (
              <div key={group.title}>
                {groupIndex > 0 && <Separator className="my-3" />}
                <h4 className="text-sm font-medium mb-3 text-muted-foreground">{group.title}</h4>
                <div className="space-y-3">
                  {group.options.map((option) => (
                    <div key={option.key} className="flex items-start gap-3">
                      <Checkbox
                        id={option.key}
                        checked={options[option.key]}
                        onCheckedChange={(checked) => 
                          handleOptionChange(option.key, checked as boolean)
                        }
                      />
                      <div className="flex-1 space-y-1">
                        <Label
                          htmlFor={option.key}
                          className="flex items-center gap-2 cursor-pointer font-medium"
                        >
                          <option.icon className="h-4 w-4 text-muted-foreground" />
                          {option.label}
                        </Label>
                        <p className="text-xs text-muted-foreground">
                          {option.description}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}

        <DialogFooter>
          <Button variant="outline" onClick={handleClose} disabled={exporting}>
            Cancel
          </Button>
          <Button onClick={handleExport} disabled={exporting}>
            {exporting ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Exporting...
              </>
            ) : (
              <>
                <FolderOpen className="mr-2 h-4 w-4" />
                Export
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export default ExportInstanceDialog;
