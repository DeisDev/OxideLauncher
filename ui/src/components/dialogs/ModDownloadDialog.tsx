import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Search,
  Download,
  Plus,
  ExternalLink,
  Loader2,
  Package,
  Heart,
  Trash2,
  CheckCircle,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";

// Modrinth logo SVG
const ModrinthLogo = () => (
  <svg viewBox="0 0 512 514" className="h-4 w-4" fill="currentColor">
    <path d="M503.16 323.56C514.55 281.47 515.32 235.91 503.2 190.76C466.57 54.2299 326.04 -26.8001 189.33 9.77991C83.8101 38.0199 11.3899 128.07 0.689941 230.47H43.99C54.29 147.33 113.74 74.7298 199.75 51.7098C306.05 23.2598 415.13 80.6699 453.17 181.38L411.03 192.65C391.64 145.8 352.57 111.45 306.3 96.8198L298.56 140.66C335.09 154.13 364.72 184.5 375.56 224.91C391.36 283.8 361.94 344.14 308.56 369.17L320.09 412.16C390.25 383.21 432.4 310.3 422.43 235.14L464.41 223.91C468.91 252.62 467.35 281.16 460.55 308.07L503.16 323.56Z" />
    <path d="M321.99 504.22C185.27 540.8 44.7501 459.77 8.11011 323.24C3.84011 307.31 1.17 291.33 0 275.46H43.27C44.36 287.37 46.4699 299.35 49.6799 311.29C53.0399 323.8 57.45 335.75 62.79 347.07L101.38 323.92C98.1299 316.42 95.39 308.6 93.21 300.47C69.17 210.87 122.41 118.77 212.13 94.7601C229.13 90.2101 246.23 88.4401 262.93 89.1501L255.19 133C244.73 133.05 234.11 134.42 223.53 137.25C157.31 154.98 118.01 222.95 135.75 289.09C136.85 293.16 138.13 297.13 139.59 300.99L188.94 271.38L174.07 231.95L220.67 184.36L279.57 171.03L296.62 192.15L281.1 252.07L244.63 293.78H197.98L148.63 323.39C163.78 359.57 195.84 386.54 235.83 397.46C301.98 415.17 371.23 376.27 388.98 310.12C391.96 299.12 393.27 287.99 393.04 277L436.53 265.76C436.85 285.09 434.79 304.58 430.06 323.9C413.36 387.89 368.96 438.74 311.42 465.53L323.05 508.51C328.05 506.71 332.98 504.72 337.84 502.55L337.84 502.55C345.84 499.04 353.63 495.1 361.18 490.75L375.08 532.93C364.48 539.15 353.38 544.63 341.82 549.33L332.79 545.46L321.99 504.22Z" />
  </svg>
);

// CurseForge logo SVG
const CurseForgeLogo = () => (
  <svg viewBox="0 0 24 24" className="h-4 w-4" fill="currentColor">
    <path d="M18.326 9.2177L15.892 9.21765C17.254 10.3397 18.147 11.9859 18.326 13.8296H23.9999L18.326 9.2177ZM13.268 19.2319L13.263 19.218C12.762 18.9558 12.312 18.5997 11.94 18.1696C11.287 17.4135 10.841 16.4694 10.701 15.4313H10.7L10.6949 15.3992C10.6479 15.0633 10.6239 14.7233 10.6239 14.3812C10.6239 12.4734 11.4629 10.7632 12.7779 9.59917L12.778 9.59922C13.166 9.24919 13.604 8.95017 14.079 8.71515H8.16187V16.4754L10.7 19.0135V24H4.02899C2.27397 22.2469 1.15194 19.8888 0.979932 17.2686C0.89593 16.0206 1.03793 14.7655 1.40394 13.5835H1.40399C2.52602 9.92232 5.53608 7.09023 9.32816 6.20922L6.19909 3.08115V0H17.2461L21.073 3.82709H18.326L14.501 0.00109863L14.4999 5.58617C19.662 6.29619 23.606 10.3072 23.9999 15.431H18.326C18.036 13.5695 17.0349 11.9434 15.609 10.8653L14.8949 10.3023L14.893 10.3033C13.965 9.61425 12.8129 9.2063 11.569 9.2063C10.961 9.2063 10.374 9.3023 9.82297 9.47931H9.82302C7.54196 10.1923 5.85392 12.1413 5.47592 14.5233C5.41292 14.8873 5.37891 15.2613 5.37891 15.6443C5.37891 16.2494 5.46391 16.8344 5.62092 17.3874L5.62297 17.3944C5.65097 17.4954 5.68397 17.5964 5.71897 17.6974L5.71803 17.6964L6.15897 18.9925L6.16095 18.9965L6.52698 19.9596L6.85299 20.5677L7.30698 21.3037L7.77198 22.1988H13.268V19.2319Z" />
  </svg>
);

