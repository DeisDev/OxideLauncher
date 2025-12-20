// Skin management dialog for uploading and managing player skins
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
import { useSearchParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { open as openExternal } from "@tauri-apps/plugin-shell";
import { open as openFile } from "@tauri-apps/plugin-dialog";
import { emit } from "@tauri-apps/api/event";
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Separator } from "@/components/ui/separator";
import { ScrollArea } from "@/components/ui/scroll-area";
import { SkinViewer3D, CapeViewer3D } from "@/components/common";
import { DialogWindowHeader } from "@/components/common/DialogWindowHeader";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import {
  PlayerProfileResponse,
  FetchedSkinResponse,
} from "@/types";

export function SkinManagementPage() {
  const [searchParams] = useSearchParams();
  
  // Account parameters from URL
  const accountId = searchParams.get("accountId") || "";
  const accountUsername = searchParams.get("username") || "";
  const accountUuid = searchParams.get("uuid") || "";
  const accountType = (searchParams.get("accountType") || "Microsoft") as "Microsoft" | "Offline";

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
  const [_hoveredCapeUrl, setHoveredCapeUrl] = useState<string | null>(null);

  // Drag-drop state
  const [isDragging, setIsDragging] = useState(false);

  // Load profile when page loads
  useEffect(() => {
    if (accountId && accountType === "Microsoft") {
      loadProfile();
    } else {
      setLoading(false);
    }
  }, [accountId, accountType]);

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
        uuid: accountUuid,
        skinUrl: profile.active_skin.url,
      }).catch(console.error);
    } else {
      setSkinDataUrl(null);
    }
  }, [profile?.active_skin?.url, loadSkinImage, accountUuid]);

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
        accountId: accountId,
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

  // Emit account updated event for main window
  const onAccountUpdated = () => {
    emit("account-updated", {});
  };

  const handleChangeSkinUrl = async () => {
    if (!skinUrl.trim()) return;

    setSaving(true);
    setError(null);

    try {
      await invoke("change_skin_url", {
        accountId: accountId,
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
        accountId: accountId,
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
        accountId: accountId,
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
      await invoke("reset_skin", { accountId: accountId });
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
        await invoke("set_cape", { accountId: accountId, capeId });
        setSelectedCape(capeId);
        showSuccess("Cape updated!");
      } else {
        await invoke("hide_cape", { accountId: accountId });
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

  // Handle variant change for preview
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
          accountId: accountId,
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
    [accountId, selectedVariant]
  );

  // Offline account message
  if (accountType === "Offline") {
    return (
      <div className="flex flex-col h-screen bg-background">
        <DialogWindowHeader 
          title={`Skin Management - ${accountUsername}`}
          icon={<User className="h-5 w-5" />}
        />
        <div className="flex-1 p-6">
          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              Skin management is only available for Microsoft accounts. Offline
              accounts cannot change their skins through the launcher.
            </AlertDescription>
          </Alert>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-screen bg-background">
      <DialogWindowHeader 
        title={`Skin Management - ${accountUsername}`}
        icon={<User className="h-5 w-5" />}
      />

      <div className="flex-1 p-6 overflow-hidden">
        {error && (
          <Alert variant="destructive" className="mb-4">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {successMessage && (
          <Alert className="border-green-500 text-green-500 mb-4">
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
          <div className="grid grid-cols-[240px_1fr] gap-6 h-full">
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
                        width={200}
                        height={320}
                        autoRotate={true}
                        autoRotateSpeed={1}
                        zoom={0.85}
                        fallbackText={accountUsername.charAt(0).toUpperCase()}
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
            <div className="flex-1 min-h-0">
              <Tabs defaultValue="upload" className="h-full flex flex-col">
                <TabsList className="grid w-full grid-cols-4">
                  <TabsTrigger value="upload">Upload</TabsTrigger>
                  <TabsTrigger value="url">From URL</TabsTrigger>
                  <TabsTrigger value="import">Import</TabsTrigger>
                  <TabsTrigger value="cape">Capes</TabsTrigger>
                </TabsList>

                <ScrollArea className="flex-1 mt-4 pr-4">
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
                            <RadioGroupItem value="classic" id="classic-pop" />
                            <Label
                              htmlFor="classic-pop"
                              className="cursor-pointer font-normal"
                            >
                              Classic (Steve)
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="slim" id="slim-pop" />
                            <Label
                              htmlFor="slim-pop"
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
                            <RadioGroupItem value="classic" id="classic-url-pop" />
                            <Label
                              htmlFor="classic-url-pop"
                              className="cursor-pointer font-normal"
                            >
                              Classic (Steve)
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="slim" id="slim-url-pop" />
                            <Label
                              htmlFor="slim-url-pop"
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
                        Enter a direct URL to a skin PNG file
                      </p>
                      <div className="flex gap-2">
                        <Input
                          placeholder="https://example.com/skin.png"
                          value={skinUrl}
                          onChange={(e) => setSkinUrl(e.target.value)}
                          disabled={saving}
                        />
                        <Button
                          onClick={handleChangeSkinUrl}
                          disabled={saving || !skinUrl.trim()}
                        >
                          {saving ? (
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                          ) : (
                            <Download className="mr-2 h-4 w-4" />
                          )}
                          Apply
                        </Button>
                      </div>
                    </div>
                  </TabsContent>

                  {/* Import Tab */}
                  <TabsContent value="import" className="space-y-4 mt-0">
                    <div className="rounded-lg border p-4 space-y-4">
                      <div>
                        <Label className="text-base font-semibold flex items-center gap-2">
                          <Sparkles className="h-4 w-4" />
                          Model Type
                        </Label>
                        <p className="text-sm text-muted-foreground mb-3">
                          Select the model type to use with the imported skin
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
                              id="classic-import-pop"
                            />
                            <Label
                              htmlFor="classic-import-pop"
                              className="cursor-pointer font-normal"
                            >
                              Classic (Steve)
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="slim" id="slim-import-pop" />
                            <Label
                              htmlFor="slim-import-pop"
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
                        <User className="h-4 w-4" />
                        Import from Player
                      </Label>
                      <p className="text-sm text-muted-foreground">
                        Enter a Minecraft username to copy their skin
                      </p>
                      <div className="flex gap-2">
                        <Input
                          placeholder="Username"
                          value={importUsername}
                          onChange={(e) => setImportUsername(e.target.value)}
                          disabled={loadingImport || saving}
                        />
                        <Button
                          onClick={handleImportFromUsername}
                          disabled={
                            loadingImport || saving || !importUsername.trim()
                          }
                        >
                          {loadingImport ? (
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                          ) : (
                            <Download className="mr-2 h-4 w-4" />
                          )}
                          Fetch
                        </Button>
                      </div>
                    </div>

                    {/* Imported skin preview */}
                    {importedSkin && (
                      <div className="rounded-lg border p-4 space-y-4">
                        <div className="flex items-start gap-4">
                          <SkinViewer3D
                            skinUrl={importedSkinDataUrl}
                            capeUrl={null}
                            variant={selectedVariant}
                            width={120}
                            height={180}
                            autoRotate={true}
                            autoRotateSpeed={0.8}
                            zoom={0.9}
                            fallbackText={importedSkin.username
                              .charAt(0)
                              .toUpperCase()}
                          />
                          <div className="flex-1 space-y-2">
                            <h4 className="font-semibold">
                              {importedSkin.username}
                            </h4>
                            <p className="text-sm text-muted-foreground">
                              UUID: {importedSkin.uuid}
                            </p>
                            <Badge variant="secondary" className="capitalize">
                              {importedSkin.skin_variant} Model
                            </Badge>
                            <div className="pt-2">
                              <Button
                                onClick={handleApplyImportedSkin}
                                disabled={saving}
                                className="w-full"
                              >
                                {saving ? (
                                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                ) : (
                                  <Check className="mr-2 h-4 w-4" />
                                )}
                                Apply This Skin
                              </Button>
                            </div>
                          </div>
                        </div>
                      </div>
                    )}
                  </TabsContent>

                  {/* Cape Tab */}
                  <TabsContent value="cape" className="space-y-4 mt-0">
                    {profile?.capes && profile.capes.length > 0 ? (
                      <>
                        <div className="text-sm text-muted-foreground mb-2">
                          Select a cape to wear, or hide your cape
                        </div>

                        <div className="grid grid-cols-2 gap-3">
                          {/* No cape option */}
                          <div
                            className={cn(
                              "relative rounded-lg border-2 p-3 cursor-pointer transition-all hover:border-primary/50",
                              selectedCape === null
                                ? "border-primary bg-primary/5"
                                : "border-muted"
                            )}
                            onClick={() => handleSetCape(null)}
                          >
                            <div className="flex flex-col items-center gap-2">
                              <div className="h-[100px] w-[80px] rounded bg-muted flex items-center justify-center">
                                <User className="h-8 w-8 text-muted-foreground" />
                              </div>
                              <span className="text-sm font-medium">
                                No Cape
                              </span>
                            </div>
                            {selectedCape === null && (
                              <div className="absolute top-2 right-2">
                                <Badge variant="default" className="text-xs">
                                  Active
                                </Badge>
                              </div>
                            )}
                          </div>

                          {/* Available capes */}
                          {profile.capes.map((cape) => (
                            <div
                              key={cape.id}
                              className={cn(
                                "relative rounded-lg border-2 p-3 cursor-pointer transition-all hover:border-primary/50",
                                selectedCape === cape.id
                                  ? "border-primary bg-primary/5"
                                  : "border-muted"
                              )}
                              onClick={() => handleSetCape(cape.id)}
                              onMouseEnter={() =>
                                setHoveredCapeUrl(cape.url || null)
                              }
                              onMouseLeave={() => setHoveredCapeUrl(null)}
                            >
                              <div className="flex flex-col items-center gap-2">
                                <CapeViewer3D
                                  skinUrl={skinDataUrl}
                                  capeUrl={cape.url}
                                  variant={previewVariant}
                                  width={80}
                                  height={100}
                                />
                                <span className="text-sm font-medium text-center">
                                  {cape.alias || cape.id}
                                </span>
                              </div>
                              {selectedCape === cape.id && (
                                <div className="absolute top-2 right-2">
                                  <Badge variant="default" className="text-xs">
                                    Active
                                  </Badge>
                                </div>
                              )}
                            </div>
                          ))}
                        </div>
                      </>
                    ) : (
                      <Alert>
                        <AlertCircle className="h-4 w-4" />
                        <AlertDescription>
                          You don't have any capes available. Capes are obtained
                          through special events, purchases, or being a Minecraft
                          Realms subscriber.
                        </AlertDescription>
                      </Alert>
                    )}
                  </TabsContent>
                </ScrollArea>
              </Tabs>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
