import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { open as openExternal } from "@tauri-apps/plugin-shell";
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
  ArrowDownCircle,
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
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";

// Platform logos
const ModrinthLogo = () => (
  <svg viewBox="0 0 512 514" className="h-4 w-4" fill="currentColor">
    <path d="M503.16 323.56C514.55 281.47 515.32 235.91 503.2 190.76C466.57 54.2299 326.04 -26.8001 189.33 9.77991C83.8101 38.0199 11.3899 128.07 0.689941 230.47H43.99C54.29 147.33 113.74 74.7298 199.75 51.7098C306.05 23.2598 415.13 80.6699 453.17 181.38L411.03 192.65C391.64 145.8 352.57 111.45 306.3 96.8198L298.56 140.66C335.09 154.13 364.72 184.5 375.56 224.91C391.36 283.8 361.94 344.14 308.56 369.17L320.09 412.16C390.25 383.21 432.4 310.3 422.43 235.14L464.41 223.91C468.91 252.62 467.35 281.16 460.55 308.07L503.16 323.56Z" />
    <path d="M321.99 504.22C185.27 540.8 44.7501 459.77 8.11011 323.24C3.84011 307.31 1.17 291.33 0 275.46H43.27C44.36 287.37 46.4699 299.35 49.6799 311.29C53.0399 323.8 57.45 335.75 62.79 347.07L101.38 323.92C98.1299 316.42 95.39 308.6 93.21 300.47C69.17 210.87 122.41 118.77 212.13 94.7601C229.13 90.2101 246.23 88.4401 262.93 89.1501L255.19 133C244.73 133.05 234.11 134.42 223.53 137.25C157.31 154.98 118.01 222.95 135.75 289.09C136.85 293.16 138.13 297.13 139.59 300.99L188.94 271.38L174.07 231.95L220.67 184.36L279.57 171.03L296.62 192.15L281.1 252.07L244.63 293.78H197.98L148.63 323.39C163.78 359.57 195.84 386.54 235.83 397.46C301.98 415.17 371.23 376.27 388.98 310.12C391.96 299.12 393.27 287.99 393.04 277L436.53 265.76C436.85 285.09 434.79 304.58 430.06 323.9C413.36 387.89 368.96 438.74 311.42 465.53L323.05 508.51C328.05 506.71 332.98 504.72 337.84 502.55L337.84 502.55C345.84 499.04 353.63 495.1 361.18 490.75L375.08 532.93C364.48 539.15 353.38 544.63 341.82 549.33L332.79 545.46L321.99 504.22Z" />
  </svg>
);

const CurseForgeLogo = () => (
  <svg viewBox="0 0 24 24" className="h-4 w-4" fill="currentColor">
    <path d="M18.326 9.2177L15.892 9.21765C17.254 10.3397 18.147 11.9859 18.326 13.8296H23.9999L18.326 9.2177ZM13.268 19.2319L13.263 19.218C12.762 18.9558 12.312 18.5997 11.94 18.1696C11.287 17.4135 10.841 16.4694 10.701 15.4313H10.7L10.6949 15.3992C10.6479 15.0633 10.6239 14.7233 10.6239 14.3812C10.6239 12.4734 11.4629 10.7632 12.7779 9.59917L12.778 9.59922C13.166 9.24919 13.604 8.95017 14.079 8.71515H8.16187V16.4754L10.7 19.0135V24H4.02899C2.27397 22.2469 1.15194 19.8888 0.979932 17.2686C0.89593 16.0206 1.03793 14.7655 1.40394 13.5835H1.40399C2.52602 9.92232 5.53608 7.09023 9.32816 6.20922L6.19909 3.08115V0H17.2461L21.073 3.82709H18.326L14.501 0.00109863L14.4999 5.58617C19.662 6.29619 23.606 10.3072 23.9999 15.431H18.326C18.036 13.5695 17.0349 11.9434 15.609 10.8653L14.8949 10.3023L14.893 10.3033C13.965 9.61425 12.8129 9.2063 11.569 9.2063C10.961 9.2063 10.374 9.3023 9.82297 9.47931H9.82302C7.54196 10.1923 5.85392 12.1413 5.47592 14.5233C5.41292 14.8873 5.37891 15.2613 5.37891 15.6443C5.37891 16.2494 5.46391 16.8344 5.62092 17.3874L5.62297 17.3944C5.65097 17.4954 5.68397 17.5964 5.71897 17.6974L5.71803 17.6964L6.15897 18.9925L6.16095 18.9965L6.52698 19.9596L6.85299 20.5677L7.30698 21.3037L7.77198 22.1988H13.268V19.2319Z" />
  </svg>
);

