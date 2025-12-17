import { useState, useEffect, useMemo, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { open as openExternal } from "@tauri-apps/plugin-shell";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
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
  X,
  CheckCircle,
  Plus,
  Trash2,
  Maximize2,
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
import { Label } from "@/components/ui/label";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
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

interface ModpackDownloadDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onModpackInstalled?: () => void;
  initialPlatform?: "modrinth" | "curseforge";
}

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

interface QueuedModpack {
  id: string;
  name: string;
  version: ModpackVersion;
  platform: "modrinth" | "curseforge";
  iconUrl: string | null;
  instanceName: string;
}

type SortOption = "relevance" | "downloads" | "follows" | "newest" | "updated";

const PAGE_SIZE_OPTIONS = [10, 25, 50, 100] as const;

// Minecraft version options
const MC_VERSIONS = [
  "1.21.4", "1.21.3", "1.21.1", "1.21", "1.20.6", "1.20.4", "1.20.1",
  "1.19.4", "1.19.2", "1.18.2", "1.16.5", "1.12.2", "1.7.10"
];

// Mod loader options
const MOD_LOADERS = ["forge", "fabric", "quilt", "neoforge"];

export function ModpackDownloadDialog({
  open,
  onOpenChange,
  onModpackInstalled,
  initialPlatform = "modrinth",
}: ModpackDownloadDialogProps) {
  const navigate = useNavigate();

  // Platform state
  const [platform, setPlatform] = useState<"modrinth" | "curseforge">(initialPlatform);

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

  // Queue state
  const [queue, setQueue] = useState<QueuedModpack[]>([]);
  const [showReviewQueue, setShowReviewQueue] = useState(false);
  const [isInstalling, setIsInstalling] = useState(false);
  const [installProgress, setInstallProgress] = useState<string>("");

  // Abort dialog
  const [showAbortDialog, setShowAbortDialog] = useState(false);

  // Quick add loading state
  const [quickAddingModpack, setQuickAddingModpack] = useState<string | null>(null);

  // Instance name for adding to queue
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

  // Helper function to check if a modpack is in queue
  const isModpackInQueue = (modpackId: string): boolean => {
    return queue.some(q => q.id === modpackId);
  };

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

  // Load initial modpacks list
  useEffect(() => {
    if (open && !hasSearched) {
      searchModpacks("");
    }
  }, [open]);

  // Refresh results when platform changes
  useEffect(() => {
    if (open && hasSearched) {
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

  // Quick add modpack to queue
  const quickAddToQueue = async (modpack: ModpackSearchResult) => {
    if (queue.some(q => q.id === modpack.id)) {
      return;
    }

    setQuickAddingModpack(modpack.id);

    try {
      const versions = await invoke<ModpackVersion[]>("get_modpack_versions", {
        modpackId: modpack.id,
        platform,
        minecraftVersion: minecraftVersion || null,
        modLoader: modLoader || null,
      });

      if (versions.length === 0) {
        console.error("No compatible versions found");
        return;
      }

      const latestVersion = versions[0];

      const queuedModpack: QueuedModpack = {
        id: modpack.id,
        name: modpack.name,
        version: latestVersion,
        platform,
        iconUrl: modpack.icon_url,
        instanceName: modpack.name,
      };

      setQueue(prev => [...prev, queuedModpack]);
    } catch (error) {
      console.error("Failed to add modpack to queue:", error);
    } finally {
      setQuickAddingModpack(null);
    }
  };

  // Add modpack to queue (from details panel)
  const addToQueue = async () => {
    if (!selectedModpack || !selectedVersion) return;

    if (queue.some(q => q.id === selectedModpack.id)) {
      return;
    }

    const queuedModpack: QueuedModpack = {
      id: selectedModpack.id,
      name: selectedModpack.name,
      version: selectedVersion,
      platform,
      iconUrl: selectedModpack.icon_url,
      instanceName: instanceName || selectedModpack.name,
    };

    setQueue(prev => [...prev, queuedModpack]);
  };

  // Remove modpack from queue
  const removeFromQueue = (modpackId: string) => {
    setQueue(prev => prev.filter(q => q.id !== modpackId));
  };

  // Update instance name in queue
  const updateQueueInstanceName = (modpackId: string, name: string) => {
    setQueue(prev => prev.map(q => 
      q.id === modpackId ? { ...q, instanceName: name } : q
    ));
  };

  // Install all modpacks in queue
  const installModpacks = async () => {
    setIsInstalling(true);
    setInstallProgress("Preparing downloads...");

    try {
      for (let i = 0; i < queue.length; i++) {
        const modpack = queue[i];
        setInstallProgress(`Installing ${i + 1}/${queue.length}: ${modpack.name}...`);

        if (modpack.version.download_url) {
          await invoke("import_instance_from_url", {
            url: modpack.version.download_url,
            nameOverride: modpack.instanceName || null,
          });
        } else {
          throw new Error(`No download URL available for ${modpack.name}`);
        }
      }

      setInstallProgress("Installation complete!");
      setQueue([]);
      setShowReviewQueue(false);
      onModpackInstalled?.();

      // Close dialog and navigate after a short delay
      setTimeout(() => {
        onOpenChange(false);
        navigate("/");
      }, 1000);
    } catch (error) {
      console.error("Failed to install modpacks:", error);
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

  // Pop out dialog to a new window
  const handlePopOut = useCallback(async () => {
    try {
      // Close the current dialog first
      onOpenChange(false);

      // Create a new resizable window
      const webview = new WebviewWindow("modpack-browser", {
        url: "/modpack-browser",
        title: "Browse Modpacks",
        width: 1200,
        height: 800,
        minWidth: 800,
        minHeight: 600,
        resizable: true,
        center: true,
      });

      webview.once("tauri://created", () => {
        console.log("Modpack browser window created");
      });

      webview.once("tauri://error", (e) => {
        console.error("Failed to create modpack browser window:", e);
        // Re-open dialog if window creation fails
        onOpenChange(true);
      });
    } catch (error) {
      console.error("Failed to pop out dialog:", error);
      // Re-open dialog if pop out fails
      onOpenChange(true);
    }
  }, [onOpenChange]);

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="max-w-6xl h-[85vh] flex flex-col p-0">
          <DialogHeader className="px-6 pt-6 pb-4 border-b">
            <div className="flex items-center justify-between">
              <DialogTitle className="flex items-center gap-2">
                <Package className="h-5 w-5" />
                Browse Modpacks
              </DialogTitle>
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8"
                      onClick={handlePopOut}
                    >
                      <Maximize2 className="h-4 w-4" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>Open in new window</p>
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>
            </div>
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
                            <div className="flex items-center gap-2">
                              <span className="font-medium truncate">{modpack.name}</span>
                              {isModpackInQueue(modpack.id) && (
                                <Badge variant="secondary" className="text-[10px] px-1.5 py-0">
                                  <CheckCircle className="h-3 w-3 mr-1" />
                                  Queued
                                </Badge>
                              )}
                            </div>
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
                          <div className="flex items-center gap-1 shrink-0">
                            {isModpackInQueue(modpack.id) && (
                              <Button
                                size="sm"
                                variant="ghost"
                                className="h-8 w-8 p-0 text-destructive hover:text-destructive hover:bg-destructive/10"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  removeFromQueue(modpack.id);
                                }}
                              >
                                <X className="h-4 w-4" />
                              </Button>
                            )}
                            <Button
                              size="sm"
                              variant={isModpackInQueue(modpack.id) ? "secondary" : "default"}
                              className="shrink-0"
                              onClick={(e) => {
                                e.stopPropagation();
                                if (!isModpackInQueue(modpack.id)) {
                                  quickAddToQueue(modpack);
                                }
                              }}
                              disabled={isModpackInQueue(modpack.id) || quickAddingModpack === modpack.id}
                            >
                              {quickAddingModpack === modpack.id ? (
                                <Loader2 className="h-4 w-4 animate-spin" />
                              ) : isModpackInQueue(modpack.id) ? (
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
                        {PAGE_SIZE_OPTIONS.map((size) => (
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
            <div className="w-96 flex flex-col bg-muted/20">
              {selectedModpack ? (
                <>
                  {isLoadingDetails ? (
                    <div className="flex-1 flex items-center justify-center">
                      <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                    </div>
                  ) : modpackDetails ? (
                    <>
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
                      <ScrollArea className="flex-1 p-4">
                        <div className="prose prose-sm dark:prose-invert max-w-none prose-img:rounded-lg">
                          <ReactMarkdown
                            remarkPlugins={[remarkGfm]}
                            rehypePlugins={[rehypeRaw, rehypeSanitize]}
                            components={markdownComponents}
                          >
                            {modpackDetails.body}
                          </ReactMarkdown>
                        </div>
                      </ScrollArea>

                      {/* Version Select and Add Button */}
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

                        <Button
                          className="w-full"
                          onClick={addToQueue}
                          disabled={isModpackInQueue(selectedModpack.id) || !selectedVersion}
                        >
                          {isModpackInQueue(selectedModpack.id) ? (
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

          {/* Footer with Queue */}
          <div className="px-6 py-4 border-t bg-muted/30 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground">
                Queue: <strong>{queue.length}</strong> modpack{queue.length !== 1 ? "s" : ""}
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
              {queue.map((modpack) => (
                <div key={modpack.id} className="flex items-center gap-3 p-3 bg-muted/50 rounded-lg">
                  {modpack.iconUrl ? (
                    <img
                      src={modpack.iconUrl}
                      alt={modpack.name}
                      className="w-10 h-10 rounded object-cover"
                    />
                  ) : (
                    <div className="w-10 h-10 rounded bg-muted flex items-center justify-center">
                      <Package className="h-5 w-5 text-muted-foreground" />
                    </div>
                  )}
                  <div className="flex-1 min-w-0">
                    <p className="font-medium truncate">{modpack.name}</p>
                    <p className="text-xs text-muted-foreground">
                      {modpack.version.name || modpack.version.version_number}
                    </p>
                    <Input
                      value={modpack.instanceName}
                      onChange={(e) => updateQueueInstanceName(modpack.id, e.target.value)}
                      placeholder="Instance name"
                      className="mt-2 h-8 text-xs"
                      disabled={isInstalling}
                    />
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => removeFromQueue(modpack.id)}
                    disabled={isInstalling}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
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
                onClick={installModpacks}
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
                    Install {queue.length} modpack{queue.length !== 1 ? "s" : ""}
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
              This will clear your download queue. You'll need to add the modpacks again if you want to install them.
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

export default ModpackDownloadDialog;
