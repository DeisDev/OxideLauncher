// Import instance dialog supporting multiple modpack formats
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
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import {
  Loader2,
  FileUp,
  Upload,
  CheckCircle,
  AlertCircle,
  FileArchive,
  Package,
  AlertTriangle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Progress } from "@/components/ui/progress";
import { Alert, AlertDescription } from "@/components/ui/alert";

interface ImportInstanceDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onImportComplete: () => void;
}

interface ImportTypeInfo {
  format_type: string;
  display_name: string;
}

interface ImportResult {
  instance_id: string;
  name: string;
  minecraft_version: string;
  mod_loader_type: string | null;
  mod_loader_version: string | null;
  files_to_download: number;
  warnings: string[];
}

// Format badges with colors
const formatColors: Record<string, { bg: string; text: string }> = {
  oxide: { bg: "bg-orange-500/20", text: "text-orange-600 dark:text-orange-400" },
  modrinth: { bg: "bg-green-500/20", text: "text-green-600 dark:text-green-400" },
  curseforge: { bg: "bg-orange-600/20", text: "text-orange-700 dark:text-orange-300" },
  prism: { bg: "bg-purple-500/20", text: "text-purple-600 dark:text-purple-400" },
  unknown: { bg: "bg-red-500/20", text: "text-red-600 dark:text-red-400" },
};

