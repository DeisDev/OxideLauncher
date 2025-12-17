import { useState, useEffect, useRef } from "react";
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

// 3D-like skin preview component
function SkinPreview({
  skinUrl,
  username,
  size = 128,
}: {
  skinUrl: string | null;
  username: string;
  size?: number;
}) {
  const [imageError, setImageError] = useState(false);
  const [skinImage, setSkinImage] = useState<string | null>(null);

  useEffect(() => {
    setImageError(false);
    setSkinImage(null);

    if (skinUrl) {
      // Download skin image via backend to avoid CORS
      invoke<string>("download_skin_image", { skinUrl })
        .then((base64) => {
          setSkinImage(`data:image/png;base64,${base64}`);
        })
        .catch(() => {
          setImageError(true);
        });
    }
  }, [skinUrl]);

  if (!skinUrl || imageError) {
    // Fallback: Show a colored avatar with initial
    return (
      <div
        className="flex items-center justify-center rounded-lg bg-muted text-4xl font-bold"
        style={{ width: size, height: size }}
      >
        {username.charAt(0).toUpperCase()}
      </div>
    );
  }

  if (!skinImage) {
    return (
      <div
        className="flex items-center justify-center rounded-lg bg-muted"
        style={{ width: size, height: size }}
      >
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  // Render skin face (8x8 pixels at position 8,8)
  return (
    <div
      className="rounded-lg bg-muted overflow-hidden relative"
      style={{ width: size, height: size }}
    >
      <canvas
        ref={(canvas) => {
          if (canvas && skinImage) {
            const ctx = canvas.getContext("2d");
            if (ctx) {
              const img = new Image();
              img.onload = () => {
                // Clear canvas
                ctx.imageSmoothingEnabled = false;
                ctx.clearRect(0, 0, canvas.width, canvas.height);

                // Draw the face (8x8 at position 8,8) scaled to fill
                ctx.drawImage(
                  img,
                  8, 8, 8, 8, // Source: face position and size
                  0, 0, canvas.width, canvas.height // Dest: fill canvas
                );

                // Draw the hat/overlay layer (8x8 at position 40,8)
                ctx.drawImage(
                  img,
                  40, 8, 8, 8, // Source: hat overlay position and size
                  0, 0, canvas.width, canvas.height // Dest: fill canvas
                );
              };
              img.src = skinImage;
            }
          }
        }}
        width={size}
        height={size}
        style={{ imageRendering: "pixelated" }}
      />
    </div>
  );
}

// Full body skin preview (front view)
function SkinBodyPreview({
  skinUrl,
  variant,
  username,
}: {
  skinUrl: string | null;
  variant: string;
  username: string;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [skinImage, setSkinImage] = useState<string | null>(null);
  const [imageError, setImageError] = useState(false);

  useEffect(() => {
    setImageError(false);
    setSkinImage(null);

    if (skinUrl) {
      invoke<string>("download_skin_image", { skinUrl })
        .then((base64) => {
          setSkinImage(`data:image/png;base64,${base64}`);
        })
        .catch(() => {
          setImageError(true);
        });
    }
  }, [skinUrl]);

  useEffect(() => {
    if (!canvasRef.current || !skinImage) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const img = new Image();
    img.onload = () => {
      ctx.imageSmoothingEnabled = false;
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      const scale = 8;

      // Draw body parts (front view)
      // Head (8x8 at 8,8) - position at top center
      ctx.drawImage(img, 8, 8, 8, 8, 4 * scale, 0, 8 * scale, 8 * scale);
      // Head overlay
      ctx.drawImage(img, 40, 8, 8, 8, 4 * scale, 0, 8 * scale, 8 * scale);

      // Body (8x12 at 20,20)
      ctx.drawImage(img, 20, 20, 8, 12, 4 * scale, 8 * scale, 8 * scale, 12 * scale);
      // Body overlay
      ctx.drawImage(img, 20, 36, 8, 12, 4 * scale, 8 * scale, 8 * scale, 12 * scale);

      // Arms - width depends on variant (slim = 3, classic = 4)
      const armWidth = variant === "slim" ? 3 : 4;

      // Right arm
      ctx.drawImage(img, 44, 20, armWidth, 12, (4 - armWidth) * scale, 8 * scale, armWidth * scale, 12 * scale);
      // Right arm overlay
      ctx.drawImage(img, 44, 36, armWidth, 12, (4 - armWidth) * scale, 8 * scale, armWidth * scale, 12 * scale);

      // Left arm
      ctx.drawImage(img, 36, 52, armWidth, 12, 12 * scale, 8 * scale, armWidth * scale, 12 * scale);
      // Left arm overlay
      ctx.drawImage(img, 52, 52, armWidth, 12, 12 * scale, 8 * scale, armWidth * scale, 12 * scale);

      // Legs (4x12 each)
      // Right leg
      ctx.drawImage(img, 4, 20, 4, 12, 4 * scale, 20 * scale, 4 * scale, 12 * scale);
      ctx.drawImage(img, 4, 36, 4, 12, 4 * scale, 20 * scale, 4 * scale, 12 * scale);

      // Left leg
      ctx.drawImage(img, 20, 52, 4, 12, 8 * scale, 20 * scale, 4 * scale, 12 * scale);
      ctx.drawImage(img, 4, 52, 4, 12, 8 * scale, 20 * scale, 4 * scale, 12 * scale);
    };
    img.src = skinImage;
  }, [skinImage, variant]);

  if (!skinUrl || imageError) {
    return (
      <div className="flex items-center justify-center w-32 h-64 rounded-lg bg-muted text-4xl font-bold">
        {username.charAt(0).toUpperCase()}
      </div>
    );
  }

  if (!skinImage) {
    return (
      <div className="flex items-center justify-center w-32 h-64 rounded-lg bg-muted">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <canvas
      ref={canvasRef}
      width={128}
      height={256}
      className="rounded-lg bg-muted"
      style={{ imageRendering: "pixelated" }}
    />
  );
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

  // Form state
  const [selectedVariant, setSelectedVariant] = useState<"classic" | "slim">("classic");
  const [skinUrl, setSkinUrl] = useState("");
  const [importUsername, setImportUsername] = useState("");
  const [importedSkin, setImportedSkin] = useState<FetchedSkinResponse | null>(null);
  const [loadingImport, setLoadingImport] = useState(false);
  const [selectedCape, setSelectedCape] = useState<string | null>(null);

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
      setSelectedVariant(profile.active_skin.variant as "classic" | "slim");
    }
  }, [profile]);

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

      // Read file as bytes
      const response = await fetch(`file://${selected}`);
      const blob = await response.blob();
      const arrayBuffer = await blob.arrayBuffer();
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
  };

  const handleImportFromUsername = async () => {
    if (!importUsername.trim()) return;

    setLoadingImport(true);
    setError(null);
    setImportedSkin(null);

    try {
      const skin = await invoke<FetchedSkinResponse>("fetch_skin_from_username", {
        username: importUsername.trim(),
      });
      setImportedSkin(skin);
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

  // Offline account message
  if (account.account_type === "Offline") {
    return (
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Skin Management</DialogTitle>
            <DialogDescription>
              Manage your Minecraft skin
            </DialogDescription>
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
      <DialogContent className="max-w-3xl max-h-[90vh]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <User className="h-5 w-5" />
            Skin Management - {account.username}
          </DialogTitle>
          <DialogDescription>
            Change your skin, model type, or cape
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
          <div className="grid grid-cols-[200px_1fr] gap-6">
            {/* Left side - Current skin preview */}
            <div className="flex flex-col items-center gap-4">
              <div className="text-sm font-medium text-muted-foreground">
                Current Skin
              </div>
              <SkinBodyPreview
                skinUrl={profile?.active_skin?.url || null}
                variant={profile?.active_skin?.variant || "classic"}
                username={account.username}
              />
              <Badge variant="secondary">
                {profile?.active_skin?.variant || "classic"}
              </Badge>

              <div className="flex gap-2 mt-4">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={loadProfile}
                  disabled={loading}
                >
                  <RefreshCw className="h-4 w-4" />
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleOpenSkinsFolder}
                  title="Open skins folder"
                >
                  <FolderOpen className="h-4 w-4" />
                </Button>
              </div>

              <Button
                variant="link"
                size="sm"
                onClick={handleOpenMinecraftSkins}
                className="text-xs"
              >
                Browse MinecraftSkins.com
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

                <ScrollArea className="h-[350px] mt-4 pr-4">
                  {/* Upload Tab */}
                  <TabsContent value="upload" className="space-y-4">
                    <div>
                      <Label className="text-base font-semibold">
                        Model Type
                      </Label>
                      <p className="text-sm text-muted-foreground mb-3">
                        Select your skin model type
                      </p>
                      <RadioGroup
                        value={selectedVariant}
                        onValueChange={(v) =>
                          setSelectedVariant(v as "classic" | "slim")
                        }
                        className="flex gap-4"
                      >
                        <div className="flex items-center space-x-2">
                          <RadioGroupItem value="classic" id="classic" />
                          <Label htmlFor="classic" className="cursor-pointer">
                            Classic (Steve)
                          </Label>
                        </div>
                        <div className="flex items-center space-x-2">
                          <RadioGroupItem value="slim" id="slim" />
                          <Label htmlFor="slim" className="cursor-pointer">
                            Slim (Alex)
                          </Label>
                        </div>
                      </RadioGroup>
                    </div>

                    <Separator />

                    <div>
                      <Label className="text-base font-semibold">
                        Upload Skin File
                      </Label>
                      <p className="text-sm text-muted-foreground mb-3">
                        Select a PNG image (64x64 or 64x32)
                      </p>
                      <Button
                        onClick={handleUploadSkin}
                        disabled={saving}
                        className="w-full"
                      >
                        {saving ? (
                          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        ) : (
                          <Upload className="mr-2 h-4 w-4" />
                        )}
                        Choose File & Upload
                      </Button>
                    </div>

                    <Separator />

                    <div>
                      <Label className="text-base font-semibold text-destructive">
                        Reset Skin
                      </Label>
                      <p className="text-sm text-muted-foreground mb-3">
                        Reset your skin to the default Steve/Alex skin
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
                  <TabsContent value="url" className="space-y-4">
                    <div>
                      <Label className="text-base font-semibold">
                        Model Type
                      </Label>
                      <p className="text-sm text-muted-foreground mb-3">
                        Select your skin model type
                      </p>
                      <RadioGroup
                        value={selectedVariant}
                        onValueChange={(v) =>
                          setSelectedVariant(v as "classic" | "slim")
                        }
                        className="flex gap-4"
                      >
                        <div className="flex items-center space-x-2">
                          <RadioGroupItem value="classic" id="url-classic" />
                          <Label
                            htmlFor="url-classic"
                            className="cursor-pointer"
                          >
                            Classic (Steve)
                          </Label>
                        </div>
                        <div className="flex items-center space-x-2">
                          <RadioGroupItem value="slim" id="url-slim" />
                          <Label htmlFor="url-slim" className="cursor-pointer">
                            Slim (Alex)
                          </Label>
                        </div>
                      </RadioGroup>
                    </div>

                    <Separator />

                    <div>
                      <Label className="text-base font-semibold">
                        Skin URL
                      </Label>
                      <p className="text-sm text-muted-foreground mb-3">
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
                            <Link className="h-4 w-4" />
                          )}
                        </Button>
                      </div>
                    </div>
                  </TabsContent>

                  {/* Import Tab */}
                  <TabsContent value="import" className="space-y-4">
                    <div>
                      <Label className="text-base font-semibold">
                        Import from Player
                      </Label>
                      <p className="text-sm text-muted-foreground mb-3">
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
                      <>
                        <Separator />

                        <div className="flex gap-4">
                          <div className="flex flex-col items-center gap-2">
                            <SkinPreview
                              skinUrl={importedSkin.skin_url}
                              username={importedSkin.username}
                              size={96}
                            />
                            <div className="text-sm font-medium">
                              {importedSkin.username}
                            </div>
                            <Badge variant="secondary">
                              {importedSkin.skin_variant}
                            </Badge>
                          </div>

                          <div className="flex-1 space-y-4">
                            <div>
                              <Label className="text-base font-semibold">
                                Apply as Model Type
                              </Label>
                              <RadioGroup
                                value={selectedVariant}
                                onValueChange={(v) =>
                                  setSelectedVariant(v as "classic" | "slim")
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
                                    className="cursor-pointer"
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
                                    className="cursor-pointer"
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
                              <p className="text-sm text-muted-foreground">
                                This player doesn't have a custom skin.
                              </p>
                            )}
                          </div>
                        </div>
                      </>
                    )}
                  </TabsContent>

                  {/* Capes Tab */}
                  <TabsContent value="cape" className="space-y-4">
                    <div>
                      <Label className="text-base font-semibold">
                        Your Capes
                      </Label>
                      <p className="text-sm text-muted-foreground mb-3">
                        Select a cape to wear, or hide your cape
                      </p>

                      {profile?.capes && profile.capes.length > 0 ? (
                        <div className="grid grid-cols-3 gap-4">
                          {/* No cape option */}
                          <button
                            onClick={() => handleSetCape(null)}
                            disabled={saving}
                            className={`flex flex-col items-center p-4 rounded-lg border-2 transition-colors ${
                              selectedCape === null
                                ? "border-primary bg-primary/10"
                                : "border-border hover:border-muted-foreground"
                            }`}
                          >
                            <div className="w-12 h-16 bg-muted rounded flex items-center justify-center text-muted-foreground">
                              ✕
                            </div>
                            <span className="text-sm mt-2">No Cape</span>
                          </button>

                          {/* Cape options */}
                          {profile.capes.map((cape) => (
                            <button
                              key={cape.id}
                              onClick={() => handleSetCape(cape.id)}
                              disabled={saving}
                              className={`flex flex-col items-center p-4 rounded-lg border-2 transition-colors ${
                                selectedCape === cape.id
                                  ? "border-primary bg-primary/10"
                                  : "border-border hover:border-muted-foreground"
                              }`}
                            >
                              <img
                                src={cape.url}
                                alt={cape.alias || "Cape"}
                                className="w-12 h-16 object-contain"
                                style={{ imageRendering: "pixelated" }}
                              />
                              <span className="text-sm mt-2 truncate max-w-full">
                                {cape.alias || "Cape"}
                              </span>
                              {cape.is_active && (
                                <Badge variant="default" className="mt-1">
                                  Active
                                </Badge>
                              )}
                            </button>
                          ))}
                        </div>
                      ) : (
                        <Alert>
                          <AlertDescription>
                            You don't have any capes. Capes can be obtained from
                            Minecraft events, Realms, or marketplace purchases.
                          </AlertDescription>
                        </Alert>
                      )}
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
