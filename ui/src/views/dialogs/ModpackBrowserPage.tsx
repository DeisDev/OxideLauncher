import { useState, useEffect, useMemo, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open as openExternal } from "@tauri-apps/plugin-shell";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { emit, listen, UnlistenFn } from "@tauri-apps/api/event";
import {
  Search,
  Download,
  Loader2,
  Package,
  Heart,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  ExternalLink,
  Filter,
  AlertCircle,
  CheckCircle,
} from "lucide-react";
import ReactMarkdown, { Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";
import { DialogWindowHeader } from "@/components/common/DialogWindowHeader";
import { BlockedModsDialog, BlockedMod } from "@/components/dialogs/BlockedModsDialog";

// Import custom platform logos
import modrinthLogo from "../../../art/modrinth.svg";
import curseforgeLogo from "../../../art/flame.svg";

// Modrinth logo component
const ModrinthLogo = ({ className = "h-4 w-4" }: { className?: string }) => (
  <img src={modrinthLogo} alt="Modrinth" className={className} />
);

// CurseForge logo component
const CurseForgeLogo = ({ className = "h-4 w-4" }: { className?: string }) => (
  <img src={curseforgeLogo} alt="CurseForge" className={className} />
);

interface ModpackSearchResult {
  id: string;
  slug: string;
  name: string;
  description: string;
  author: string;
  downloads: number;
  follows: number;
  icon_url: string | null;
  categories: string[];
  versions: string[];
  loaders: string[];
  date_created: string;
  date_modified: string;
  platform: string;
}

interface ModpackVersion {
  id: string;
  project_id: string;
  name: string;
  version_number: string;
  changelog: string | null;
  game_versions: string[];
  loaders: string[];
  download_url: string | null;
  filename: string;
  size: number;
  downloads: number;
  date_published: string;
  version_type: string;
  platform: string;
}

interface ModpackDetails {
  id: string;
  slug: string;
  name: string;
  description: string;
  body: string;
  author: string;
  icon_url: string | null;
  downloads: number;
  followers: number;
  categories: string[];
  versions: string[];
  loaders: string[];
  website_url: string | null;
  source_url: string | null;
  issues_url: string | null;
  wiki_url: string | null;
  discord_url: string | null;
  date_created: string;
  date_modified: string;
  platform: string;
}

interface ModpackSearchResponse {
  modpacks: ModpackSearchResult[];
  total_hits: number;
  offset: number;
  limit: number;
}

// Import result from backend
interface ImportResultInfo {
  instance_id: string;
  name: string;
  minecraft_version: string;
  mod_loader_type: string | null;
  mod_loader_version: string | null;
  files_to_download: number;
  warnings: string[];
  blocked_files: BlockedFileInfo[];
}

// Simple blocked file info from import result
interface BlockedFileInfo {
  project_id: string;
  file_id: string;
  filename: string;
}

// Download progress event from backend
interface ModpackDownloadProgress {
  downloaded: number;
  total: number;
  bytes_downloaded: number;
  speed_bps: number;
  current_file: string | null;
  phase?: "preparing" | "resolving" | "downloading";
}

type SortOption = "relevance" | "downloads" | "follows" | "newest" | "updated";

// Page size options - CurseForge API max is 50, Modrinth allows 100
const PAGE_SIZE_OPTIONS_MODRINTH = [10, 25, 50, 100] as const;
const PAGE_SIZE_OPTIONS_CURSEFORGE = [10, 25, 50] as const;

const getPageSizeOptions = (platform: "modrinth" | "curseforge") =>
  platform === "curseforge" ? PAGE_SIZE_OPTIONS_CURSEFORGE : PAGE_SIZE_OPTIONS_MODRINTH;

const getMaxPageSize = (platform: "modrinth" | "curseforge") =>
  platform === "curseforge" ? 50 : 100;

// Minecraft version options
const MC_VERSIONS = [
  "1.21.4", "1.21.3", "1.21.1", "1.21", "1.20.6", "1.20.4", "1.20.1",
  "1.19.4", "1.19.2", "1.18.2", "1.16.5", "1.12.2", "1.7.10"
];

// Mod loader options
const MOD_LOADERS = ["forge", "fabric", "quilt", "neoforge"];

export function ModpackBrowserPage() {
  // Platform state
  const [platform, setPlatform] = useState<"modrinth" | "curseforge">("modrinth");

  // Search state
  const [searchQuery, setSearchQuery] = useState("");
  const [sortBy, setSortBy] = useState<SortOption>("downloads");
  const [searchResults, setSearchResults] = useState<ModpackSearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [hasSearched, setHasSearched] = useState(false);

  // Filter state
  const [minecraftVersion, setMinecraftVersion] = useState<string>("");
  const [modLoader, setModLoader] = useState<string>("");
  const [selectedCategories, setSelectedCategories] = useState<string[]>([]);
  const [availableCategories, setAvailableCategories] = useState<string[]>([]);
  const [showCategoryFilter, setShowCategoryFilter] = useState(false);

  // Pagination state
  const [currentPage, setCurrentPage] = useState(1);
  const [totalHits, setTotalHits] = useState(0);
  const [pageSize, setPageSize] = useState<number>(25);

  // Selected modpack state
  const [selectedModpack, setSelectedModpack] = useState<ModpackSearchResult | null>(null);
  const [modpackDetails, setModpackDetails] = useState<ModpackDetails | null>(null);
  const [modpackVersions, setModpackVersions] = useState<ModpackVersion[]>([]);
  const [selectedVersion, setSelectedVersion] = useState<ModpackVersion | null>(null);
  const [isLoadingDetails, setIsLoadingDetails] = useState(false);

  // Installation state
  const [isInstalling, setIsInstalling] = useState(false);
  const [installProgress, setInstallProgress] = useState<string>("");
  const [installError, setInstallError] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<ModpackDownloadProgress | null>(null);

  // Blocked mods dialog state
  const [showBlockedModsDialog, setShowBlockedModsDialog] = useState(false);
  const [currentBlockedMods, setCurrentBlockedMods] = useState<BlockedMod[]>([]);
  const [currentInstanceId, setCurrentInstanceId] = useState<string>("");

  // Instance name for installation
  const [instanceName, setInstanceName] = useState("");

  // Custom markdown components to open links externally
  const markdownComponents: Components = useMemo(() => ({
    a: ({ href, children, ...props }: React.AnchorHTMLAttributes<HTMLAnchorElement> & { children?: React.ReactNode }) => (
      <a
        {...props}
        href={href}
        onClick={(e) => {
          e.preventDefault();
          if (href) {
            openExternal(href);
          }
        }}
        className="text-primary hover:underline cursor-pointer"
      >
        {children}
      </a>
    ),
  }), []);

  // Calculate total pages
  const totalPages = Math.ceil(totalHits / pageSize);

  // Format numbers
  const formatNumber = (num: number) => {
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toString();
  };

  // Format file size
  const formatFileSize = (bytes: number): string => {
    if (bytes >= 1073741824) return `${(bytes / 1073741824).toFixed(2)} GB`;
    if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(2)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(2)} KB`;
    return `${bytes} B`;
  };

  // Format download speed
  const formatDownloadSpeed = (bytesPerSecond: number): string => {
    if (bytesPerSecond >= 1073741824) return `${(bytesPerSecond / 1073741824).toFixed(2)} GB/s`;
    if (bytesPerSecond >= 1048576) return `${(bytesPerSecond / 1048576).toFixed(2)} MB/s`;
    if (bytesPerSecond >= 1024) return `${(bytesPerSecond / 1024).toFixed(2)} KB/s`;
    return `${bytesPerSecond} B/s`;
  };

  // Load initial modpacks list
  useEffect(() => {
    if (!hasSearched) {
      searchModpacks("");
    }
  }, []);

  // Listen for download progress events
  useEffect(() => {
    let unlisten: UnlistenFn | null = null;
    
    listen<ModpackDownloadProgress>("modpack-download-progress", (event) => {
      setDownloadProgress(event.payload);
      // Update text progress based on phase
      const { phase, downloaded, total, current_file } = event.payload;
      if (phase === "preparing") {
        setInstallProgress("Preparing download...");
      } else if (phase === "resolving") {
        setInstallProgress(current_file || "Resolving download URLs...");
      } else if (total > 0) {
        setInstallProgress(`Downloading mods... ${downloaded}/${total}`);
      }
    }).then((fn) => {
      unlisten = fn;
    });
    
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  // Refresh results when platform changes
  useEffect(() => {
    if (hasSearched) {
      searchModpacks(searchQuery, sortBy, selectedCategories, 1);
    }
  }, [platform]);

  // Search modpacks
  const searchModpacks = async (
    query: string,
    newSortBy?: SortOption,
    categories?: string[],
    page?: number,
    newPageSize?: number
  ) => {
    setIsSearching(true);
    setHasSearched(true);

    const effectiveSortBy = newSortBy || sortBy;
    const effectiveCategories = categories ?? selectedCategories;
    const effectivePage = page ?? currentPage;
    const effectivePageSize = newPageSize ?? pageSize;
    const offset = (effectivePage - 1) * effectivePageSize;

    try {
      const response = await invoke<ModpackSearchResponse>("search_modpacks", {
        query: query || "",
        platform,
        minecraftVersion: minecraftVersion || null,
        modLoader: modLoader || null,
        sort: effectiveSortBy,
        limit: effectivePageSize,
        offset,
        categories: effectiveCategories.length > 0 ? effectiveCategories : null,
      });

      // Extract all unique categories from results
      const allCategories = new Set<string>();
      response.modpacks.forEach(modpack => {
        modpack.categories.forEach(cat => allCategories.add(cat));
      });
      setAvailableCategories(Array.from(allCategories).sort());

      setSearchResults(response.modpacks);
      setTotalHits(response.total_hits);
      setCurrentPage(effectivePage);
    } catch (error) {
      console.error("Failed to search modpacks:", error);
      setSearchResults([]);
      setTotalHits(0);
    } finally {
      setIsSearching(false);
    }
  };

  // Handle page change
  const handlePageChange = (newPage: number) => {
    if (newPage >= 1 && newPage <= totalPages && newPage !== currentPage) {
      setCurrentPage(newPage);
      searchModpacks(searchQuery, sortBy, selectedCategories, newPage);
    }
  };

  // Handle page size change
  const handlePageSizeChange = (newSize: number) => {
    setPageSize(newSize);
    setCurrentPage(1);
    searchModpacks(searchQuery, sortBy, selectedCategories, 1, newSize);
  };

  // Load modpack details when selected
  useEffect(() => {
    if (selectedModpack) {
      loadModpackDetails(selectedModpack.id);
      setInstanceName(selectedModpack.name);
    } else {
      setModpackDetails(null);
      setModpackVersions([]);
      setSelectedVersion(null);
      setInstanceName("");
    }
  }, [selectedModpack]);

  const loadModpackDetails = async (modpackId: string) => {
    setIsLoadingDetails(true);
    try {
      const [details, versions] = await Promise.all([
        invoke<ModpackDetails>("get_modpack_details", {
          modpackId,
          platform,
        }),
        invoke<ModpackVersion[]>("get_modpack_versions", {
          modpackId,
          platform,
          minecraftVersion: minecraftVersion || null,
          modLoader: modLoader || null,
        }),
      ]);

      // If author not in details, use from search result
      if (!details.author && selectedModpack) {
        details.author = selectedModpack.author;
      }

      setModpackDetails(details);
      setModpackVersions(versions);

      // Auto-select the first (latest compatible) version
      if (versions.length > 0) {
        setSelectedVersion(versions[0]);
      }
    } catch (error) {
      console.error("Failed to load modpack details:", error);
    } finally {
      setIsLoadingDetails(false);
    }
  };

  // Handle search
  const handleSearch = () => {
    setCurrentPage(1);
    searchModpacks(searchQuery, sortBy, selectedCategories, 1);
  };

  // Handle platform change
  const handlePlatformChange = (newPlatform: "modrinth" | "curseforge") => {
    if (newPlatform !== platform) {
      setPlatform(newPlatform);
      setSelectedModpack(null);
      setSelectedCategories([]);
      setCurrentPage(1);
      // Adjust page size if it exceeds the new platform's max
      const maxSize = getMaxPageSize(newPlatform);
      if (pageSize > maxSize) {
        setPageSize(maxSize);
      }
    }
  };

  // Handle sort change
  const handleSortChange = (newSort: SortOption) => {
    setSortBy(newSort);
    setCurrentPage(1);
    searchModpacks(searchQuery, newSort, selectedCategories, 1);
  };

  // Handle filter changes
  const handleMinecraftVersionChange = (version: string) => {
    setMinecraftVersion(version === "__all__" ? "" : version);
  };

  const handleModLoaderChange = (loader: string) => {
    setModLoader(loader === "__all__" ? "" : loader);
  };

  // Toggle category filter
  const toggleCategory = (category: string) => {
    const newCategories = selectedCategories.includes(category)
      ? selectedCategories.filter(c => c !== category)
      : [...selectedCategories, category];
    setSelectedCategories(newCategories);
    setCurrentPage(1);
    searchModpacks(searchQuery, sortBy, newCategories, 1);
  };

  // Clear all category filters
  const clearCategoryFilters = () => {
    setSelectedCategories([]);
    setCurrentPage(1);
    searchModpacks(searchQuery, sortBy, [], 1);
  };

  // Install selected modpack directly
  const installModpack = async () => {
    if (!selectedModpack || !selectedVersion) return;
    
    if (!selectedVersion.download_url) {
      setInstallError(`No download URL available for ${selectedModpack.name}`);
      return;
    }

    setIsInstalling(true);
    setInstallError(null);
    setInstallProgress(`Installing ${selectedModpack.name}...`);
    setDownloadProgress(null);

    try {
      const result = await invoke<ImportResultInfo>("import_instance_from_url", {
        url: selectedVersion.download_url,
        nameOverride: instanceName || selectedModpack.name,
        iconUrl: modpackDetails?.icon_url || selectedModpack.icon_url || null,
      });

      // Check if there are blocked files that need manual download
      if (result.blocked_files && result.blocked_files.length > 0) {
        setInstallProgress(`Resolving blocked mod info...`);
        
        // Resolve blocked files to full BlockedMod objects
        const blockedMods = await invoke<BlockedMod[]>("resolve_blocked_files", {
          blockedFiles: result.blocked_files,
          targetFolder: "mods",
        });

        if (blockedMods.length > 0) {
          // Store instance id and show blocked mods dialog
          setCurrentInstanceId(result.instance_id);
          setCurrentBlockedMods(blockedMods);
          setShowBlockedModsDialog(true);
          setIsInstalling(false);
          setInstallProgress("");
          setDownloadProgress(null);
          return;
        }
      }

      // Installation complete - close window
      await finishInstallation(result.instance_id);
    } catch (error) {
      console.error("Failed to install modpack:", error);
      setInstallError(`${error}`);
      setIsInstalling(false);
      setInstallProgress("");
      setDownloadProgress(null);
    }
  };

  // Handle blocked mods dialog continue (user downloaded files)
  const handleBlockedModsContinue = async () => {
    setShowBlockedModsDialog(false);
    
    try {
      // Copy matched blocked mods to the instance
      await invoke("copy_blocked_mods_to_instance", {
        blockedMods: currentBlockedMods.filter(m => m.matched),
        instanceId: currentInstanceId,
      });
    } catch (error) {
      console.error("Failed to copy blocked mods:", error);
    }

    await finishInstallation(currentInstanceId);
  };

  // Handle blocked mods dialog skip (user wants to skip missing mods)
  const handleBlockedModsSkip = async () => {
    setShowBlockedModsDialog(false);

    // Copy any matched mods that were found
    try {
      const matchedMods = currentBlockedMods.filter(m => m.matched);
      if (matchedMods.length > 0) {
        await invoke("copy_blocked_mods_to_instance", {
          blockedMods: matchedMods,
          instanceId: currentInstanceId,
        });
      }
    } catch (error) {
      console.error("Failed to copy matched blocked mods:", error);
    }

    await finishInstallation(currentInstanceId);
  };

  // Finish installation and close window
  const finishInstallation = async (instanceId?: string) => {
    setInstallProgress("Installation complete!");
    setIsInstalling(false);
    setDownloadProgress(null);

    // Check config to see if we should open the instance details
    try {
      const config = await invoke<{ ui: { open_instance_after_install: boolean } }>("get_config");
      if (config.ui.open_instance_after_install && instanceId) {
        // Emit event to navigate to instance details
        await emit("instances-changed", {});
        await emit("navigate-to-instance", { instanceId });
        await emit("dialog-closed", {});
      } else {
        // Emit events to notify main window to refresh instances and navigate to instances view
        await emit("instances-changed", {});
        await emit("navigate-to-instances", {});
        await emit("dialog-closed", {});
      }
    } catch {
      // Default to instances view
      await emit("instances-changed", {});
      await emit("navigate-to-instances", {});
      await emit("dialog-closed", {});
    }

    // Close this window after a short delay
    setTimeout(async () => {
      try {
        const currentWindow = getCurrentWebviewWindow();
        await currentWindow.close();
      } catch {
        // Ignore errors
      }
    }, 500);
  };

  // Close this window
  const handleClose = useCallback(async () => {
    try {
      await emit("dialog-closed", {});
      const currentWindow = getCurrentWebviewWindow();
      await currentWindow.close();
    } catch {
      // Ignore errors
    }
  }, []);

  return (
    <div className="flex flex-col h-screen bg-background text-foreground">
      {/* Header */}
      <DialogWindowHeader 
        title="Browse Modpacks" 
        icon={<Package className="h-5 w-5" />}
      />

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
            Filters
          </span>
          <div className="space-y-2">
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">MC Version</label>
              <Select value={minecraftVersion || "__all__"} onValueChange={handleMinecraftVersionChange}>
                <SelectTrigger className="h-8 text-xs">
                  <SelectValue placeholder="All" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="__all__">All Versions</SelectItem>
                  {MC_VERSIONS.map((v) => (
                    <SelectItem key={v} value={v}>{v}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">Mod Loader</label>
              <Select value={modLoader || "__all__"} onValueChange={handleModLoaderChange}>
                <SelectTrigger className="h-8 text-xs">
                  <SelectValue placeholder="All" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="__all__">All Loaders</SelectItem>
                  {MOD_LOADERS.map((l) => (
                    <SelectItem key={l} value={l} className="capitalize">{l}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <Button
              variant="secondary"
              size="sm"
              className="w-full mt-2 h-7 text-xs"
              onClick={() => searchModpacks(searchQuery, sortBy, selectedCategories, 1)}
            >
              Apply Filters
            </Button>
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
                  placeholder="Search modpacks..."
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

              <Button
                variant={showCategoryFilter ? "secondary" : "outline"}
                size="sm"
                className="h-8 text-xs gap-1"
                onClick={() => setShowCategoryFilter(!showCategoryFilter)}
              >
                <Filter className="h-3 w-3" />
                Categories
                {selectedCategories.length > 0 && (
                  <Badge variant="secondary" className="ml-1 h-4 px-1 text-[10px]">
                    {selectedCategories.length}
                  </Badge>
                )}
              </Button>
            </div>

            {/* Category Filter Panel */}
            {showCategoryFilter && availableCategories.length > 0 && (
              <div className="border rounded-lg p-3 bg-muted/30">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-xs font-medium text-muted-foreground uppercase">Categories</span>
                  {selectedCategories.length > 0 && (
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-6 text-xs px-2"
                      onClick={clearCategoryFilters}
                    >
                      Clear all
                    </Button>
                  )}
                </div>
                <div className="flex flex-wrap gap-1.5">
                  {availableCategories.map((category) => (
                    <Badge
                      key={category}
                      variant={selectedCategories.includes(category) ? "default" : "outline"}
                      className="cursor-pointer text-xs capitalize"
                      onClick={() => toggleCategory(category)}
                    >
                      {category.replace(/-/g, " ")}
                    </Badge>
                  ))}
                </div>
              </div>
            )}
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
                  {hasSearched ? "No modpacks found" : "Search for modpacks or browse popular ones"}
                </div>
              ) : (
                <div className="space-y-1">
                  {searchResults.map((modpack) => (
                    <div
                      key={modpack.id}
                      className={cn(
                        "flex items-start gap-3 p-3 rounded-lg cursor-pointer transition-colors",
                        selectedModpack?.id === modpack.id
                          ? "bg-primary/10 border border-primary/30"
                          : "hover:bg-muted/50"
                      )}
                      onClick={() => setSelectedModpack(modpack)}
                    >
                      {modpack.icon_url ? (
                        <img
                          src={modpack.icon_url}
                          alt={modpack.name}
                          className="w-12 h-12 rounded-lg object-cover"
                        />
                      ) : (
                        <div className="w-12 h-12 rounded-lg bg-muted flex items-center justify-center">
                          <Package className="h-6 w-6 text-muted-foreground" />
                        </div>
                      )}
                      <div className="flex-1 min-w-0">
                        <span className="font-medium truncate">{modpack.name}</span>
                        <p className="text-xs text-muted-foreground truncate">
                          by {modpack.author || "Unknown"}
                        </p>
                        <p className="text-xs text-muted-foreground line-clamp-2 mt-1">
                          {modpack.description}
                        </p>
                        <div className="flex items-center gap-3 mt-2 text-xs text-muted-foreground">
                          <span className="flex items-center gap-1">
                            <Download className="h-3 w-3" />
                            {formatNumber(modpack.downloads)}
                          </span>
                          {modpack.follows > 0 && (
                            <span className="flex items-center gap-1">
                              <Heart className="h-3 w-3" />
                              {formatNumber(modpack.follows)}
                            </span>
                          )}
                          {modpack.loaders.length > 0 && (
                            <span className="capitalize">{modpack.loaders.slice(0, 2).join(", ")}</span>
                          )}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </ScrollArea>

          {/* Pagination Controls */}
          {totalHits > 0 && (
            <div className="p-3 border-t flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span className="text-xs text-muted-foreground">
                  {totalHits.toLocaleString()} {totalHits === 1 ? "result" : "results"}
                </span>
                <Select
                  value={pageSize.toString()}
                  onValueChange={(value) => handlePageSizeChange(Number(value))}
                >
                  <SelectTrigger className="h-7 w-[70px] text-xs">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {getPageSizeOptions(platform).map((size) => (
                      <SelectItem key={size} value={size.toString()}>
                        {size}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <span className="text-xs text-muted-foreground">per page</span>
              </div>

              <div className="flex items-center gap-1">
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-7 w-7 p-0"
                  onClick={() => handlePageChange(1)}
                  disabled={currentPage === 1 || isSearching}
                >
                  <ChevronsLeft className="h-4 w-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-7 w-7 p-0"
                  onClick={() => handlePageChange(currentPage - 1)}
                  disabled={currentPage === 1 || isSearching}
                >
                  <ChevronLeft className="h-4 w-4" />
                </Button>
                <span className="text-xs px-2 min-w-[80px] text-center">
                  Page {currentPage} of {totalPages || 1}
                </span>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-7 w-7 p-0"
                  onClick={() => handlePageChange(currentPage + 1)}
                  disabled={currentPage >= totalPages || isSearching}
                >
                  <ChevronRight className="h-4 w-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-7 w-7 p-0"
                  onClick={() => handlePageChange(totalPages)}
                  disabled={currentPage >= totalPages || isSearching}
                >
                  <ChevronsRight className="h-4 w-4" />
                </Button>
              </div>
            </div>
          )}
        </div>

        {/* Right Panel - Modpack Details */}
        <div className="w-96 flex flex-col bg-muted/20 overflow-hidden overflow-x-hidden">
          {selectedModpack ? (
            <>
              {isLoadingDetails ? (
                <div className="flex-1 flex items-center justify-center">
                  <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                </div>
              ) : modpackDetails ? (
                <div className="flex flex-col flex-1 min-h-0">
                  {/* Header */}
                  <div className="p-4 border-b">
                    <div className="flex items-start gap-3">
                      {modpackDetails.icon_url ? (
                        <img
                          src={modpackDetails.icon_url}
                          alt={modpackDetails.name}
                          className="w-16 h-16 rounded-lg object-cover"
                        />
                      ) : (
                        <div className="w-16 h-16 rounded-lg bg-muted flex items-center justify-center">
                          <Package className="h-8 w-8 text-muted-foreground" />
                        </div>
                      )}
                      <div className="flex-1 min-w-0">
                        <h3 className="font-semibold text-lg truncate">{modpackDetails.name}</h3>
                        <p className="text-sm text-muted-foreground">
                          by <span className="font-medium text-foreground">{modpackDetails.author || "Unknown"}</span>
                          {modpackDetails.slug && (
                            <a
                              href={`https://${platform === "modrinth" ? "modrinth.com/modpack" : "curseforge.com/minecraft/modpacks"}/${modpackDetails.slug}`}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="ml-2 text-primary hover:underline inline-flex items-center gap-1"
                              onClick={(e) => {
                                e.preventDefault();
                                openExternal(`https://${platform === "modrinth" ? "modrinth.com/modpack" : "curseforge.com/minecraft/modpacks"}/${modpackDetails.slug}`);
                              }}
                            >
                              View on {platform === "modrinth" ? "Modrinth" : "CurseForge"}
                              <ExternalLink className="h-3 w-3" />
                            </a>
                          )}
                        </p>
                      </div>
                    </div>

                    <div className="flex items-center gap-4 mt-3 text-sm text-muted-foreground">
                      <span className="flex items-center gap-1">
                        <Download className="h-4 w-4" />
                        {formatNumber(modpackDetails.downloads)} downloads
                      </span>
                      {modpackDetails.followers > 0 && (
                        <span className="flex items-center gap-1">
                          <Heart className="h-4 w-4" />
                          {formatNumber(modpackDetails.followers)} followers
                        </span>
                      )}
                    </div>

                    {/* External Links */}
                    <div className="flex flex-wrap gap-2 mt-3">
                      {modpackDetails.source_url && (
                        <a
                          href={modpackDetails.source_url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                          onClick={(e) => {
                            e.preventDefault();
                            openExternal(modpackDetails.source_url!);
                          }}
                        >
                          Source <ExternalLink className="h-3 w-3" />
                        </a>
                      )}
                      {modpackDetails.issues_url && (
                        <a
                          href={modpackDetails.issues_url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                          onClick={(e) => {
                            e.preventDefault();
                            openExternal(modpackDetails.issues_url!);
                          }}
                        >
                          Issues <ExternalLink className="h-3 w-3" />
                        </a>
                      )}
                      {modpackDetails.wiki_url && (
                        <a
                          href={modpackDetails.wiki_url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                          onClick={(e) => {
                            e.preventDefault();
                            openExternal(modpackDetails.wiki_url!);
                          }}
                        >
                          Wiki <ExternalLink className="h-3 w-3" />
                        </a>
                      )}
                      {modpackDetails.discord_url && (
                        <a
                          href={modpackDetails.discord_url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                          onClick={(e) => {
                            e.preventDefault();
                            openExternal(modpackDetails.discord_url!);
                          }}
                        >
                          Discord <ExternalLink className="h-3 w-3" />
                        </a>
                      )}
                    </div>
                  </div>

                  {/* Description */}
                  <ScrollArea className="flex-1 min-h-0">
                    <div className="p-4 pr-6">
                      <div className="prose prose-sm dark:prose-invert max-w-none prose-img:rounded-lg break-words [overflow-wrap:anywhere] [word-break:break-word]">
                        <ReactMarkdown
                          remarkPlugins={[remarkGfm]}
                          rehypePlugins={[rehypeRaw, rehypeSanitize]}
                          components={markdownComponents}
                        >
                          {modpackDetails.body}
                        </ReactMarkdown>
                      </div>
                    </div>
                  </ScrollArea>

                  {/* Version Select and Install Button */}
                  <div className="p-4 border-t space-y-3">
                    <div>
                      <Label className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                        Instance Name
                      </Label>
                      <Input
                        value={instanceName}
                        onChange={(e) => setInstanceName(e.target.value)}
                        placeholder={selectedModpack?.name || "Instance name"}
                        className="mt-1 h-9"
                        disabled={isInstalling}
                      />
                    </div>
                    <div>
                      <Label className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                        Version
                      </Label>
                      <Select
                        value={selectedVersion?.id || ""}
                        onValueChange={(v) => {
                          const version = modpackVersions.find(ver => ver.id === v);
                          setSelectedVersion(version || null);
                        }}
                        disabled={isInstalling}
                      >
                        <SelectTrigger className="mt-1">
                          <SelectValue placeholder="Select version" />
                        </SelectTrigger>
                        <SelectContent>
                          {modpackVersions.map((version) => (
                            <SelectItem key={version.id} value={version.id}>
                              <div className="flex items-center gap-2">
                                <span>{version.name || version.version_number}</span>
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

                    {selectedVersion && selectedVersion.size > 0 && (
                      <div className="text-xs text-muted-foreground">
                        <span className="font-medium">Size: </span>
                        {formatFileSize(selectedVersion.size)}
                      </div>
                    )}

                    {/* Installation progress */}
                    {isInstalling && (
                      <div className="space-y-2 p-3 bg-primary/10 rounded-md">
                        <div className="flex items-center justify-between text-sm">
                          <div className="flex items-center gap-2">
                            <Loader2 className="h-4 w-4 animate-spin text-primary" />
                            <span className="text-foreground">{installProgress}</span>
                          </div>
                          {downloadProgress && downloadProgress.total > 0 && (
                            <span className="text-xs text-muted-foreground">
                              {formatDownloadSpeed(downloadProgress.speed_bps)}
                            </span>
                          )}
                        </div>
                        {/* Progress bar - real or indeterminate */}
                        {downloadProgress && downloadProgress.total > 0 ? (
                          <Progress 
                            value={(downloadProgress.downloaded / downloadProgress.total) * 100} 
                            className="h-2"
                          />
                        ) : (
                          <div className="relative h-2 w-full overflow-hidden rounded-full bg-primary/20">
                            <div className="absolute h-full w-1/3 bg-primary rounded-full animate-[progress-indeterminate_1.5s_ease-in-out_infinite]" />
                          </div>
                        )}
                      </div>
                    )}

                    {/* Installation error */}
                    {installError && (
                      <div className="flex items-start gap-2 p-2 bg-destructive/10 rounded-md text-sm text-destructive">
                        <AlertCircle className="h-4 w-4 mt-0.5 shrink-0" />
                        <span>{installError}</span>
                      </div>
                    )}

                    <Button
                      className="w-full"
                      onClick={installModpack}
                      disabled={isInstalling || !selectedVersion}
                    >
                      {isInstalling ? (
                        <>
                          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                          Installing...
                        </>
                      ) : (
                        <>
                          <Download className="mr-2 h-4 w-4" />
                          Install
                        </>
                      )}
                    </Button>
                  </div>
                </div>
              ) : (
                <div className="flex-1 flex items-center justify-center text-muted-foreground">
                  Failed to load modpack details
                </div>
              )}
            </>
          ) : (
            <div className="flex-1 flex items-center justify-center text-muted-foreground">
              <div className="text-center">
                <Package className="h-12 w-12 mx-auto mb-2 opacity-50" />
                <p>Select a modpack to view details</p>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Blocked Mods Dialog */}
      <BlockedModsDialog
        open={showBlockedModsDialog}
        onOpenChange={setShowBlockedModsDialog}
        blockedMods={currentBlockedMods}
        instanceId={currentInstanceId}
        onContinue={handleBlockedModsContinue}
        onSkip={handleBlockedModsSkip}
      />
    </div>
  );
}

export default ModpackBrowserPage;