export function ImportInstanceDialog({
  open,
  onOpenChange,
  onImportComplete,
}: ImportInstanceDialogProps) {
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [detectedFormat, setDetectedFormat] = useState<ImportTypeInfo | null>(null);
  const [nameOverride, setNameOverride] = useState("");
  const [detecting, setDetecting] = useState(false);
  const [importing, setImporting] = useState(false);
  const [progress, setProgress] = useState(0);
  const [status, setStatus] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<ImportResult | null>(null);

  const handleSelectFile = async () => {
    try {
      setError(null);
      setDetectedFormat(null);
      setResult(null);
      
      const file = await openFileDialog({
        multiple: false,
        filters: [
          { 
            name: "Instance Archives", 
            extensions: ["oxide", "mrpack", "zip"] 
          },
          { name: "All Files", extensions: ["*"] },
        ],
      });

      if (!file || Array.isArray(file)) {
        return;
      }

      setSelectedFile(file);
      setDetecting(true);

      // Detect format
      const formatInfo = await invoke<ImportTypeInfo>("detect_import_format", {
        archivePath: file,
      });

      setDetectedFormat(formatInfo);
      setDetecting(false);
    } catch (err) {
      console.error("Failed to detect format:", err);
      setError(String(err));
      setDetecting(false);
    }
  };

  const handleImport = async () => {
    if (!selectedFile) return;

    try {
      setError(null);
      setImporting(true);
      setProgress(0);
      setStatus("Importing instance...");

      const importResult = await invoke<ImportResult>("import_instance_from_file", {
        archivePath: selectedFile,
        nameOverride: nameOverride || null,
      });

      setProgress(100);
      setStatus("Import complete!");
      setResult(importResult);

      // Notify parent and close after delay
      setTimeout(() => {
        onImportComplete();
        handleClose();
      }, 2000);
    } catch (err) {
      console.error("Import failed:", err);
      setError(String(err));
      setImporting(false);
      setProgress(0);
      setStatus("");
    }
  };

  const handleClose = () => {
    if (!importing) {
      setSelectedFile(null);
      setDetectedFormat(null);
      setNameOverride("");
      setError(null);
      setResult(null);
      setProgress(0);
      setStatus("");
      onOpenChange(false);
    }
  };

  const formatColor = detectedFormat 
    ? formatColors[detectedFormat.format_type] || formatColors.unknown
    : formatColors.unknown;

  const canImport = selectedFile && detectedFormat && detectedFormat.format_type !== "unknown";

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FileUp className="h-5 w-5" />
            Import Instance
          </DialogTitle>
          <DialogDescription>
            Import an instance from OxideLauncher, Modrinth, CurseForge, or Prism Launcher
          </DialogDescription>
        </DialogHeader>

        {result ? (
          // Success state
          <div className="py-6 space-y-4">
            <div className="flex items-center justify-center">
              <div className="h-16 w-16 rounded-full bg-green-500/20 flex items-center justify-center">
                <CheckCircle className="h-8 w-8 text-green-500" />
              </div>
            </div>
            <div className="text-center space-y-2">
              <h3 className="font-semibold text-lg">{result.name}</h3>
              <div className="flex items-center justify-center gap-2">
                <Badge variant="secondary">{result.minecraft_version}</Badge>
                {result.mod_loader_type && (
                  <Badge variant="outline">
                    {result.mod_loader_type}
                    {result.mod_loader_version && ` ${result.mod_loader_version}`}
                  </Badge>
                )}
              </div>
            </div>
            {result.warnings.length > 0 && (
              <Alert>
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>
                  <ul className="list-disc list-inside text-sm">
                    {result.warnings.map((warning, i) => (
                      <li key={i}>{warning}</li>
                    ))}
                  </ul>
                </AlertDescription>
              </Alert>
            )}
          </div>
        ) : importing ? (
          // Importing state
          <div className="py-6 space-y-4">
            <div className="flex items-center gap-3">
              <Loader2 className="h-5 w-5 animate-spin text-primary" />
              <span className="text-sm">{status}</span>
            </div>
            <Progress value={progress} className="h-2" />
          </div>
        ) : (
          // Selection state
          <div className="py-4 space-y-4">
            {error && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}

            {/* File Selection */}
            <div className="space-y-2">
              <Label>Instance Archive</Label>
              <div className="flex gap-2">
                <Button
                  variant="outline"
                  className="flex-1 justify-start"
                  onClick={handleSelectFile}
                  disabled={detecting}
                >
                  {detecting ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      Detecting format...
                    </>
                  ) : selectedFile ? (
                    <>
                      <FileArchive className="mr-2 h-4 w-4" />
                      <span className="truncate">
                        {selectedFile.split(/[/\\]/).pop()}
                      </span>
                    </>
                  ) : (
                    <>
                      <Upload className="mr-2 h-4 w-4" />
                      Select file...
                    </>
                  )}
                </Button>
              </div>
            </div>

            {/* Detected Format */}
            {detectedFormat && (
              <div className="space-y-2">
                <Label>Detected Format</Label>
                <div className="flex items-center gap-2">
                  <Badge className={`${formatColor.bg} ${formatColor.text} border-0`}>
                    {detectedFormat.display_name}
                  </Badge>
                  {detectedFormat.format_type === "unknown" && (
                    <span className="text-sm text-muted-foreground">
                      Unable to detect format
                    </span>
                  )}
                </div>
              </div>
            )}

            {/* Name Override */}
            {canImport && (
              <div className="space-y-2">
                <Label htmlFor="nameOverride">Instance Name (optional)</Label>
                <Input
                  id="nameOverride"
                  placeholder="Leave empty to use name from archive"
                  value={nameOverride}
                  onChange={(e) => setNameOverride(e.target.value)}
                />
              </div>
            )}

            {/* Supported Formats Info */}
            {!selectedFile && (
              <div className="pt-2 space-y-2">
                <Label className="text-muted-foreground">Supported Formats</Label>
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <div className="flex items-center gap-2">
                    <Badge className={`${formatColors.oxide.bg} ${formatColors.oxide.text} border-0`}>
                      .oxide
                    </Badge>
                    <span className="text-muted-foreground">OxideLauncher</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge className={`${formatColors.modrinth.bg} ${formatColors.modrinth.text} border-0`}>
                      .mrpack
                    </Badge>
                    <span className="text-muted-foreground">Modrinth</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge className={`${formatColors.curseforge.bg} ${formatColors.curseforge.text} border-0`}>
                      .zip
                    </Badge>
                    <span className="text-muted-foreground">CurseForge</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge className={`${formatColors.prism.bg} ${formatColors.prism.text} border-0`}>
                      .zip
                    </Badge>
                    <span className="text-muted-foreground">Prism Launcher</span>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}

        <DialogFooter>
          {!result && (
            <>
              <Button variant="outline" onClick={handleClose} disabled={importing}>
                Cancel
              </Button>
              <Button 
                onClick={handleImport} 
                disabled={!canImport || importing}
              >
                {importing ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Importing...
                  </>
                ) : (
                  <>
                    <Package className="mr-2 h-4 w-4" />
                    Import
                  </>
                )}
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export default ImportInstanceDialog;