interface ModDownloadDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  instanceId: string;
  instanceName: string;
  minecraftVersion: string;
  modLoader: string;
  onModsInstalled: () => void;
}

interface ModSearchResult {
  id: string;
  slug: string;
  name: string;
  description: string;
  author: string;
  downloads: number;
  follows: number;
  icon_url: string | null;
  project_type: string;
  platform: string;
  categories: string[];
  date_created: string;
  date_modified: string;
}

interface ModVersion {
  id: string;
  version_number: string;
  name: string;
  game_versions: string[];
  loaders: string[];
  date_published: string;
  downloads: number;
  files: ModFile[];
  dependencies: ModDependency[];
}

interface ModFile {
  filename: string;
  url: string;
  size: number;
  primary: boolean;
}

interface ModDependency {
  project_id: string;
  dependency_type: string; // "required", "optional", "incompatible"
}

interface ModDetails {
  id: string;
  slug: string;
  name: string;
  description: string;
  body: string; // Full description in markdown/HTML
  author: string;
  downloads: number;
  follows: number;
  icon_url: string | null;
  source_url: string | null;
  issues_url: string | null;
  wiki_url: string | null;
  discord_url: string | null;
  donation_urls: { platform: string; url: string }[];
  gallery: { url: string; title: string; description: string }[];
  categories: string[];
  versions: string[];
  loaders: string[];
}

interface QueuedMod {
  id: string;
  name: string;
  version: ModVersion;
  platform: "modrinth" | "curseforge";
  iconUrl: string | null;
  dependencies: QueuedMod[];
}

type SortOption = "relevance" | "downloads" | "follows" | "newest" | "updated";

