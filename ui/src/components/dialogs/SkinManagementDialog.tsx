// Skin management dialog for uploading and managing Minecraft skins
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

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open as openExternal } from "@tauri-apps/plugin-shell";
import { open as openFile } from "@tauri-apps/plugin-dialog";
import {
  Loader2,
  Upload,
  Download,
  User,
  Link,
  RotateCcw,
  FolderOpen,
  ExternalLink,
  RefreshCw,
  AlertCircle,
  Check,
  Sparkles,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Separator } from "@/components/ui/separator";
import { ScrollArea } from "@/components/ui/scroll-area";
import { SkinViewer3D, CapeViewer3D } from "@/components/common";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  AccountInfo,
  PlayerProfileResponse,
  FetchedSkinResponse,
} from "@/types";

interface SkinManagementDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  account: AccountInfo;
  onAccountUpdated: () => void;
}

export function SkinManagementDialog({
  open,
  onOpenChange,
  account,
  onAccountUpdated,
}: SkinManagementDialogProps) {
  const [profile, setProfile] = useState<PlayerProfileResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // Skin data URLs for the 3D viewer (base64 encoded)
  const [skinDataUrl, setSkinDataUrl] = useState<string | null>(null);
  const [capeDataUrl, setCapeDataUrl] = useState<string | null>(null);

  // Form state
  const [selectedVariant, setSelectedVariant] = useState<"classic" | "slim">(
    "classic"
  );
  const [previewVariant, setPreviewVariant] = useState<"classic" | "slim">(
    "classic"
  );
  const [skinUrl, setSkinUrl] = useState("");
  const [importUsername, setImportUsername] = useState("");
  const [importedSkin, setImportedSkin] = useState<FetchedSkinResponse | null>(
    null
  );
  const [importedSkinDataUrl, setImportedSkinDataUrl] = useState<string | null>(
    null
  );
  const [loadingImport, setLoadingImport] = useState(false);
  const [selectedCape, setSelectedCape] = useState<string | null>(null);
  const [hoveredCapeUrl, setHoveredCapeUrl] = useState<string | null>(null);

  // Drag-drop state
  const [isDragging, setIsDragging] = useState(false);

  // Load profile when dialog opens
  useEffect(() => {
    if (open && account.account_type === "Microsoft") {
      loadProfile();
    }
  }, [open, account.id]);

  // Update selected cape when profile loads
  useEffect(() => {
    if (profile?.active_cape) {
      setSelectedCape(profile.active_cape.id);
    } else {
      setSelectedCape(null);
    }
  }, [profile]);

  // Update variant when profile loads
  useEffect(() => {
    if (profile?.active_skin) {
      const variant = profile.active_skin.variant as "classic" | "slim";
      setSelectedVariant(variant);
      setPreviewVariant(variant);
    }
  }, [profile]);

  // Load skin image for 3D viewer
  const loadSkinImage = useCallback(async (url: string) => {
    try {
      const base64 = await invoke<string>("download_skin_image", {
        skinUrl: url,
      });
      return `data:image/png;base64,${base64}`;
    } catch (err) {
      console.error("Failed to load skin image:", err);
      return null;
    }
  }, []);

  // Load and cache skin when profile changes
  useEffect(() => {
    if (profile?.active_skin?.url) {
      loadSkinImage(profile.active_skin.url).then(setSkinDataUrl);

      // Also cache the skin for future use
      invoke("cache_skin_image", {
        uuid: account.uuid,
        skinUrl: profile.active_skin.url,
      }).catch(console.error);
    } else {
      setSkinDataUrl(null);
    }
  }, [profile?.active_skin?.url, loadSkinImage, account.uuid]);

  // Load cape image when profile changes
  useEffect(() => {
    if (profile?.active_cape?.url) {
      loadSkinImage(profile.active_cape.url).then(setCapeDataUrl);
    } else {
      setCapeDataUrl(null);
    }
  }, [profile?.active_cape?.url, loadSkinImage]);

  const loadProfile = async () => {
    setLoading(true);
    setError(null);

    try {
      const data = await invoke<PlayerProfileResponse>("get_player_profile", {
        accountId: account.id,
      });
      setProfile(data);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const showSuccess = (message: string) => {
    setSuccessMessage(message);
    setTimeout(() => setSuccessMessage(null), 3000);
  };

  const handleChangeSkinUrl = async () => {
    if (!skinUrl.trim()) return;

    setSaving(true);
    setError(null);

    try {
      await invoke("change_skin_url", {
        accountId: account.id,
        skinUrl: skinUrl.trim(),
        variant: selectedVariant,
      });
      showSuccess("Skin updated successfully!");
      setSkinUrl("");
      loadProfile();
      onAccountUpdated();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleUploadSkin = async () => {
    try {
      const selected = await openFile({
        multiple: false,
        filters: [{ name: "PNG Image", extensions: ["png"] }],
      });

      if (!selected) return;

      setSaving(true);
      setError(null);

      // Read file using Tauri backend (avoids CSP issues)
      const bytes = await invoke<number[]>("read_file_bytes", {
        filePath: selected,
      });

      await invoke("upload_skin", {
        accountId: account.id,
        imageData: bytes,
        variant: selectedVariant,
      });

      showSuccess("Skin uploaded successfully!");
      loadProfile();
      onAccountUpdated();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleImportFromUsername = async () => {
    if (!importUsername.trim()) return;

    setLoadingImport(true);
    setError(null);
    setImportedSkin(null);
    setImportedSkinDataUrl(null);

    try {
      const skin = await invoke<FetchedSkinResponse>(
        "fetch_skin_from_username",
        {
          username: importUsername.trim(),
        }
      );
      setImportedSkin(skin);

      // Load the skin image for preview
      if (skin.skin_url) {
        const dataUrl = await loadSkinImage(skin.skin_url);
        setImportedSkinDataUrl(dataUrl);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoadingImport(false);
    }
  };

  const handleApplyImportedSkin = async () => {
    if (!importedSkin?.skin_url) return;

    setSaving(true);
    setError(null);

    try {
      await invoke("import_skin_from_username", {
        accountId: account.id,
        username: importedSkin.username,
        useOriginalVariant: false,
        overrideVariant: selectedVariant,
      });

      showSuccess(`Imported skin from ${importedSkin.username}!`);
      setImportedSkin(null);
      setImportedSkinDataUrl(null);
      setImportUsername("");
      loadProfile();
      onAccountUpdated();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleResetSkin = async () => {
    setSaving(true);
    setError(null);

    try {
      await invoke("reset_skin", { accountId: account.id });
      showSuccess("Skin reset to default!");
      loadProfile();
      onAccountUpdated();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleSetCape = async (capeId: string | null) => {
    setSaving(true);
    setError(null);

    try {
      if (capeId) {
        await invoke("set_cape", { accountId: account.id, capeId });
        setSelectedCape(capeId);
        showSuccess("Cape updated!");
      } else {
        await invoke("hide_cape", { accountId: account.id });
        setSelectedCape(null);
        showSuccess("Cape hidden!");
      }
      loadProfile();
      onAccountUpdated();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleOpenSkinsFolder = async () => {
    try {
      await invoke("open_skins_folder");
    } catch (err) {
      setError(String(err));
    }
  };

  const handleOpenMinecraftSkins = () => {
    openExternal("https://www.minecraftskins.com/");
  };

  // Handle variant change for preview (doesn't apply until skin is uploaded/changed)
  const handleVariantChange = (variant: "classic" | "slim") => {
    setSelectedVariant(variant);
    setPreviewVariant(variant);
  };

  // Handle drag-drop file upload
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragging(false);

      const files = e.dataTransfer.files;
      if (files.length === 0) return;

      const file = files[0];
      if (!file.name.toLowerCase().endsWith(".png")) {
        setError("Only PNG files are supported");
        return;
      }

      setSaving(true);
      setError(null);

      try {
        // Read file as ArrayBuffer, then convert to bytes array
        const arrayBuffer = await file.arrayBuffer();
        const bytes = Array.from(new Uint8Array(arrayBuffer));

        await invoke("upload_skin", {
          accountId: account.id,
          imageData: bytes,
          variant: selectedVariant,
        });

        showSuccess("Skin uploaded successfully!");
        loadProfile();
        onAccountUpdated();
      } catch (err) {
        setError(String(err));
      } finally {
        setSaving(false);
      }
    },
    [account.id, selectedVariant, onAccountUpdated]
  );

  // Offline account message
  if (account.account_type === "Offline") {
    return (
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Skin Management</DialogTitle>
            <DialogDescription>Manage your Minecraft skin</DialogDescription>
          </DialogHeader>

          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              Skin management is only available for Microsoft accounts. Offline
              accounts cannot change their skins through the launcher.
            </AlertDescription>
          </Alert>

          <div className="flex justify-end">
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              Close
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <User className="h-5 w-5" />
            Skin Management - {account.username}
          </DialogTitle>
          <DialogDescription>
            Customize your Minecraft appearance
          </DialogDescription>
        </DialogHeader>

        {error && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {successMessage && (
          <Alert className="border-green-500 text-green-500">
            <Check className="h-4 w-4" />
            <AlertDescription>{successMessage}</AlertDescription>
          </Alert>
        )}

        {loading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-8 w-8 animate-spin" />
            <span className="ml-2">Loading profile...</span>
          </div>
        ) : (
          <div className="grid grid-cols-[220px_1fr] gap-6">
            {/* Left side - 3D skin preview */}
            <div className="flex flex-col items-center gap-3">
              <div className="text-sm font-medium text-muted-foreground">
                Current Skin
              </div>

              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <div className="cursor-grab active:cursor-grabbing">
                      <SkinViewer3D
                        skinUrl={skinDataUrl}
                        capeUrl={capeDataUrl}
                        variant={previewVariant}
                        width={180}
                        height={280}
                        autoRotate={true}
                        autoRotateSpeed={1}
                        zoom={0.85}
                        fallbackText={account.username.charAt(0).toUpperCase()}
                      />
                    </div>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>Drag to rotate</p>
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>

              <Badge variant="secondary" className="capitalize">
                {previewVariant} Model
              </Badge>

              <div className="flex gap-2 mt-2">
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={loadProfile}
                        disabled={loading}
                      >
                        <RefreshCw
                          className={`h-4 w-4 ${loading ? "animate-spin" : ""}`}
                        />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Refresh profile</TooltipContent>
                  </Tooltip>
                </TooltipProvider>

                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={handleOpenSkinsFolder}
                      >
                        <FolderOpen className="h-4 w-4" />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Open skins folder</TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              </div>

              <Button
                variant="link"
                size="sm"
                onClick={handleOpenMinecraftSkins}
                className="text-xs"
              >
                Browse Skins
                <ExternalLink className="ml-1 h-3 w-3" />
              </Button>
            </div>

            {/* Right side - Actions */}
            <div className="flex-1">
              <Tabs defaultValue="upload" className="w-full">
                <TabsList className="grid w-full grid-cols-4">
                  <TabsTrigger value="upload">Upload</TabsTrigger>
                  <TabsTrigger value="url">From URL</TabsTrigger>
                  <TabsTrigger value="import">Import</TabsTrigger>
                  <TabsTrigger value="cape">Capes</TabsTrigger>
                </TabsList>

                <ScrollArea className="h-[380px] mt-4 pr-4">
                  {/* Upload Tab */}
                  <TabsContent value="upload" className="space-y-4 mt-0">
                    <div className="rounded-lg border p-4 space-y-4">
                      <div>
                        <Label className="text-base font-semibold flex items-center gap-2">
                          <Sparkles className="h-4 w-4" />
                          Model Type
                        </Label>
                        <p className="text-sm text-muted-foreground mb-3">
                          Classic has wider arms, Slim has thinner arms
                        </p>
                        <RadioGroup
                          value={selectedVariant}
                          onValueChange={(v) =>
                            handleVariantChange(v as "classic" | "slim")
                          }
                          className="flex gap-6"
                        >
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="classic" id="classic" />
                            <Label
                              htmlFor="classic"
                              className="cursor-pointer font-normal"
                            >
                              Classic (Steve)
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="slim" id="slim" />
                            <Label
                              htmlFor="slim"
                              className="cursor-pointer font-normal"
                            >
                              Slim (Alex)
                            </Label>
                          </div>
                        </RadioGroup>
                      </div>
                    </div>

                    <div
                      className={`rounded-lg border-2 border-dashed p-6 space-y-4 transition-colors ${
                        isDragging
                          ? "border-primary bg-primary/10"
                          : "border-muted-foreground/25 hover:border-muted-foreground/50"
                      }`}
                      onDragOver={handleDragOver}
                      onDragLeave={handleDragLeave}
                      onDrop={handleDrop}
                    >
                      <div className="flex flex-col items-center gap-3 text-center">
                        <Upload className={`h-10 w-10 ${isDragging ? "text-primary" : "text-muted-foreground"}`} />
                        <div>
                          <Label className="text-base font-semibold">
                            {isDragging ? "Drop to upload" : "Upload Skin File"}
                          </Label>
                          <p className="text-sm text-muted-foreground mt-1">
                            Drag & drop a PNG file here, or click to browse
                          </p>
                          <p className="text-xs text-muted-foreground mt-1">
                            (64×64 or 64×32 pixels)
                          </p>
                        </div>
                      </div>
                      <Button
                        onClick={handleUploadSkin}
                        disabled={saving}
                        className="w-full"
                        size="lg"
                        variant={isDragging ? "default" : "outline"}
                      >
                        {saving ? (
                          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        ) : (
                          <Upload className="mr-2 h-4 w-4" />
                        )}
                        Choose File
                      </Button>
                    </div>

                    <Separator />

                    <div className="rounded-lg border border-destructive/30 p-4 space-y-3">
                      <Label className="text-base font-semibold text-destructive flex items-center gap-2">
                        <RotateCcw className="h-4 w-4" />
                        Reset Skin
                      </Label>
                      <p className="text-sm text-muted-foreground">
                        Reset to the default Steve/Alex skin
                      </p>
                      <Button
                        variant="destructive"
                        onClick={handleResetSkin}
                        disabled={saving}
                      >
                        {saving ? (
                          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        ) : (
                          <RotateCcw className="mr-2 h-4 w-4" />
                        )}
                        Reset to Default
                      </Button>
                    </div>
                  </TabsContent>

                  {/* URL Tab */}
                  <TabsContent value="url" className="space-y-4 mt-0">
                    <div className="rounded-lg border p-4 space-y-4">
                      <div>
                        <Label className="text-base font-semibold flex items-center gap-2">
                          <Sparkles className="h-4 w-4" />
                          Model Type
                        </Label>
                        <p className="text-sm text-muted-foreground mb-3">
                          Select the model type to use with this skin
                        </p>
                        <RadioGroup
                          value={selectedVariant}
                          onValueChange={(v) =>
                            handleVariantChange(v as "classic" | "slim")
                          }
                          className="flex gap-6"
                        >
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem
                              value="classic"
                              id="url-classic"
                            />
                            <Label
                              htmlFor="url-classic"
                              className="cursor-pointer font-normal"
                            >
                              Classic (Steve)
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="slim" id="url-slim" />
                            <Label
                              htmlFor="url-slim"
                              className="cursor-pointer font-normal"
                            >
                              Slim (Alex)
                            </Label>
                          </div>
                        </RadioGroup>
                      </div>
                    </div>

                    <div className="rounded-lg border p-4 space-y-3">
                      <Label className="text-base font-semibold flex items-center gap-2">
                        <Link className="h-4 w-4" />
                        Skin URL
                      </Label>
                      <p className="text-sm text-muted-foreground">
                        Enter a direct URL to a skin PNG image
                      </p>
                      <div className="flex gap-2">
                        <Input
                          placeholder="https://example.com/skin.png"
                          value={skinUrl}
                          onChange={(e) => setSkinUrl(e.target.value)}
                        />
                        <Button
                          onClick={handleChangeSkinUrl}
                          disabled={saving || !skinUrl.trim()}
                        >
                          {saving ? (
                            <Loader2 className="h-4 w-4 animate-spin" />
                          ) : (
                            <Check className="h-4 w-4" />
                          )}
                        </Button>
                      </div>
                    </div>
                  </TabsContent>

                  {/* Import Tab */}
                  <TabsContent value="import" className="space-y-4 mt-0">
                    <div className="rounded-lg border p-4 space-y-3">
                      <Label className="text-base font-semibold flex items-center gap-2">
                        <Download className="h-4 w-4" />
                        Import from Player
                      </Label>
                      <p className="text-sm text-muted-foreground">
                        Enter a Minecraft username to preview and import their
                        skin
                      </p>
                      <div className="flex gap-2">
                        <Input
                          placeholder="Enter username..."
                          value={importUsername}
                          onChange={(e) => setImportUsername(e.target.value)}
                          onKeyDown={(e) =>
                            e.key === "Enter" && handleImportFromUsername()
                          }
                        />
                        <Button
                          onClick={handleImportFromUsername}
                          disabled={loadingImport || !importUsername.trim()}
                        >
                          {loadingImport ? (
                            <Loader2 className="h-4 w-4 animate-spin" />
                          ) : (
                            <Download className="h-4 w-4" />
                          )}
                        </Button>
                      </div>
                    </div>

                    {importedSkin && (
                      <div className="rounded-lg border p-4 space-y-4">
                        <div className="flex gap-4">
                          <div className="flex flex-col items-center gap-2">
                            <SkinViewer3D
                              skinUrl={importedSkinDataUrl}
                              variant={
                                importedSkin.skin_variant as "classic" | "slim"
                              }
                              width={120}
                              height={180}
                              autoRotate={true}
                              autoRotateSpeed={1.5}
                              zoom={0.8}
                              fallbackText={importedSkin.username
                                .charAt(0)
                                .toUpperCase()}
                            />
                            <div className="text-sm font-medium">
                              {importedSkin.username}
                            </div>
                            <Badge variant="secondary" className="capitalize">
                              {importedSkin.skin_variant}
                            </Badge>
                          </div>

                          <div className="flex-1 space-y-4">
                            <div>
                              <Label className="text-sm font-semibold">
                                Apply as Model Type
                              </Label>
                              <RadioGroup
                                value={selectedVariant}
                                onValueChange={(v) =>
                                  handleVariantChange(v as "classic" | "slim")
                                }
                                className="flex gap-4 mt-2"
                              >
                                <div className="flex items-center space-x-2">
                                  <RadioGroupItem
                                    value="classic"
                                    id="import-classic"
                                  />
                                  <Label
                                    htmlFor="import-classic"
                                    className="cursor-pointer font-normal"
                                  >
                                    Classic
                                  </Label>
                                </div>
                                <div className="flex items-center space-x-2">
                                  <RadioGroupItem
                                    value="slim"
                                    id="import-slim"
                                  />
                                  <Label
                                    htmlFor="import-slim"
                                    className="cursor-pointer font-normal"
                                  >
                                    Slim
                                  </Label>
                                </div>
                              </RadioGroup>
                            </div>

                            <Button
                              onClick={handleApplyImportedSkin}
                              disabled={saving || !importedSkin.skin_url}
                              className="w-full"
                            >
                              {saving ? (
                                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                              ) : (
                                <Check className="mr-2 h-4 w-4" />
                              )}
                              Apply This Skin
                            </Button>

                            {!importedSkin.skin_url && (
                              <p className="text-sm text-amber-500">
                                This player doesn't have a custom skin.
                              </p>
                            )}
                          </div>
                        </div>
                      </div>
                    )}
                  </TabsContent>

                  {/* Capes Tab */}
                  <TabsContent value="cape" className="space-y-4 mt-0">
                    <div className="rounded-lg border p-4 space-y-4">
                      <div>
                        <Label className="text-base font-semibold">
                          Your Capes
                        </Label>
                        <p className="text-sm text-muted-foreground mb-4">
                          Select a cape to wear, or choose to hide your cape
                        </p>

                        {profile?.capes && profile.capes.length > 0 ? (
                          <div className="flex gap-6">
                            {/* Cape preview with 3D viewer */}
                            <div className="flex flex-col items-center gap-2">
                              <div className="text-xs text-muted-foreground">
                                Preview
                              </div>
                              <CapeViewer3D
                                skinUrl={skinDataUrl}
                                capeUrl={hoveredCapeUrl || capeDataUrl}
                                variant={previewVariant}
                                width={140}
                                height={200}
                              />
                            </div>

                            {/* Cape selection grid */}
                            <div className="flex-1">
                              <div className="grid grid-cols-3 gap-3">
                                {/* No cape option */}
                                <button
                                  onClick={() => handleSetCape(null)}
                                  onMouseEnter={() => setHoveredCapeUrl(null)}
                                  onMouseLeave={() => setHoveredCapeUrl(null)}
                                  disabled={saving}
                                  className={`flex flex-col items-center p-3 rounded-lg border-2 transition-all hover:bg-muted/50 ${
                                    selectedCape === null
                                      ? "border-primary bg-primary/10"
                                      : "border-border"
                                  }`}
                                >
                                  <div className="w-10 h-14 bg-muted rounded flex items-center justify-center text-muted-foreground text-lg">
                                    ✕
                                  </div>
                                  <span className="text-xs mt-2">No Cape</span>
                                </button>

                                {/* Cape options */}
                                {profile.capes.map((cape) => (
                                  <CapeButton
                                    key={cape.id}
                                    cape={cape}
                                    isSelected={selectedCape === cape.id}
                                    disabled={saving}
                                    onClick={() => handleSetCape(cape.id)}
                                    onHover={setHoveredCapeUrl}
                                    loadImage={loadSkinImage}
                                  />
                                ))}
                              </div>
                            </div>
                          </div>
                        ) : (
                          <Alert>
                            <AlertDescription>
                              You don't have any capes. Capes can be obtained
                              from Minecraft events, Realms, or marketplace
                              purchases.
                            </AlertDescription>
                          </Alert>
                        )}
                      </div>
                    </div>
                  </TabsContent>
                </ScrollArea>
              </Tabs>
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}

// Cape button component with image loading
function CapeButton({
  cape,
  isSelected,
  disabled,
  onClick,
  onHover,
  loadImage,
}: {
  cape: { id: string; url: string; alias: string | null; is_active: boolean };
  isSelected: boolean;
  disabled: boolean;
  onClick: () => void;
  onHover: (url: string | null) => void;
  loadImage: (url: string) => Promise<string | null>;
}) {
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    loadImage(cape.url)
      .then(setImageUrl)
      .finally(() => setLoading(false));
  }, [cape.url, loadImage]);

  return (
    <button
      onClick={onClick}
      onMouseEnter={() => onHover(imageUrl)}
      onMouseLeave={() => onHover(null)}
      disabled={disabled}
      className={`flex flex-col items-center p-3 rounded-lg border-2 transition-all hover:bg-muted/50 ${
        isSelected ? "border-primary bg-primary/10" : "border-border"
      }`}
    >
      {loading ? (
        <div className="w-10 h-14 bg-muted rounded flex items-center justify-center">
          <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
        </div>
      ) : imageUrl ? (
        <img
          src={imageUrl}
          alt={cape.alias || "Cape"}
          className="w-10 h-14 object-contain"
          style={{ imageRendering: "pixelated" }}
        />
      ) : (
        <div className="w-10 h-14 bg-muted rounded flex items-center justify-center text-muted-foreground">
          ?
        </div>
      )}
      <span className="text-xs mt-2 truncate max-w-full">
        {cape.alias || "Cape"}
      </span>
      {cape.is_active && (
        <Badge variant="default" className="mt-1 text-[10px] px-1.5 py-0">
          Active
        </Badge>
      )}
    </button>
  );
}
