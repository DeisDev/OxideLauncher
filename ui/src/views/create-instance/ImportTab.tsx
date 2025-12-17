import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from "@tauri-apps/api/event";
import {
  Upload,
  FileArchive,
  Link as LinkIcon,
  Folder,
  Loader2,
  Check,
  AlertCircle,
  Info,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { cn } from "@/lib/utils";
import { IMPORT_FORMATS } from "./types";

// Progress event from backend
interface ImportProgressEvent {
  stage: string;
  message: string;
  progress: number;
}

export function ImportTab() {
  const navigate = useNavigate();
  
  // State
  const [activeImportTab, setActiveImportTab] = useState<"file" | "url" | "folder">("file");
  const [isDragging, setIsDragging] = useState(false);
  const [importing, setImporting] = useState(false);
  const [importProgress, setImportProgress] = useState(0);
  const [importStatus, setImportStatus] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  
  // URL import
  const [importUrl, setImportUrl] = useState("");
  const [nameOverride, setNameOverride] = useState("");
  
  // Folder import state
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [detectedFormat, setDetectedFormat] = useState<string | null>(null);

  // Listen for import progress events
  useEffect(() => {
    const unlistenProgress = listen<ImportProgressEvent>("import_progress", (event) => {
      const { stage, message, progress } = event.payload;
      setImportStatus(`${stage}: ${message}`);
      setImportProgress(progress);
    });

    return () => {
      unlistenProgress.then((fn) => fn());
    };
  }, []);

  // Set up Tauri drag-drop event listener
  useEffect(() => {
    const webview = getCurrentWebviewWindow();
    let unlisten: (() => void) | undefined;

    webview.onDragDropEvent((event) => {
      if (event.payload.type === 'over' || event.payload.type === 'enter') {
        setIsDragging(true);
      } else if (event.payload.type === 'drop') {
        setIsDragging(false);
        if (event.payload.paths && event.payload.paths.length > 0) {
          handleDroppedPaths(event.payload.paths);
        }
      } else if (event.payload.type === 'leave') {
        setIsDragging(false);
      }
    }).then(fn => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleDroppedPaths = useCallback(async (paths: string[]) => {
    // Take only the first file
    const filePath = paths[0];
    if (!filePath) return;

    // Check if it's a supported file type
    const ext = filePath.toLowerCase().split('.').pop();
    const supportedExtensions = ['.zip', '.mrpack', '.oxide'];
    const hasSupported = supportedExtensions.some(e => filePath.toLowerCase().endsWith(e));
    
    if (!hasSupported) {
      setError(`Unsupported file format: .${ext}. Supported formats: .zip, .mrpack, .oxide`);
      return;
    }

    await importFromFile(filePath);
  }, []);

  const importFromFile = async (filePath: string) => {
    setImporting(true);
    setImportProgress(0);
    setImportStatus("Starting import...");
    setError(null);
    setSuccess(false);

    try {
      await invoke("import_instance_from_file", {
        filePath,
        nameOverride: nameOverride || null,
      });
      
      setImportProgress(100);
      setImportStatus("Import complete!");
      setSuccess(true);
      
      // Navigate after delay
      setTimeout(() => {
        navigate("/");
      }, 1500);
    } catch (err) {
      console.error("Import failed:", err);
      setError(String(err));
    } finally {
      setImporting(false);
    }
  };

  const importFromUrl = async () => {
    if (!importUrl) {
      setError("Please enter a URL");
      return;
    }

    setImporting(true);
    setImportProgress(0);
    setImportStatus("Downloading...");
    setError(null);
    setSuccess(false);

    try {
      await invoke("import_instance_from_url", {
        url: importUrl,
        nameOverride: nameOverride || null,
      });
      
      setImportProgress(100);
      setImportStatus("Import complete!");
      setSuccess(true);
      
      // Navigate after delay
      setTimeout(() => {
        navigate("/");
      }, 1500);
    } catch (err) {
      console.error("Import failed:", err);
      setError(String(err));
    } finally {
      setImporting(false);
    }
  };

  const importFromFolder = async () => {
    if (!selectedFolder) {
      setError("Please select a folder");
      return;
    }

    setImporting(true);
    setImportProgress(0);
    setImportStatus("Importing from folder...");
    setError(null);
    setSuccess(false);

    try {
      await invoke("import_instance_from_folder", {
        folderPath: selectedFolder,
        nameOverride: nameOverride || null,
      });
      
      setImportProgress(100);
      setImportStatus("Import complete!");
      setSuccess(true);
      
      // Navigate after delay
      setTimeout(() => {
        navigate("/");
      }, 1500);
    } catch (err) {
      console.error("Import failed:", err);
      setError(String(err));
    } finally {
      setImporting(false);
    }
  };

  const selectFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: "Modpack Files", extensions: ["zip", "mrpack", "oxide"] },
        ],
      });

      if (selected && typeof selected === "string") {
        await importFromFile(selected);
      }
    } catch (error) {
      console.error("Failed to select file:", error);
      setError(String(error));
    }
  };

  const selectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === "string") {
        setSelectedFolder(selected);
        // Try to detect the format
        try {
          const format = await invoke<string | null>("detect_instance_format", {
            folderPath: selected,
          });
          setDetectedFormat(format);
        } catch {
          setDetectedFormat(null);
        }
      }
    } catch (error) {
      console.error("Failed to select folder:", error);
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    // HTML5 drag-drop fallback (used when Tauri native doesn't trigger)
    // This is handled by the Tauri drag-drop event listener
  };

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <ScrollArea className="flex-1">
        <div className="space-y-4 pr-4">
          {/* Error/Success display */}
          {error && (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
          
          {success && (
            <Alert className="border-green-500 bg-green-500/10">
              <Check className="h-4 w-4 text-green-500" />
              <AlertDescription className="text-green-700 dark:text-green-400">
                Instance imported successfully! Redirecting...
              </AlertDescription>
            </Alert>
          )}

          {/* Import progress */}
          {importing && (
            <Card>
              <CardContent className="pt-6">
                <div className="space-y-2">
                  <div className="flex items-center justify-between text-sm">
                    <span>{importStatus}</span>
                    <span>{importProgress}%</span>
                  </div>
                  <Progress value={importProgress} />
                </div>
              </CardContent>
            </Card>
          )}

          {/* Name Override */}
          <div className="space-y-1.5">
            <Label htmlFor="nameOverride" className="text-sm">Instance Name (optional)</Label>
            <Input
              id="nameOverride"
              value={nameOverride}
              onChange={(e) => setNameOverride(e.target.value)}
              placeholder="Leave empty to use modpack name"
              disabled={importing}
              className="h-9"
            />
          </div>

          {/* Import Tabs */}
          <Tabs value={activeImportTab} onValueChange={(v) => setActiveImportTab(v as "file" | "url" | "folder")}>
            <TabsList className="grid w-full grid-cols-3 h-9">
              <TabsTrigger value="file" disabled={importing} className="text-xs sm:text-sm">
                <FileArchive className="h-3.5 w-3.5 mr-1 sm:mr-2" />
                File
              </TabsTrigger>
              <TabsTrigger value="url" disabled={importing} className="text-xs sm:text-sm">
                <LinkIcon className="h-3.5 w-3.5 mr-1 sm:mr-2" />
                URL
              </TabsTrigger>
              <TabsTrigger value="folder" disabled={importing} className="text-xs sm:text-sm">
                <Folder className="h-3.5 w-3.5 mr-1 sm:mr-2" />
                Folder
              </TabsTrigger>
            </TabsList>

            {/* File Import Tab */}
            <TabsContent value="file" className="space-y-3">
              {/* Drop Zone */}
              <div
                className={cn(
                  "border-2 border-dashed rounded-lg p-6 text-center transition-colors cursor-pointer",
                  isDragging 
                    ? "border-primary bg-primary/5" 
                    : "border-muted-foreground/25 hover:border-primary/50",
                  importing && "pointer-events-none opacity-50"
                )}
                onClick={selectFile}
                onDragOver={handleDragOver}
                onDragLeave={handleDragLeave}
                onDrop={handleDrop}
              >
                <Upload className="h-8 w-8 mx-auto mb-3 text-muted-foreground" />
                <p className="text-base font-medium mb-1">
                  {isDragging ? "Drop file here" : "Drag & drop modpack file"}
                </p>
                <p className="text-xs text-muted-foreground mb-3">
                  or click to browse
                </p>
                <Button variant="outline" size="sm" disabled={importing} onClick={(e) => { e.stopPropagation(); selectFile(); }}>
                  Select File
                </Button>
              </div>
            </TabsContent>

            {/* URL Import Tab */}
            <TabsContent value="url" className="space-y-3">
              <div className="space-y-1.5">
                <Label htmlFor="importUrl" className="text-sm">Modpack URL</Label>
                <div className="flex gap-2">
                  <Input
                    id="importUrl"
                    value={importUrl}
                    onChange={(e) => setImportUrl(e.target.value)}
                    placeholder="https://..."
                    disabled={importing}
                    className="h-9"
                  />
                  <Button onClick={importFromUrl} disabled={importing || !importUrl} size="sm">
                    {importing ? <Loader2 className="h-4 w-4 animate-spin" /> : "Import"}
                  </Button>
                </div>
                <p className="text-xs text-muted-foreground">
                  Direct download link to a .zip, .mrpack, or .oxide file
                </p>
              </div>

              <Alert className="py-2">
                <Info className="h-3.5 w-3.5" />
                <AlertDescription className="text-xs">
                  Paste direct download links from Modrinth, CurseForge, or Technic.
                </AlertDescription>
              </Alert>
            </TabsContent>

            {/* Folder Import Tab */}
            <TabsContent value="folder" className="space-y-3">
              <div className="space-y-1.5">
                <Label className="text-sm">Instance Folder</Label>
                <div className="flex gap-2">
                  <Input
                    value={selectedFolder || ""}
                    placeholder="Select a folder..."
                    readOnly
                    className="h-9"
                  />
                  <Button variant="outline" size="sm" onClick={selectFolder} disabled={importing}>
                    Browse
                  </Button>
                </div>
              </div>

              {selectedFolder && (
                <div className="space-y-2">
                  {detectedFormat ? (
                    <Alert className="border-green-500 bg-green-500/10 py-2">
                      <Check className="h-3.5 w-3.5 text-green-500" />
                      <AlertDescription className="text-green-700 dark:text-green-400 text-xs">
                        Detected: <strong>{detectedFormat}</strong>
                      </AlertDescription>
                    </Alert>
                  ) : (
                    <Alert className="py-2">
                      <AlertCircle className="h-3.5 w-3.5" />
                      <AlertDescription className="text-xs">
                        Could not detect format. Will attempt generic import.
                      </AlertDescription>
                    </Alert>
                  )}

                  <Button onClick={importFromFolder} disabled={importing} className="w-full" size="sm">
                    {importing ? (
                      <>
                        <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                        Importing...
                      </>
                    ) : (
                      "Import Folder"
                    )}
                  </Button>
                </div>
              )}
            </TabsContent>
          </Tabs>

          {/* Supported Formats */}
          <Card>
            <CardHeader className="pb-2 pt-3 px-3">
              <CardTitle className="text-xs font-medium">Supported Formats</CardTitle>
            </CardHeader>
            <CardContent className="px-3 pb-3 pt-0">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-1.5">
                {IMPORT_FORMATS.map((format) => {
                  const categoryColors = {
                    native: "bg-primary/10 border-primary/30 text-primary",
                    popular: "bg-green-500/10 border-green-500/30 text-green-600 dark:text-green-400",
                    legacy: "bg-orange-500/10 border-orange-500/30 text-orange-600 dark:text-orange-400",
                  };
                  const categoryLabels = {
                    native: "Native",
                    popular: "Popular",
                    legacy: "Legacy",
                  };
                  return (
                    <div
                      key={format.id}
                      className={cn(
                        "flex items-center gap-2 p-2 rounded-md border text-xs",
                        categoryColors[format.category]
                      )}
                    >
                      <FileArchive className="h-3.5 w-3.5 flex-shrink-0" />
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-1.5">
                          <span className="font-medium truncate">{format.name}</span>
                          <span className="text-[10px] opacity-70 hidden sm:inline">
                            ({categoryLabels[format.category]})
                          </span>
                        </div>
                        <p className="text-[10px] opacity-70 truncate">
                          {format.extensions.join(", ")}
                        </p>
                      </div>
                    </div>
                  );
                })}
              </div>
            </CardContent>
          </Card>

          {/* Bottom Actions */}
          <div className="flex justify-end gap-2 pt-3 border-t">
            <Button type="button" variant="outline" size="sm" onClick={() => navigate("/")}>
              Cancel
            </Button>
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}