export function ModDownloadDialog({
  open,
  onOpenChange,
  instanceId,
  instanceName,
  minecraftVersion,
  modLoader,
  onModsInstalled,
}: ModDownloadDialogProps) {
  // Platform state
  const [platform, setPlatform] = useState<"modrinth" | "curseforge">("modrinth");
  
  // Search state
  const [searchQuery, setSearchQuery] = useState("");
  const [sortBy, setSortBy] = useState<SortOption>("downloads");
  const [searchResults, setSearchResults] = useState<ModSearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [hasSearched, setHasSearched] = useState(false);
  
  // Selected mod state
  const [selectedMod, setSelectedMod] = useState<ModSearchResult | null>(null);
  const [modDetails, setModDetails] = useState<ModDetails | null>(null);
  const [modVersions, setModVersions] = useState<ModVersion[]>([]);
  const [selectedVersion, setSelectedVersion] = useState<ModVersion | null>(null);
  const [isLoadingDetails, setIsLoadingDetails] = useState(false);
  
  // Queue state
  const [queue, setQueue] = useState<QueuedMod[]>([]);
  const [showReviewQueue, setShowReviewQueue] = useState(false);
  const [isInstalling, setIsInstalling] = useState(false);
  const [installProgress, setInstallProgress] = useState<string>("");
  
  // Abort dialog
  const [showAbortDialog, setShowAbortDialog] = useState(false);
  
  // Quick add loading state
  const [quickAddingMod, setQuickAddingMod] = useState<string | null>(null);

  // Load initial mods list (most downloaded)
  useEffect(() => {
    if (open && !hasSearched) {
      searchMods("");
    }
  }, [open]);
  
  // Refresh results when platform changes
  useEffect(() => {
    if (open && hasSearched) {
      searchMods(searchQuery);
    }
  }, [platform]);

  // Search mods
  const searchMods = async (query: string, newSortBy?: SortOption) => {
    setIsSearching(true);
    setHasSearched(true);
    
    const effectiveSortBy = newSortBy || sortBy;
    
    try {
      const results = await invoke<ModSearchResult[]>("search_mods_detailed", {
        query: query || "",
        minecraftVersion,
        modLoader: modLoader.toLowerCase(),
        platform,
        sortBy: effectiveSortBy,
        limit: 50,
      });
      setSearchResults(results);
    } catch (error) {
      console.error("Failed to search mods:", error);
      setSearchResults([]);
    } finally {
      setIsSearching(false);
    }
  };

  // Load mod details when a mod is selected
  useEffect(() => {
    if (selectedMod) {
      loadModDetails(selectedMod.id);
    } else {
      setModDetails(null);
      setModVersions([]);
      setSelectedVersion(null);
    }
  }, [selectedMod]);

  const loadModDetails = async (modId: string) => {
    setIsLoadingDetails(true);
    try {
      // Load mod details and versions in parallel
      const [details, versions] = await Promise.all([
        invoke<ModDetails>("get_mod_details", {
          modId,
          platform,
        }),
        invoke<ModVersion[]>("get_mod_versions", {
          modId,
          platform,
          minecraftVersion,
          modLoader: modLoader.toLowerCase(),
        }),
      ]);
      
      setModDetails(details);
      setModVersions(versions);
      
      // Auto-select the first (latest compatible) version
      if (versions.length > 0) {
        setSelectedVersion(versions[0]);
      }
    } catch (error) {
      console.error("Failed to load mod details:", error);
    } finally {
      setIsLoadingDetails(false);
    }
  };

  // Handle search
  const handleSearch = () => {
    searchMods(searchQuery);
  };

  // Handle platform change - just change the platform, useEffect will trigger refresh
  const handlePlatformChange = (newPlatform: "modrinth" | "curseforge") => {
    if (newPlatform !== platform) {
      setPlatform(newPlatform);
      setSelectedMod(null);
    }
  };

  // Handle sort change - update state and search with new sort immediately
  const handleSortChange = (newSort: SortOption) => {
    setSortBy(newSort);
    searchMods(searchQuery, newSort);
  };

  // Quick add mod to queue (from the plus button in the list)
  const quickAddToQueue = async (mod: ModSearchResult) => {
    // Check if already in queue
    if (queue.some(q => q.id === mod.id)) {
      return;
    }
    
    setQuickAddingMod(mod.id);
    
    try {
      // Get the latest compatible version
      const versions = await invoke<ModVersion[]>("get_mod_versions", {
        modId: mod.id,
        platform,
        minecraftVersion,
        modLoader: modLoader.toLowerCase(),
      });
      
      if (versions.length === 0) {
        console.error("No compatible versions found");
        return;
      }
      
      const latestVersion = versions[0];
      
      // Resolve dependencies
      const dependencies: QueuedMod[] = [];
      for (const dep of latestVersion.dependencies) {
        if (dep.dependency_type === "required") {
          try {
            const depDetails = await invoke<ModDetails>("get_mod_details", {
              modId: dep.project_id,
              platform,
            });
            const depVersions = await invoke<ModVersion[]>("get_mod_versions", {
              modId: dep.project_id,
              platform,
              minecraftVersion,
              modLoader: modLoader.toLowerCase(),
            });
            
            if (depVersions.length > 0 && !queue.some(q => q.id === dep.project_id)) {
              dependencies.push({
                id: dep.project_id,
                name: depDetails.name,
                version: depVersions[0],
                platform,
                iconUrl: depDetails.icon_url,
                dependencies: [],
              });
            }
          } catch (error) {
            console.error("Failed to resolve dependency:", error);
          }
        }
      }

      const queuedMod: QueuedMod = {
        id: mod.id,
        name: mod.name,
        version: latestVersion,
        platform,
        iconUrl: mod.icon_url,
        dependencies,
      };

      setQueue(prev => [...prev, queuedMod]);
    } catch (error) {
      console.error("Failed to add mod to queue:", error);
    } finally {
      setQuickAddingMod(null);
    }
  };

  // Add mod to queue (from the details panel)
  const addToQueue = async () => {
    if (!selectedMod || !selectedVersion) return;
    
    // Check if already in queue
    if (queue.some(q => q.id === selectedMod.id)) {
      return;
    }

    // Resolve dependencies
    const dependencies: QueuedMod[] = [];
    for (const dep of selectedVersion.dependencies) {
      if (dep.dependency_type === "required") {
        try {
          const depDetails = await invoke<ModDetails>("get_mod_details", {
            modId: dep.project_id,
            platform,
          });
          const depVersions = await invoke<ModVersion[]>("get_mod_versions", {
            modId: dep.project_id,
            platform,
            minecraftVersion,
            modLoader: modLoader.toLowerCase(),
          });
          
          if (depVersions.length > 0 && !queue.some(q => q.id === dep.project_id)) {
            dependencies.push({
              id: dep.project_id,
              name: depDetails.name,
              version: depVersions[0],
              platform,
              iconUrl: depDetails.icon_url,
              dependencies: [],
            });
          }
        } catch (error) {
          console.error("Failed to resolve dependency:", error);
        }
      }
    }

    const queuedMod: QueuedMod = {
      id: selectedMod.id,
      name: selectedMod.name,
      version: selectedVersion,
      platform,
      iconUrl: selectedMod.icon_url,
      dependencies,
    };

    setQueue(prev => [...prev, queuedMod]);
  };

  // Remove mod from queue
  const removeFromQueue = (modId: string) => {
    setQueue(prev => prev.filter(q => q.id !== modId));
  };

  // Check if mod is in queue
  const isInQueue = (modId: string) => {
    return queue.some(q => q.id === modId);
  };

  // Install all mods in queue
  const installMods = async () => {
    setIsInstalling(true);
    setInstallProgress("Preparing downloads...");
    
    try {
      // Flatten queue with dependencies
      const modsToInstall: { id: string; versionId: string; platform: string }[] = [];
      
      for (const mod of queue) {
        // Add dependencies first
        for (const dep of mod.dependencies) {
          if (!modsToInstall.some(m => m.id === dep.id)) {
            modsToInstall.push({
              id: dep.id,
              versionId: dep.version.id,
              platform: dep.platform,
            });
          }
        }
        // Add main mod
        if (!modsToInstall.some(m => m.id === mod.id)) {
          modsToInstall.push({
            id: mod.id,
            versionId: mod.version.id,
            platform: mod.platform,
          });
        }
      }

      // Download each mod
      for (let i = 0; i < modsToInstall.length; i++) {
        const mod = modsToInstall[i];
        setInstallProgress(`Downloading ${i + 1}/${modsToInstall.length}...`);
        
        await invoke("download_mod_version", {
          instanceId,
          modId: mod.id,
          versionId: mod.versionId,
          platform: mod.platform,
        });
      }

      setInstallProgress("Installation complete!");
      setQueue([]);
      setShowReviewQueue(false);
      onModsInstalled();
      
      // Close dialog after a short delay
      setTimeout(() => {
        onOpenChange(false);
      }, 1000);
    } catch (error) {
      console.error("Failed to install mods:", error);
      setInstallProgress(`Error: ${error}`);
    } finally {
      setIsInstalling(false);
    }
  };

  // Abort and clear queue
  const handleAbort = () => {
    setQueue([]);
    setShowAbortDialog(false);
    setShowReviewQueue(false);
  };

  // Format numbers
  const formatNumber = (num: number) => {
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toString();
  };

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="max-w-6xl h-[85vh] flex flex-col p-0">
          <DialogHeader className="px-6 pt-6 pb-4 border-b">
            <DialogTitle className="flex items-center gap-2">
              <Download className="h-5 w-5" />
              Download Mods for {instanceName}
            </DialogTitle>
          </DialogHeader>

          <div className="flex-1 flex overflow-hidden">
            {/* Left Sidebar - Platform Selection */}
            <div className="w-48 border-r bg-muted/30 p-4 flex flex-col gap-2">
              <span className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-2">
                Platform
              </span>
              <Button
                variant={platform === "modrinth" ? "default" : "ghost"}
                className="justify-start gap-2"
                onClick={() => handlePlatformChange("modrinth")}
              >
                <ModrinthLogo />
                Modrinth
              </Button>
              <Button
                variant={platform === "curseforge" ? "default" : "ghost"}
                className="justify-start gap-2"
                onClick={() => handlePlatformChange("curseforge")}
              >
                <CurseForgeLogo />
                CurseForge
              </Button>
              
              <Separator className="my-4" />
              
              <span className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-2">
                Instance Info
              </span>
              <div className="text-xs text-muted-foreground space-y-1">
                <p>MC: {minecraftVersion}</p>
                <p>Loader: {modLoader}</p>
              </div>
            </div>

            {/* Middle - Search and Results */}
            <div className="flex-1 flex flex-col min-w-0 border-r">
              {/* Search Bar */}
              <div className="p-4 border-b space-y-3">
                <div className="flex gap-2">
                  <div className="flex-1 relative">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                    <Input
                      placeholder="Search mods..."
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      onKeyDown={(e) => e.key === "Enter" && handleSearch()}
                      className="pl-9"
                    />
                  </div>
                  <Button onClick={handleSearch} disabled={isSearching}>
                    {isSearching ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <Search className="h-4 w-4" />
                    )}
                  </Button>
                </div>
                
                <div className="flex items-center gap-2">
                  <span className="text-xs text-muted-foreground">Sort by:</span>
                  <Select value={sortBy} onValueChange={(v) => handleSortChange(v as SortOption)}>
                    <SelectTrigger className="w-40 h-8 text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="relevance">Relevance</SelectItem>
                      <SelectItem value="downloads">Downloads</SelectItem>
                      <SelectItem value="follows">Followers</SelectItem>
                      <SelectItem value="newest">Newest</SelectItem>
                      <SelectItem value="updated">Last Updated</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              {/* Results List */}
              <ScrollArea className="flex-1">
                <div className="p-2">
                  {isSearching ? (
                    <div className="flex items-center justify-center py-12">
                      <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                    </div>
                  ) : searchResults.length === 0 ? (
                    <div className="text-center py-12 text-muted-foreground">
                      {hasSearched ? "No mods found" : "Search for mods or browse popular ones"}
                    </div>
                  ) : (
                    <div className="space-y-1">
                      {searchResults.map((mod) => (
                        <div
                          key={mod.id}
                          className={cn(
                            "flex items-start gap-3 p-3 rounded-lg cursor-pointer transition-colors",
                            selectedMod?.id === mod.id
                              ? "bg-primary/10 border border-primary/30"
                              : "hover:bg-muted/50"
                          )}
                          onClick={() => setSelectedMod(mod)}
                        >
                          {mod.icon_url ? (
                            <img
                              src={mod.icon_url}
                              alt={mod.name}
                              className="w-12 h-12 rounded-lg object-cover"
                            />
                          ) : (
                            <div className="w-12 h-12 rounded-lg bg-muted flex items-center justify-center">
                              <Package className="h-6 w-6 text-muted-foreground" />
                            </div>
                          )}
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2">
                              <span className="font-medium truncate">{mod.name}</span>
                              {isInQueue(mod.id) && (
                                <Badge variant="secondary" className="text-[10px] px-1.5 py-0">
                                  <CheckCircle className="h-3 w-3 mr-1" />
                                  Queued
                                </Badge>
                              )}
                            </div>
                            <p className="text-xs text-muted-foreground truncate">
                              by {mod.author}
                            </p>
                            <p className="text-xs text-muted-foreground line-clamp-2 mt-1">
                              {mod.description}
                            </p>
                            <div className="flex items-center gap-3 mt-2 text-xs text-muted-foreground">
                              <span className="flex items-center gap-1">
                                <Download className="h-3 w-3" />
                                {formatNumber(mod.downloads)}
                              </span>
                              <span className="flex items-center gap-1">
                                <Heart className="h-3 w-3" />
                                {formatNumber(mod.follows)}
                              </span>
                            </div>
                          </div>
                          <div className="flex items-center gap-1 shrink-0">
                            {isInQueue(mod.id) && (
                              <Button
                                size="sm"
                                variant="ghost"
                                className="h-8 w-8 p-0 text-destructive hover:text-destructive hover:bg-destructive/10"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  removeFromQueue(mod.id);
                                }}
                              >
                                <X className="h-4 w-4" />
                              </Button>
                            )}
                            <Button
                              size="sm"
                              variant={isInQueue(mod.id) ? "secondary" : "default"}
                              className="shrink-0"
                              onClick={(e) => {
                                e.stopPropagation();
                                if (!isInQueue(mod.id)) {
                                  quickAddToQueue(mod);
                                }
                              }}
                              disabled={isInQueue(mod.id) || quickAddingMod === mod.id}
                            >
                              {quickAddingMod === mod.id ? (
                                <Loader2 className="h-4 w-4 animate-spin" />
                              ) : isInQueue(mod.id) ? (
                                <CheckCircle className="h-4 w-4" />
                              ) : (
                                <Plus className="h-4 w-4" />
                              )}
                            </Button>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </ScrollArea>
            </div>

            {/* Right Panel - Mod Details */}
            <div className="w-96 flex flex-col bg-muted/20">
              {selectedMod ? (
                <>
                  {isLoadingDetails ? (
                    <div className="flex-1 flex items-center justify-center">
                      <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                    </div>
                  ) : modDetails ? (
                    <>
                      {/* Header */}
                      <div className="p-4 border-b">
                        <div className="flex items-start gap-3">
                          {modDetails.icon_url ? (
                            <img
                              src={modDetails.icon_url}
                              alt={modDetails.name}
                              className="w-16 h-16 rounded-lg object-cover"
                            />
                          ) : (
                            <div className="w-16 h-16 rounded-lg bg-muted flex items-center justify-center">
                              <Package className="h-8 w-8 text-muted-foreground" />
                            </div>
                          )}
                          <div className="flex-1 min-w-0">
                            <h3 className="font-semibold text-lg truncate">{modDetails.name}</h3>
                            <a
                              href={`https://${platform === "modrinth" ? "modrinth.com/mod" : "curseforge.com/minecraft/mc-mods"}/${modDetails.slug}`}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="text-sm text-primary hover:underline flex items-center gap-1"
                            >
                              by {modDetails.author}
                              <ExternalLink className="h-3 w-3" />
                            </a>
                          </div>
                        </div>
                        
                        <div className="flex items-center gap-4 mt-3 text-sm text-muted-foreground">
                          <span className="flex items-center gap-1">
                            <Download className="h-4 w-4" />
                            {formatNumber(modDetails.downloads)} downloads
                          </span>
                          <span className="flex items-center gap-1">
                            <Heart className="h-4 w-4" />
                            {formatNumber(modDetails.follows)} followers
                          </span>
                        </div>
                        
                        {/* External Links */}
                        <div className="flex flex-wrap gap-2 mt-3">
                          {modDetails.source_url && (
                            <a
                              href={modDetails.source_url}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                            >
                              Source <ExternalLink className="h-3 w-3" />
                            </a>
                          )}
                          {modDetails.issues_url && (
                            <a
                              href={modDetails.issues_url}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                            >
                              Issues <ExternalLink className="h-3 w-3" />
                            </a>
                          )}
                          {modDetails.wiki_url && (
                            <a
                              href={modDetails.wiki_url}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                            >
                              Wiki <ExternalLink className="h-3 w-3" />
                            </a>
                          )}
                          {modDetails.discord_url && (
                            <a
                              href={modDetails.discord_url}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                            >
                              Discord <ExternalLink className="h-3 w-3" />
                            </a>
                          )}
                        </div>
                      </div>

                      {/* Description */}
                      <ScrollArea className="flex-1 p-4">
                        <div 
                          className="prose prose-sm dark:prose-invert max-w-none"
                          dangerouslySetInnerHTML={{ __html: modDetails.body }}
                        />
                      </ScrollArea>

                      {/* Version Select and Add Button */}
                      <div className="p-4 border-t space-y-3">
                        <div>
                          <label className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                            Version
                          </label>
                          <Select
                            value={selectedVersion?.id || ""}
                            onValueChange={(v) => {
                              const version = modVersions.find(ver => ver.id === v);
                              setSelectedVersion(version || null);
                            }}
                          >
                            <SelectTrigger className="mt-1">
                              <SelectValue placeholder="Select version" />
                            </SelectTrigger>
                            <SelectContent>
                              {modVersions.map((version) => (
                                <SelectItem key={version.id} value={version.id}>
                                  <div className="flex items-center gap-2">
                                    <span>{version.version_number}</span>
                                    <span className="text-xs text-muted-foreground">
                                      ({version.game_versions.slice(0, 2).join(", ")}
                                      {version.game_versions.length > 2 ? "..." : ""})
                                    </span>
                                  </div>
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </div>
                        
                        {selectedVersion && selectedVersion.dependencies.length > 0 && (
                          <div className="text-xs text-muted-foreground">
                            <span className="font-medium">Dependencies: </span>
                            {selectedVersion.dependencies
                              .filter(d => d.dependency_type === "required")
                              .length} required
                          </div>
                        )}
                        
                        <Button
                          className="w-full"
                          onClick={addToQueue}
                          disabled={isInQueue(selectedMod.id) || !selectedVersion}
                        >
                          {isInQueue(selectedMod.id) ? (
                            <>
                              <CheckCircle className="mr-2 h-4 w-4" />
                              Added to Queue
                            </>
                          ) : (
                            <>
                              <Plus className="mr-2 h-4 w-4" />
                              Add to Queue
                            </>
                          )}
                        </Button>
                      </div>
                    </>
                  ) : (
                    <div className="flex-1 flex items-center justify-center text-muted-foreground">
                      Failed to load mod details
                    </div>
                  )}
                </>
              ) : (
                <div className="flex-1 flex items-center justify-center text-muted-foreground">
                  <div className="text-center">
                    <Package className="h-12 w-12 mx-auto mb-2 opacity-50" />
                    <p>Select a mod to view details</p>
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Footer with Queue */}
          <div className="px-6 py-4 border-t bg-muted/30 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground">
                Queue: <strong>{queue.length}</strong> mod{queue.length !== 1 ? "s" : ""}
                {queue.reduce((acc, m) => acc + m.dependencies.length, 0) > 0 && (
                  <span className="text-xs ml-1">
                    (+{queue.reduce((acc, m) => acc + m.dependencies.length, 0)} dependencies)
                  </span>
                )}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                onClick={() => onOpenChange(false)}
              >
                Cancel
              </Button>
              <Button
                onClick={() => setShowReviewQueue(true)}
                disabled={queue.length === 0}
              >
                Review and Install
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* Review Queue Dialog */}
      <Dialog open={showReviewQueue} onOpenChange={setShowReviewQueue}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <CheckCircle className="h-5 w-5" />
              Review Download Queue
            </DialogTitle>
          </DialogHeader>

          <ScrollArea className="max-h-96">
            <div className="space-y-2 p-1">
              {queue.map((mod) => (
                <div key={mod.id}>
                  <div className="flex items-center gap-3 p-3 bg-muted/50 rounded-lg">
                    {mod.iconUrl ? (
                      <img
                        src={mod.iconUrl}
                        alt={mod.name}
                        className="w-10 h-10 rounded object-cover"
                      />
                    ) : (
                      <div className="w-10 h-10 rounded bg-muted flex items-center justify-center">
                        <Package className="h-5 w-5 text-muted-foreground" />
                      </div>
                    )}
                    <div className="flex-1 min-w-0">
                      <p className="font-medium truncate">{mod.name}</p>
                      <p className="text-xs text-muted-foreground">
                        v{mod.version.version_number}
                      </p>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => removeFromQueue(mod.id)}
                      disabled={isInstalling}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                  
                  {/* Dependencies */}
                  {mod.dependencies.length > 0 && (
                    <div className="ml-6 mt-1 space-y-1">
                      {mod.dependencies.map((dep) => (
                        <div
                          key={dep.id}
                          className="flex items-center gap-2 p-2 text-sm text-muted-foreground"
                        >
                          <div className="w-6 h-6 rounded bg-muted flex items-center justify-center">
                            <Package className="h-3 w-3" />
                          </div>
                          <span className="truncate">{dep.name}</span>
                          <Badge variant="outline" className="text-[10px]">
                            dependency
                          </Badge>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </ScrollArea>

          {isInstalling && (
            <div className="flex items-center gap-2 p-3 bg-primary/10 rounded-lg">
              <Loader2 className="h-4 w-4 animate-spin" />
              <span className="text-sm">{installProgress}</span>
            </div>
          )}

          <div className="flex justify-between pt-4 border-t">
            <Button
              variant="ghost"
              onClick={() => setShowReviewQueue(false)}
              disabled={isInstalling}
            >
              Back
            </Button>
            <div className="flex gap-2">
              <Button
                variant="destructive"
                onClick={() => setShowAbortDialog(true)}
                disabled={isInstalling}
              >
                Abort
              </Button>
              <Button
                onClick={installMods}
                disabled={isInstalling || queue.length === 0}
              >
                {isInstalling ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Installing...
                  </>
                ) : (
                  <>
                    <Download className="mr-2 h-4 w-4" />
                    Install {queue.length + queue.reduce((acc, m) => acc + m.dependencies.length, 0)} mod{(queue.length + queue.reduce((acc, m) => acc + m.dependencies.length, 0)) !== 1 ? "s" : ""}
                  </>
                )}
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* Abort Confirmation Dialog */}
      <AlertDialog open={showAbortDialog} onOpenChange={setShowAbortDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Abort Installation?</AlertDialogTitle>
            <AlertDialogDescription>
              This will clear your download queue. You'll need to add the mods again if you want to install them.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleAbort}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Abort
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