// Platform configuration
type PlatformType = "modrinth" | "curseforge";

interface PlatformConfig {
  id: PlatformType;
  name: string;
  icon: React.ReactNode;
  color: string;
  hasVersionFilter: boolean;
  hasLoaderFilter: boolean;
}

const PLATFORMS: PlatformConfig[] = [
  { 
    id: "modrinth", 
    name: "Modrinth", 
    icon: <ModrinthLogo />, 
    color: "bg-green-500/20 text-green-600 dark:text-green-400",
    hasVersionFilter: true,
    hasLoaderFilter: true,
  },
  { 
    id: "curseforge", 
    name: "CurseForge", 
    icon: <CurseForgeLogo />, 
    color: "bg-orange-500/20 text-orange-600 dark:text-orange-400",
    hasVersionFilter: true,
    hasLoaderFilter: true,
  },
];

interface ModpackDownloadDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onModpackInstalled?: () => void;
  initialPlatform?: PlatformType;
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

// Helper functions
function formatNumber(num: number): string {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
  return num.toString();
}

function formatFileSize(bytes: number): string {
  if (bytes >= 1073741824) return `${(bytes / 1073741824).toFixed(2)} GB`;
  if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(2)} MB`;
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  return `${bytes} B`;
}

function formatDate(dateString: string): string {
  if (!dateString) return "Unknown";
  try {
    const date = new Date(dateString);
    return date.toLocaleDateString(undefined, { 
      year: "numeric", 
      month: "short", 
      day: "numeric" 
    });
  } catch {
    return "Unknown";
  }
}

// Markdown components
const markdownComponents: Components = {
  a: ({ href, children }) => (
    <a
      href={href}
      className="text-primary hover:underline"
      onClick={(e) => {
        e.preventDefault();
        if (href) openExternal(href);
      }}
    >
      {children}
    </a>
  ),
  img: ({ src, alt }) => (
    <img src={src} alt={alt || ""} className="max-w-full h-auto rounded-md my-2" />
  ),
  h1: ({ children }) => <h1 className="text-2xl font-bold mt-4 mb-2">{children}</h1>,
  h2: ({ children }) => <h2 className="text-xl font-bold mt-3 mb-2">{children}</h2>,
  h3: ({ children }) => <h3 className="text-lg font-semibold mt-2 mb-1">{children}</h3>,
  p: ({ children }) => <p className="my-2">{children}</p>,
  ul: ({ children }) => <ul className="list-disc list-inside my-2 space-y-1">{children}</ul>,
  ol: ({ children }) => <ol className="list-decimal list-inside my-2 space-y-1">{children}</ol>,
  code: ({ children, className }) => {
    const isInline = !className;
    return isInline ? (
      <code className="bg-muted px-1 py-0.5 rounded text-sm">{children}</code>
    ) : (
      <code className="block bg-muted p-3 rounded-md text-sm overflow-x-auto my-2">
        {children}
      </code>
    );
  },
  blockquote: ({ children }) => (
    <blockquote className="border-l-4 border-primary/50 pl-4 my-2 italic">
      {children}
    </blockquote>
  ),
  hr: () => <hr className="my-4 border-border" />,
  table: ({ children }) => (
    <div className="overflow-x-auto my-2">
      <table className="min-w-full border-collapse border border-border">{children}</table>
    </div>
  ),
  th: ({ children }) => (
    <th className="border border-border px-3 py-2 bg-muted font-semibold text-left">{children}</th>
  ),
  td: ({ children }) => (
    <td className="border border-border px-3 py-2">{children}</td>
  ),
};

export function ModpackDownloadDialog({
  open,
  onOpenChange,
  onModpackInstalled,
  initialPlatform = "modrinth",
}: ModpackDownloadDialogProps) {
  const navigate = useNavigate();
  
  // Platform state
  const [platform, setPlatform] = useState<PlatformType>(initialPlatform);
  
  // Search state
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<ModpackSearchResult[]>([]);
  const [searching, setSearching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  // Filters
  const [minecraftVersion, setMinecraftVersion] = useState<string>("");
  const [modLoader, setModLoader] = useState<string>("");
  const [sortBy, setSortBy] = useState<string>("relevance");
  
  // Pagination
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(25);
  const [totalResults, setTotalResults] = useState(0);
  
  // Selected modpack
  const [selectedModpack, setSelectedModpack] = useState<ModpackSearchResult | null>(null);
  const [modpackDetails, setModpackDetails] = useState<ModpackDetails | null>(null);
  const [modpackVersions, setModpackVersions] = useState<ModpackVersion[]>([]);
  const [loadingDetails, setLoadingDetails] = useState(false);
  
  // Installation state
  const [selectedVersion, setSelectedVersion] = useState<ModpackVersion | null>(null);
  const [instanceName, setInstanceName] = useState("");
  const [installing, setInstalling] = useState(false);
  const [installProgress, setInstallProgress] = useState(0);
  const [installStatus, setInstallStatus] = useState("");

  const currentPlatformConfig = useMemo(() => 
    PLATFORMS.find(p => p.id === platform) || PLATFORMS[0],
    [platform]
  );
  
  const totalPages = useMemo(() => 
    Math.ceil(totalResults / pageSize),
    [totalResults, pageSize]
  );

  // Reset state when platform changes
  useEffect(() => {
    setSearchResults([]);
    setSelectedModpack(null);
    setModpackDetails(null);
    setModpackVersions([]);
    setCurrentPage(1);
    setTotalResults(0);
    setError(null);
  }, [platform]);

  // Reset state when dialog opens
  useEffect(() => {
    if (open) {
      setSearchQuery("");
      setSearchResults([]);
      setSelectedModpack(null);
      setModpackDetails(null);
      setModpackVersions([]);
      setCurrentPage(1);
      setTotalResults(0);
      setError(null);
      setSelectedVersion(null);
      setInstanceName("");
      setInstalling(false);
      setInstallProgress(0);
      setInstallStatus("");
    }
  }, [open]);

  // Search function
  const handleSearch = async (page = 1) => {
    setSearching(true);
    setError(null);
    setSelectedModpack(null);
    
    try {
      const offset = (page - 1) * pageSize;
      
      const response = await invoke<ModpackSearchResponse>("search_modpacks", {
        query: searchQuery,
        platform,
        minecraftVersion: minecraftVersion || null,
        modLoader: modLoader || null,
        sort: sortBy,
        offset,
        limit: pageSize,
      });
      
      setSearchResults(response.modpacks);
      setTotalResults(response.total_hits);
      setCurrentPage(page);
    } catch (err) {
      console.error("Search failed:", err);
      setError(String(err));
      setSearchResults([]);
      setTotalResults(0);
    } finally {
      setSearching(false);
    }
  };

  // Load modpack details
  const handleSelectModpack = async (modpack: ModpackSearchResult) => {
    setSelectedModpack(modpack);
    setLoadingDetails(true);
    setModpackDetails(null);
    setModpackVersions([]);
    setSelectedVersion(null);
    setInstanceName(modpack.name);
    
    // Only fetch details for platforms that support it
    if (platform === "modrinth" || platform === "curseforge") {
      try {
        const [details, versions] = await Promise.all([
          invoke<ModpackDetails>("get_modpack_details", {
            modpackId: modpack.id,
            platform,
          }),
          invoke<ModpackVersion[]>("get_modpack_versions", {
            modpackId: modpack.id,
            platform,
            minecraftVersion: minecraftVersion || null,
            modLoader: modLoader || null,
          }),
        ]);
        
        setModpackDetails(details);
        setModpackVersions(versions);
        
        // Auto-select first version
        if (versions.length > 0) {
          setSelectedVersion(versions[0]);
        }
      } catch (err) {
        console.error("Failed to load modpack details:", err);
        setError(String(err));
      }
    }
    
    setLoadingDetails(false);
  };

  // Install modpack
  const handleInstall = async () => {
    if (!selectedModpack || !selectedVersion) return;
    
    setInstalling(true);
    setInstallProgress(0);
    setInstallStatus("Downloading modpack...");
    
    try {
      // For Modrinth/CurseForge, we use URL import
      if (selectedVersion.download_url) {
        await invoke("import_instance_from_url", {
          url: selectedVersion.download_url,
          nameOverride: instanceName || null,
        });
        
        setInstallProgress(100);
        setInstallStatus("Installation complete!");
        
        // Navigate to instances after delay
        setTimeout(() => {
          onModpackInstalled?.();
          onOpenChange(false);
          navigate("/");
        }, 1500);
      } else {
        throw new Error("No download URL available for this version");
      }
    } catch (err) {
      console.error("Installation failed:", err);
      setError(String(err));
      setInstalling(false);
      setInstallProgress(0);
      setInstallStatus("");
    }
  };

  // Go back to search results
  const handleBackToResults = () => {
    setSelectedModpack(null);
    setModpackDetails(null);
    setModpackVersions([]);
    setSelectedVersion(null);
    setInstanceName("");
    setError(null);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-5xl w-[95vw] h-[85vh] max-h-[85vh] flex flex-col p-0 overflow-hidden">
        <DialogHeader className="p-4 md:p-6 pb-0 flex-shrink-0">
          <DialogTitle className="flex items-center gap-2">
            <Package className="h-5 w-5" />
            Browse Modpacks
          </DialogTitle>
        </DialogHeader>

        <div className="flex flex-1 overflow-hidden flex-col md:flex-row">
          {/* Sidebar - Platform Selection (horizontal on mobile) */}
          <div className="md:w-40 border-b md:border-b-0 md:border-r p-2 md:p-4 flex md:flex-col gap-2 flex-shrink-0 overflow-x-auto md:overflow-x-visible">
            <Label className="hidden md:block text-xs text-muted-foreground uppercase tracking-wider">Platform</Label>
            {PLATFORMS.map((p) => (
              <button
                key={p.id}
                type="button"
                onClick={() => setPlatform(p.id)}
                className={cn(
                  "flex items-center gap-2 px-3 py-2 rounded-md text-sm transition-colors whitespace-nowrap md:w-full",
                  platform === p.id
                    ? "bg-primary text-primary-foreground"
                    : "hover:bg-muted"
                )}
              >
                {p.icon}
                {p.name}
              </button>
            ))}
          </div>

          {/* Main Content */}
          <div className="flex-1 flex flex-col overflow-hidden">
            {/* Search Bar */}
            <div className="p-3 md:p-4 border-b space-y-2 md:space-y-3 flex-shrink-0">
              <div className="flex gap-2">
                <div className="relative flex-1">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                  <Input
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && handleSearch(1)}
                    placeholder={`Search ${currentPlatformConfig.name}...`}
                    className="pl-9"
                  />
                </div>
                <Button onClick={() => handleSearch(1)} disabled={searching} size="sm" className="md:size-default">
                  {searching ? <Loader2 className="h-4 w-4 animate-spin" /> : "Search"}
                </Button>
              </div>
              
              {/* Filters - scrollable on mobile */}
              <div className="flex gap-2 overflow-x-auto pb-1">
                {currentPlatformConfig.hasVersionFilter && (
                  <Select value={minecraftVersion || "__all__"} onValueChange={(v) => setMinecraftVersion(v === "__all__" ? "" : v)}>
                    <SelectTrigger className="w-28 md:w-32 flex-shrink-0">
                      <SelectValue placeholder="MC Ver" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="__all__">All Versions</SelectItem>
                      <SelectItem value="1.21.4">1.21.4</SelectItem>
                      <SelectItem value="1.21.3">1.21.3</SelectItem>
                      <SelectItem value="1.21.1">1.21.1</SelectItem>
                      <SelectItem value="1.21">1.21</SelectItem>
                      <SelectItem value="1.20.6">1.20.6</SelectItem>
                      <SelectItem value="1.20.4">1.20.4</SelectItem>
                      <SelectItem value="1.20.1">1.20.1</SelectItem>
                      <SelectItem value="1.19.4">1.19.4</SelectItem>
                      <SelectItem value="1.19.2">1.19.2</SelectItem>
                      <SelectItem value="1.18.2">1.18.2</SelectItem>
                      <SelectItem value="1.16.5">1.16.5</SelectItem>
                      <SelectItem value="1.12.2">1.12.2</SelectItem>
                    </SelectContent>
                  </Select>
                )}
                
                {currentPlatformConfig.hasLoaderFilter && (
                  <Select value={modLoader || "__all__"} onValueChange={(v) => setModLoader(v === "__all__" ? "" : v)}>
                    <SelectTrigger className="w-28 md:w-32 flex-shrink-0">
                      <SelectValue placeholder="Loader" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="__all__">All Loaders</SelectItem>
                      <SelectItem value="forge">Forge</SelectItem>
                      <SelectItem value="fabric">Fabric</SelectItem>
                      <SelectItem value="quilt">Quilt</SelectItem>
                      <SelectItem value="neoforge">NeoForge</SelectItem>
                    </SelectContent>
                  </Select>
                )}
                
                <Select value={sortBy} onValueChange={setSortBy}>
                  <SelectTrigger className="w-28 md:w-32 flex-shrink-0">
                    <SelectValue placeholder="Sort" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="relevance">Relevance</SelectItem>
                    <SelectItem value="downloads">Downloads</SelectItem>
                    <SelectItem value="follows">Popularity</SelectItem>
                    <SelectItem value="newest">Newest</SelectItem>
                    <SelectItem value="updated">Updated</SelectItem>
                  </SelectContent>
                </Select>
                
                <Select value={pageSize.toString()} onValueChange={(v) => setPageSize(Number(v))}>
                  <SelectTrigger className="w-20 flex-shrink-0">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="10">10</SelectItem>
                    <SelectItem value="25">25</SelectItem>
                    <SelectItem value="50">50</SelectItem>
                    <SelectItem value="100">100</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            {/* Error Display */}
            {error && (
              <div className="mx-4 mt-4 p-3 bg-destructive/10 border border-destructive/30 rounded-md text-destructive text-sm">
                {error}
              </div>
            )}

            {/* Content Area */}
            <div className="flex-1 overflow-hidden">
              {selectedModpack ? (
                // Modpack Detail View
                <div className="h-full flex flex-col">
                  {/* Header */}
                  <div className="p-3 md:p-4 border-b flex flex-wrap items-start gap-3 md:gap-4 flex-shrink-0">
                    <Button variant="ghost" size="sm" onClick={handleBackToResults}>
                      <ChevronLeft className="h-4 w-4 mr-1" /> Back
                    </Button>
                    
                    {selectedModpack.icon_url && (
                      <img
                        src={selectedModpack.icon_url}
                        alt={selectedModpack.name}
                        className="w-12 h-12 md:w-16 md:h-16 rounded-lg object-cover"
                      />
                    )}
                    
                    <div className="flex-1 min-w-0">
                      <h2 className="text-lg md:text-xl font-bold truncate">{selectedModpack.name}</h2>
                      <p className="text-sm text-muted-foreground">
                        by {selectedModpack.author || "Unknown"}
                      </p>
                      <div className="flex items-center gap-4 mt-1 text-sm text-muted-foreground">
                        <span className="flex items-center gap-1">
                          <Download className="h-3 w-3" />
                          {formatNumber(selectedModpack.downloads)}
                        </span>
                        {selectedModpack.follows > 0 && (
                          <span className="flex items-center gap-1">
                            <Heart className="h-3 w-3" />
                            {formatNumber(selectedModpack.follows)}
                          </span>
                        )}
                      </div>
                    </div>
                    
                    <Badge className={cn(currentPlatformConfig.color, "hidden md:flex")}>
                      {currentPlatformConfig.icon}
                      <span className="ml-1">{selectedModpack.platform}</span>
                    </Badge>
                  </div>

                  {/* Detail Content */}
                  {loadingDetails ? (
                    <div className="flex-1 flex items-center justify-center">
                      <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                    </div>
                  ) : (
                    <div className="flex-1 flex flex-col md:flex-row overflow-hidden">
                      {/* Description Panel */}
                      <div className="flex-1 md:border-r overflow-hidden min-h-0">
                        <ScrollArea className="h-full p-3 md:p-4">
                          {modpackDetails?.body ? (
                            <div className="prose prose-sm dark:prose-invert max-w-none">
                              <ReactMarkdown
                                remarkPlugins={[remarkGfm]}
                                rehypePlugins={[rehypeRaw, rehypeSanitize]}
                                components={markdownComponents}
                              >
                                {modpackDetails.body}
                              </ReactMarkdown>
                            </div>
                          ) : (
                            <p className="text-muted-foreground">{selectedModpack.description}</p>
                          )}
                        </ScrollArea>
                      </div>

                      {/* Version Selection Panel */}
                      <div className="w-full md:w-72 lg:w-80 flex flex-col p-3 md:p-4 border-t md:border-t-0 flex-shrink-0 max-h-64 md:max-h-none">
                        <Label className="mb-2">Instance Name</Label>
                        <Input
                          value={instanceName}
                          onChange={(e) => setInstanceName(e.target.value)}
                          placeholder="Instance name"
                          className="mb-3"
                        />
                        
                        {modpackVersions.length > 0 ? (
                          <>
                            <Label className="mb-2">Select Version</Label>
                            <ScrollArea className="flex-1 border rounded-md min-h-0">
                              <div className="p-2 space-y-1">
                                {modpackVersions.map((version) => (
                                  <button
                                    key={version.id}
                                    type="button"
                                    onClick={() => setSelectedVersion(version)}
                                    className={cn(
                                      "w-full text-left p-2 rounded-md transition-colors",
                                      selectedVersion?.id === version.id
                                        ? "bg-primary text-primary-foreground"
                                        : "hover:bg-muted"
                                    )}
                                  >
                                    <div className="font-medium text-sm truncate">
                                      {version.name}
                                    </div>
                                    <div className="text-xs opacity-70 flex items-center gap-2 mt-0.5">
                                      <span>{version.game_versions.slice(0, 2).join(", ")}</span>
                                      {version.size > 0 && (
                                        <span className="hidden sm:inline">• {formatFileSize(version.size)}</span>
                                      )}
                                    </div>
                                    <div className="text-xs opacity-50 mt-0.5 hidden sm:block">
                                      {formatDate(version.date_published)}
                                    </div>
                                  </button>
                                ))}
                              </div>
                            </ScrollArea>
                          </>
                        ) : (
                          <p className="text-sm text-muted-foreground">
                            No versions available
                          </p>
                        )}
                        
                        {installing ? (
                          <div className="mt-3 space-y-2">
                            <Progress value={installProgress} />
                            <p className="text-sm text-center text-muted-foreground">
                              {installStatus}
                            </p>
                          </div>
                        ) : (
                          <Button
                            className="mt-3"
                            onClick={handleInstall}
                            disabled={!selectedVersion}
                          >
                            <ArrowDownCircle className="h-4 w-4 mr-2" />
                            Install
                          </Button>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              ) : (
                // Search Results View
                <ScrollArea className="h-full">
                  {searchResults.length > 0 ? (
                    <div className="p-2 md:p-4 space-y-2">
                      {searchResults.map((modpack) => (
                        <button
                          key={modpack.id}
                          type="button"
                          onClick={() => handleSelectModpack(modpack)}
                          className="w-full flex items-start gap-3 md:gap-4 p-2 md:p-3 rounded-lg border bg-card hover:bg-accent/50 transition-colors text-left"
                        >
                          {modpack.icon_url ? (
                            <img
                              src={modpack.icon_url}
                              alt={modpack.name}
                              className="w-10 h-10 md:w-12 md:h-12 rounded-md object-cover flex-shrink-0"
                            />
                          ) : (
                            <div className="w-10 h-10 md:w-12 md:h-12 rounded-md bg-muted flex items-center justify-center flex-shrink-0">
                              <Package className="h-5 w-5 md:h-6 md:w-6 text-muted-foreground" />
                            </div>
                          )}
                          
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2 flex-wrap">
                              <h3 className="font-semibold truncate text-sm md:text-base">{modpack.name}</h3>
                              <Badge variant="outline" className="text-xs hidden sm:inline-flex">
                                {modpack.versions.slice(0, 1).join(", ") || "Various"}
                              </Badge>
                            </div>
                            <p className="text-xs md:text-sm text-muted-foreground line-clamp-2 mt-0.5">
                              {modpack.description}
                            </p>
                            <div className="flex items-center gap-3 md:gap-4 mt-2 text-xs text-muted-foreground flex-wrap">
                              {modpack.author && <span className="hidden sm:inline">by {modpack.author}</span>}
                              <span className="flex items-center gap-1">
                                <Download className="h-3 w-3" />
                                {formatNumber(modpack.downloads)}
                              </span>
                              {modpack.follows > 0 && (
                                <span className="flex items-center gap-1 hidden sm:flex">
                                  <Heart className="h-3 w-3" />
                                  {formatNumber(modpack.follows)}
                                </span>
                              )}
                              {modpack.loaders.length > 0 && (
                                <span>{modpack.loaders.join(", ")}</span>
                              )}
                            </div>
                          </div>
                        </button>
                      ))}
                    </div>
                  ) : searching ? (
                    <div className="h-full flex items-center justify-center">
                      <div className="flex flex-col items-center gap-2 text-muted-foreground">
                        <Loader2 className="h-8 w-8 animate-spin" />
                        <p>Searching...</p>
                      </div>
                    </div>
                  ) : (
                    <div className="h-full flex items-center justify-center">
                      <div className="flex flex-col items-center gap-2 text-muted-foreground">
                        <Search className="h-12 w-12" />
                        <p>Search for modpacks to get started</p>
                      </div>
                    </div>
                  )}
                </ScrollArea>
              )}
            </div>

            {/* Pagination */}
            {!selectedModpack && totalResults > pageSize && (
              <div className="p-2 md:p-3 border-t flex flex-col sm:flex-row items-center justify-between gap-2 flex-shrink-0">
                <span className="text-xs md:text-sm text-muted-foreground">
                  {totalResults.toLocaleString()} results
                </span>
                <div className="flex items-center gap-1">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleSearch(1)}
                    disabled={currentPage === 1 || searching}
                    className="h-7 w-7 p-0 md:h-8 md:w-8"
                  >
                    <ChevronsLeft className="h-3 w-3 md:h-4 md:w-4" />
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleSearch(currentPage - 1)}
                    disabled={currentPage === 1 || searching}
                    className="h-7 w-7 p-0 md:h-8 md:w-8"
                  >
                    <ChevronLeft className="h-3 w-3 md:h-4 md:w-4" />
                  </Button>
                  <span className="px-2 md:px-3 text-xs md:text-sm whitespace-nowrap">
                    {currentPage}/{totalPages}
                  </span>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleSearch(currentPage + 1)}
                    disabled={currentPage >= totalPages || searching}
                    className="h-7 w-7 p-0 md:h-8 md:w-8"
                  >
                    <ChevronRight className="h-3 w-3 md:h-4 md:w-4" />
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleSearch(totalPages)}
                    disabled={currentPage >= totalPages || searching}
                    className="h-7 w-7 p-0 md:h-8 md:w-8"
                  >
                    <ChevronsRight className="h-3 w-3 md:h-4 md:w-4" />
                  </Button>
                </div>
              </div>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

export default ModpackDownloadDialog;
