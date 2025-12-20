// Modpack search tab for browsing and installing modpacks inline
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

import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import ReactMarkdown, { Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import { 
  Search, 
  Download, 
  Heart, 
  Loader2, 
  Package,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  ArrowDownCircle,
  X,
  Filter,
  ExternalLink,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";
import { open as openExternal } from "@tauri-apps/plugin-shell";

// Platform Logos
function ModrinthLogo({ className = "h-4 w-4" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 512 514" fill="currentColor">
      <path fillRule="evenodd" clipRule="evenodd" d="M503.16 323.56C514.55 281.47 515.32 235.91 503.2 190.76C466.57 54.2299 326.04 -26.8001 189.33 9.77991C83.8101 38.0199 11.3899 128.07 0.689941 230.47H43.99C54.29 147.33 113.74 74.7298 199.75 51.7098C306.05 23.2598 415.13 80.6699 453.17 181.38L411.03 192.65C391.64 145.8 352.57 111.45 306.3 96.8198L298.56 140.66C335.09 154.13 364.72 184.5 375.56 224.91C391.36 283.8 361.94 344.14 308.56 369.17L320.09 412.16C390.25 383.21 432.4 310.3 422.43 235.14L464.41 223.91C468.91 252.62 467.35 281.16 460.55 308.07L503.16 323.56Z" />
      <path d="M321.99 504.22C185.27 540.8 44.7501 459.77 8.11011 323.24C3.84011 307.31 1.17 291.33 0 275.46H43.27C44.36 287.37 46.4699 299.35 49.6799 311.29C53.0399 323.8 57.45 335.75 62.79 347.07L101.38 323.92C98.1299 316.42 95.39 308.6 93.21 300.47C69.17 210.87 122.41 118.77 212.13 94.76C229.13 90.21 246.23 88.4299 262.93 89.1499L255.19 133C244.73 133.05 234.11 134.42 223.53 137.25C157.31 154.98 118.01 222.95 135.75 289.09C136.85 293.16 138.13 297.13 139.59 300.99L188.94 271.38L174.07 231.95L220.67 184.36L279.57 171.03L296.62 192.15L281.36 252.77L308.93 267.06L327.46 246.22L379.37 231.62L395.27 267.23C399.91 278.73 402.63 291.28 402.63 304.35C402.63 352.95 369.55 395.61 324.21 408.73L332.59 452.55C393.45 435.25 440.27 385.64 446.25 325.98L486.35 349.58C457.61 424.21 390.05 481.93 307.19 504.12L321.99 504.22Z" />
    </svg>
  );
}

function CurseForgeLogo({ className = "h-4 w-4" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor">
      <path d="M18.326 9.2067L23.9997 9.203V6.6917H16.4722L16.4717 6.6873L16.4635 6.6917H0.0219727V9.203H5.66867C4.31793 9.88493 3.50887 10.8443 3.18577 12.1203C2.87833 13.3377 2.54427 14.2533 1.8137 15.1693L1.22073 15.9733H2.7851C3.66457 15.9733 4.46527 15.4793 4.89293 14.7023L5.3961 13.7833L6.35593 15.5103L6.3591 15.5103L6.87647 16.5033H9.80357V20.3553H14.2017V16.5035H16.8277L17.3621 15.5061L17.3651 15.5061L18.3076 13.7913L18.8148 14.7053C19.2431 15.4813 20.0432 15.9743 20.9219 15.9743H22.7781L22.1857 15.1703C21.4551 14.2543 21.1211 13.3387 20.8137 12.1213C20.5135 10.9303 19.7877 9.9813 18.5467 9.27867L18.326 9.2067Z" />
    </svg>
  );
}

function ATLauncherLogo({ className = "h-4 w-4" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 2L2 19h20L12 2zm0 4l6.5 11h-13L12 6z"/>
    </svg>
  );
}

function FTBLogo({ className = "h-4 w-4" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor">
      <path d="M4 4h16v2H4V4zm0 4h10v2H4V8zm0 4h16v2H4v-2zm0 4h10v2H4v-2z"/>
    </svg>
  );
}

type Platform = "modrinth" | "curseforge" | "atlauncher" | "ftb-legacy";

const PLATFORMS: { id: Platform; label: string; logo: React.FC<{ className?: string }>; hasDetails: boolean }[] = [
  { id: "modrinth", label: "Modrinth", logo: ModrinthLogo, hasDetails: true },
  { id: "curseforge", label: "CurseForge", logo: CurseForgeLogo, hasDetails: true },
  { id: "atlauncher", label: "ATLauncher", logo: ATLauncherLogo, hasDetails: false },
  { id: "ftb-legacy", label: "FTB Legacy", logo: FTBLogo, hasDetails: false },
];

interface ModpackSearchTabProps {
  name: string;
  setName: (name: string) => void;
  group: string;
  setGroup: (group: string) => void;
  selectedModpack: ModpackResult | null;
  setSelectedModpack: (pack: ModpackResult | null) => void;
}

interface ModpackResult {
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
  downloads: number;
  follows: number;
  icon_url: string | null;
  source_url: string | null;
  issues_url: string | null;
  wiki_url: string | null;
  discord_url: string | null;
  categories: string[];
  versions: string[];
  loaders: string[];
}

interface ModpackSearchResponse {
  modpacks: ModpackResult[];
  total_hits: number;
  offset: number;
  limit: number;
}

type SortOption = "relevance" | "downloads" | "follows" | "newest" | "updated";

const SORT_OPTIONS: { value: SortOption; label: string }[] = [
  { value: "relevance", label: "Relevance" },
  { value: "downloads", label: "Downloads" },
  { value: "follows", label: "Popularity" },
  { value: "newest", label: "Newest" },
  { value: "updated", label: "Recently Updated" },
];

const LOADER_OPTIONS = [
  { value: "", label: "Any Loader" },
  { value: "fabric", label: "Fabric" },
  { value: "forge", label: "Forge" },
  { value: "quilt", label: "Quilt" },
  { value: "neoforge", label: "NeoForge" },
];

// Page size options - CurseForge API max is 50, Modrinth allows 100
const PAGE_SIZE_OPTIONS_MODRINTH = [10, 25, 50, 100] as const;
const PAGE_SIZE_OPTIONS_CURSEFORGE = [10, 25, 50] as const;

const getPageSizeOptions = (platform: Platform) =>
  platform === "curseforge" ? PAGE_SIZE_OPTIONS_CURSEFORGE : PAGE_SIZE_OPTIONS_MODRINTH;

const getMaxPageSize = (platform: Platform) =>
  platform === "curseforge" ? 50 : 100;

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
  if (!dateString) return "";
  try {
    const date = new Date(dateString);
    return date.toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
  } catch {
    return "";
  }
}

export function ModpackSearchTab({
  name,
  setName,
  group,
  setGroup,
  selectedModpack,
  setSelectedModpack,
}: ModpackSearchTabProps) {
  const navigate = useNavigate();
  
  // Platform state - support all 4 platforms
  const [platform, setPlatform] = useState<Platform>("modrinth");
  
  // Check if current platform supports detailed views
  const currentPlatformConfig = PLATFORMS.find(p => p.id === platform);
  const hasDetailSupport = currentPlatformConfig?.hasDetails ?? false;
  
  // Search state
  const [modpackSearch, setModpackSearch] = useState("");
  const [modpackResults, setModpackResults] = useState<ModpackResult[]>([]);
  const [searchingModpacks, setSearchingModpacks] = useState(false);
  const [hasSearched, setHasSearched] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  // Sorting and filtering
  const [sortBy, setSortBy] = useState<SortOption>("downloads");
  const [selectedLoader, setSelectedLoader] = useState("");
  const [selectedVersion, setSelectedVersion] = useState("");
  const [selectedCategories, setSelectedCategories] = useState<string[]>([]);
  const [availableCategories, setAvailableCategories] = useState<string[]>([]);
  const [showCategoryFilter, setShowCategoryFilter] = useState(false);
  
  // Pagination
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(25);
  const [totalResults, setTotalResults] = useState(0);
  
  // Selected modpack details
  const [modpackDetails, setModpackDetails] = useState<ModpackDetails | null>(null);
  const [modpackVersions, setModpackVersions] = useState<ModpackVersion[]>([]);
  const [loadingDetails, setLoadingDetails] = useState(false);
  const [selectedModpackVer, setSelectedModpackVer] = useState<ModpackVersion | null>(null);
  
  // Installation
  const [installing, setInstalling] = useState(false);
  const [installProgress, setInstallProgress] = useState(0);
  const [installStatus, setInstallStatus] = useState("");

  // Custom markdown components for external links
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

  const totalPages = Math.ceil(totalResults / pageSize);

  // Handle platform change
  const handlePlatformChange = (newPlatform: Platform) => {
    setPlatform(newPlatform);
    setSelectedModpack(null);
    setModpackDetails(null);
    setModpackVersions([]);
    setSelectedModpackVer(null);
    setModpackResults([]);
    setTotalResults(0);
    setCurrentPage(1);
    setSelectedCategories([]);
    setAvailableCategories([]);
    // Adjust page size if it exceeds the new platform's max
    const maxSize = getMaxPageSize(newPlatform);
    if (pageSize > maxSize) {
      setPageSize(maxSize);
    }
  };

  // Count active filters
  const activeFilterCount = [
    selectedLoader,
    selectedVersion,
    ...selectedCategories,
  ].filter(Boolean).length;

  // Initial search on mount
  useEffect(() => {
    if (!hasSearched) {
      searchModpacks("");
    }
  }, []);

  // Search when platform changes
  useEffect(() => {
    if (hasSearched) {
      searchModpacks(modpackSearch, 1);
    }
  }, [platform]);

  const searchModpacks = async (query: string, page = 1) => {
    setSearchingModpacks(true);
    setHasSearched(true);
    setError(null);
    
    const offset = (page - 1) * pageSize;
    
    try {
      const response = await invoke<ModpackSearchResponse>("search_modpacks", {
        query: query || "",
        platform,
        minecraftVersion: selectedVersion || null,
        modLoader: selectedLoader || null,
        categories: selectedCategories.length > 0 ? selectedCategories : null,
        clientSide: null,
        serverSide: null,
        sort: sortBy,
        offset,
        limit: pageSize,
      });
      
      // Extract categories from results
      const allCategories = new Set<string>();
      response.modpacks.forEach(pack => {
        pack.categories.forEach(cat => allCategories.add(cat));
      });
      setAvailableCategories(Array.from(allCategories).sort());
      
      setModpackResults(response.modpacks);
      setTotalResults(response.total_hits);
      setCurrentPage(page);
    } catch (err) {
      console.error("Failed to search modpacks:", err);
      setError(String(err));
      setModpackResults([]);
      setTotalResults(0);
    } finally {
      setSearchingModpacks(false);
    }
  };

  const handleSearch = () => searchModpacks(modpackSearch, 1);

  const handleSortChange = (newSort: SortOption) => {
    setSortBy(newSort);
    if (hasSearched) {
      setTimeout(() => searchModpacks(modpackSearch, 1), 0);
    }
  };

  const handlePageChange = (page: number) => {
    searchModpacks(modpackSearch, page);
  };

  const handlePageSizeChange = (newSize: number) => {
    setPageSize(newSize);
    setCurrentPage(1);
    setTimeout(() => searchModpacks(modpackSearch, 1), 0);
  };

  const toggleCategory = (category: string) => {
    setSelectedCategories(prev => 
      prev.includes(category) 
        ? prev.filter(c => c !== category)
        : [...prev, category]
    );
  };

  const clearFilters = () => {
    setSelectedLoader("");
    setSelectedVersion("");
    setSelectedCategories([]);
    setShowCategoryFilter(false);
  };

  const handleSelectModpack = async (pack: ModpackResult) => {
    setSelectedModpack(pack);
    setName(pack.name);
    setSelectedModpackVer(null);
    setModpackVersions([]);
    setModpackDetails(null);
    
    // Only load details for platforms that support it
    if (!hasDetailSupport) {
      setLoadingDetails(false);
      return;
    }
    
    setLoadingDetails(true);
    
    try {
      // Load modpack details
      const details = await invoke<ModpackDetails>("get_modpack_details", {
        modpackId: pack.id,
        platform,
      });
      setModpackDetails(details);
      
      // Load versions
      const versions = await invoke<ModpackVersion[]>("get_modpack_versions", {
        modpackId: pack.id,
        platform,
        minecraftVersion: null,
        modLoader: null,
      });
      setModpackVersions(versions);
      
      // Auto-select first version
      if (versions.length > 0) {
        setSelectedModpackVer(versions[0]);
      }
    } catch (err) {
      console.error("Failed to load modpack details:", err);
      setError(String(err));
    } finally {
      setLoadingDetails(false);
    }
  };

  const handleInstall = async () => {
    if (!selectedModpack || !selectedModpackVer?.download_url) return;
    
    setInstalling(true);
    setInstallProgress(0);
    setInstallStatus("Downloading modpack...");
    
    try {
      await invoke("import_instance_from_url", {
        url: selectedModpackVer.download_url,
        nameOverride: name || null,
      });
      
      setInstallProgress(100);
      setInstallStatus("Installation complete!");
      
      setTimeout(() => {
        navigate("/");
      }, 1500);
    } catch (err) {
      console.error("Installation failed:", err);
      setError(String(err));
      setInstalling(false);
      setInstallProgress(0);
      setInstallStatus("");
    }
  };

  return (
    <div className="flex-1 flex min-h-0">
      {/* Left Sidebar - Platform Selection */}
      <div className="w-48 flex flex-col border-r p-3 space-y-1">
        {PLATFORMS.map((p) => {
          const Logo = p.logo;
          return (
            <Button
              key={p.id}
              variant={platform === p.id ? "default" : "ghost"}
              className="justify-start gap-2"
              onClick={() => handlePlatformChange(p.id)}
            >
              <Logo />
              {p.label}
            </Button>
          );
        })}
        
        <Separator className="my-4" />
        
        {/* Name Input */}
        <div className="space-y-2">
          <Label htmlFor="name" className="text-xs">Instance Name</Label>
          <Input
            id="name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Instance name"
            className="h-8 text-sm"
          />
        </div>
        
        {/* Group Input */}
        <div className="space-y-2">
          <Label htmlFor="group" className="text-xs">Group</Label>
          <Input
            id="group"
            value={group}
            onChange={(e) => setGroup(e.target.value)}
            placeholder="Modpacks"
            className="h-8 text-sm"
          />
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
                value={modpackSearch}
                onChange={(e) => setModpackSearch(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleSearch()}
                className="pl-9"
              />
            </div>
            <Button onClick={handleSearch} disabled={searchingModpacks}>
              {searchingModpacks ? (
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
                {SORT_OPTIONS.map((opt) => (
                  <SelectItem key={opt.value} value={opt.value}>
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            
            <Button
              variant={showCategoryFilter ? "secondary" : "outline"}
              size="sm"
              className="h-8 text-xs gap-1"
              onClick={() => setShowCategoryFilter(!showCategoryFilter)}
            >
              <Filter className="h-3 w-3" />
              Filter
              {activeFilterCount > 0 && (
                <Badge variant="secondary" className="ml-1 h-4 px-1 text-[10px]">
                  {activeFilterCount}
                </Badge>
              )}
            </Button>
          </div>
          
          {/* Filter Panel */}
          {showCategoryFilter && (
            <div className="border rounded-lg p-3 bg-muted/30 space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-xs font-medium text-muted-foreground uppercase">Filters</span>
                {activeFilterCount > 0 && (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-6 text-xs px-2"
                    onClick={clearFilters}
                  >
                    Clear all
                  </Button>
                )}
              </div>
              
              {/* Mod Loader */}
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs text-muted-foreground mb-1 block">Mod Loader</label>
                  <Select value={selectedLoader} onValueChange={setSelectedLoader}>
                    <SelectTrigger className="h-8 text-xs">
                      <SelectValue placeholder="Any" />
                    </SelectTrigger>
                    <SelectContent>
                      {LOADER_OPTIONS.map((opt) => (
                        <SelectItem key={opt.value || "any"} value={opt.value}>
                          {opt.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  <label className="text-xs text-muted-foreground mb-1 block">MC Version</label>
                  <Input
                    placeholder="e.g., 1.20.1"
                    value={selectedVersion}
                    onChange={(e) => setSelectedVersion(e.target.value)}
                    className="h-8 text-xs"
                  />
                </div>
              </div>
              
              {/* Categories */}
              {availableCategories.length > 0 && (
                <>
                  <Separator />
                  <div>
                    <span className="text-xs font-medium text-muted-foreground uppercase mb-2 block">Categories</span>
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
                </>
              )}
              
              <Button 
                variant="secondary" 
                size="sm" 
                className="w-full h-7 text-xs"
                onClick={() => {
                  setShowCategoryFilter(false);
                  searchModpacks(modpackSearch, 1);
                }}
              >
                Apply Filters
              </Button>
            </div>
          )}
        </div>

        {/* Error display */}
        {error && (
          <div className="mx-4 mt-4 p-3 bg-destructive/10 border border-destructive/30 rounded-md text-destructive text-sm flex items-center justify-between">
            <span>{error}</span>
            <Button variant="ghost" size="sm" onClick={() => setError(null)}>
              <X className="h-4 w-4" />
            </Button>
          </div>
        )}

        {/* Results List */}
        <ScrollArea className="flex-1">
          <div className="p-2">
            {searchingModpacks ? (
              <div className="flex items-center justify-center py-12">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : modpackResults.length === 0 ? (
              <div className="text-center py-12 text-muted-foreground">
                {hasSearched ? "No modpacks found" : "Search for modpacks or browse popular ones"}
              </div>
            ) : (
              <div className="space-y-1">
                {modpackResults.map((pack) => (
                  <div
                    key={pack.id}
                    className={cn(
                      "flex items-start gap-3 p-3 rounded-lg cursor-pointer transition-colors",
                      selectedModpack?.id === pack.id
                        ? "bg-primary/10 border border-primary/30"
                        : "hover:bg-muted/50"
                    )}
                    onClick={() => handleSelectModpack(pack)}
                  >
                    {pack.icon_url ? (
                      <img
                        src={pack.icon_url}
                        alt={pack.name}
                        className="w-12 h-12 rounded-lg object-cover"
                      />
                    ) : (
                      <div className="w-12 h-12 rounded-lg bg-muted flex items-center justify-center">
                        <Package className="h-6 w-6 text-muted-foreground" />
                      </div>
                    )}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-medium truncate">{pack.name}</span>
                        {pack.versions.length > 0 && (
                          <Badge variant="outline" className="text-[10px] px-1.5 py-0">
                            {pack.versions[0]}
                          </Badge>
                        )}
                      </div>
                      <p className="text-xs text-muted-foreground truncate">
                        by {pack.author}
                      </p>
                      <p className="text-xs text-muted-foreground line-clamp-2 mt-1">
                        {pack.description}
                      </p>
                      <div className="flex items-center gap-3 mt-2 text-xs text-muted-foreground">
                        <span className="flex items-center gap-1">
                          <Download className="h-3 w-3" />
                          {formatNumber(pack.downloads)}
                        </span>
                        <span className="flex items-center gap-1">
                          <Heart className="h-3 w-3" />
                          {formatNumber(pack.follows)}
                        </span>
                        {pack.loaders.length > 0 && (
                          <span>{pack.loaders.slice(0, 2).join(", ")}</span>
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
        {totalResults > 0 && (
          <div className="p-3 border-t flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="text-xs text-muted-foreground">
                {totalResults.toLocaleString()} {totalResults === 1 ? "result" : "results"}
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
                disabled={currentPage === 1 || searchingModpacks}
              >
                <ChevronsLeft className="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="h-7 w-7 p-0"
                onClick={() => handlePageChange(currentPage - 1)}
                disabled={currentPage === 1 || searchingModpacks}
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
                disabled={currentPage >= totalPages || searchingModpacks}
              >
                <ChevronRight className="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="h-7 w-7 p-0"
                onClick={() => handlePageChange(totalPages)}
                disabled={currentPage >= totalPages || searchingModpacks}
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
            {loadingDetails ? (
              <div className="flex-1 flex items-center justify-center">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : hasDetailSupport && modpackDetails ? (
              <>
                {/* Header - Full details view */}
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
                            onClick={(e) => {
                              e.preventDefault();
                              openExternal(`https://${platform === "modrinth" ? "modrinth.com/modpack" : "curseforge.com/minecraft/modpacks"}/${modpackDetails.slug}`);
                            }}
                            className="ml-2 text-primary hover:underline inline-flex items-center gap-1"
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
                    <span className="flex items-center gap-1">
                      <Heart className="h-4 w-4" />
                      {formatNumber(modpackDetails.follows)} followers
                    </span>
                  </div>
                  
                  {/* External Links */}
                  <div className="flex flex-wrap gap-2 mt-3">
                    {modpackDetails.source_url && (
                      <a
                        href={modpackDetails.source_url}
                        onClick={(e) => { e.preventDefault(); openExternal(modpackDetails.source_url!); }}
                        className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                      >
                        Source <ExternalLink className="h-3 w-3" />
                      </a>
                    )}
                    {modpackDetails.issues_url && (
                      <a
                        href={modpackDetails.issues_url}
                        onClick={(e) => { e.preventDefault(); openExternal(modpackDetails.issues_url!); }}
                        className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                      >
                        Issues <ExternalLink className="h-3 w-3" />
                      </a>
                    )}
                    {modpackDetails.wiki_url && (
                      <a
                        href={modpackDetails.wiki_url}
                        onClick={(e) => { e.preventDefault(); openExternal(modpackDetails.wiki_url!); }}
                        className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                      >
                        Wiki <ExternalLink className="h-3 w-3" />
                      </a>
                    )}
                    {modpackDetails.discord_url && (
                      <a
                        href={modpackDetails.discord_url}
                        onClick={(e) => { e.preventDefault(); openExternal(modpackDetails.discord_url!); }}
                        className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
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

                {/* Version Select and Install Button */}
                <div className="p-4 border-t space-y-3">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                      Version
                    </label>
                    <Select
                      value={selectedModpackVer?.id || ""}
                      onValueChange={(v) => {
                        const version = modpackVersions.find(ver => ver.id === v);
                        setSelectedModpackVer(version || null);
                      }}
                    >
                      <SelectTrigger className="mt-1">
                        <SelectValue placeholder="Select version" />
                      </SelectTrigger>
                      <SelectContent>
                        {modpackVersions.map((version) => (
                          <SelectItem key={version.id} value={version.id}>
                            <div className="flex items-center gap-2">
                              <span>{version.version_number || version.name}</span>
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
                  
                  {selectedModpackVer && (
                    <div className="text-xs text-muted-foreground flex items-center justify-between">
                      <span>Size: {formatFileSize(selectedModpackVer.size)}</span>
                      <span>{formatDate(selectedModpackVer.date_published)}</span>
                    </div>
                  )}
                  
                  {installing ? (
                    <div className="space-y-2">
                      <Progress value={installProgress} />
                      <p className="text-sm text-center text-muted-foreground">{installStatus}</p>
                    </div>
                  ) : (
                    <Button
                      className="w-full"
                      onClick={handleInstall}
                      disabled={!selectedModpackVer?.download_url}
                    >
                      <ArrowDownCircle className="mr-2 h-4 w-4" />
                      Install Modpack
                    </Button>
                  )}
                </div>
              </>
            ) : !hasDetailSupport ? (
              // Simplified view for ATLauncher/FTB Legacy
              <>
                <div className="p-4 border-b">
                  <div className="flex items-start gap-3">
                    {selectedModpack.icon_url ? (
                      <img
                        src={selectedModpack.icon_url}
                        alt={selectedModpack.name}
                        className="w-16 h-16 rounded-lg object-cover"
                      />
                    ) : (
                      <div className="w-16 h-16 rounded-lg bg-muted flex items-center justify-center">
                        <Package className="h-8 w-8 text-muted-foreground" />
                      </div>
                    )}
                    <div className="flex-1 min-w-0">
                      <h3 className="font-semibold text-lg truncate">{selectedModpack.name}</h3>
                      <p className="text-sm text-muted-foreground">
                        by <span className="font-medium text-foreground">{selectedModpack.author || "Unknown"}</span>
                      </p>
                    </div>
                  </div>
                  
                  <div className="flex items-center gap-4 mt-3 text-sm text-muted-foreground">
                    {selectedModpack.downloads > 0 && (
                      <span className="flex items-center gap-1">
                        <Download className="h-4 w-4" />
                        {formatNumber(selectedModpack.downloads)} downloads
                      </span>
                    )}
                    {selectedModpack.follows > 0 && (
                      <span className="flex items-center gap-1">
                        <Heart className="h-4 w-4" />
                        {formatNumber(selectedModpack.follows)} followers
                      </span>
                    )}
                  </div>
                  
                  {selectedModpack.versions.length > 0 && (
                    <div className="flex flex-wrap gap-1.5 mt-3">
                      {selectedModpack.versions.slice(0, 5).map((v) => (
                        <Badge key={v} variant="outline" className="text-xs">
                          {v}
                        </Badge>
                      ))}
                    </div>
                  )}
                </div>

                {/* Description */}
                <ScrollArea className="flex-1 p-4">
                  <p className="text-sm text-muted-foreground">{selectedModpack.description}</p>
                  
                  {selectedModpack.loaders.length > 0 && (
                    <div className="mt-4">
                      <span className="text-xs font-medium text-muted-foreground uppercase">Mod Loaders</span>
                      <div className="flex flex-wrap gap-1.5 mt-1">
                        {selectedModpack.loaders.map((loader) => (
                          <Badge key={loader} variant="secondary" className="text-xs capitalize">
                            {loader}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  )}
                  
                  {selectedModpack.categories.length > 0 && (
                    <div className="mt-4">
                      <span className="text-xs font-medium text-muted-foreground uppercase">Categories</span>
                      <div className="flex flex-wrap gap-1.5 mt-1">
                        {selectedModpack.categories.map((cat) => (
                          <Badge key={cat} variant="outline" className="text-xs capitalize">
                            {cat.replace(/-/g, " ")}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  )}
                </ScrollArea>

                {/* Info for platforms without direct install */}
                <div className="p-4 border-t space-y-3">
                  <div className="p-3 bg-muted/50 rounded-lg text-sm text-muted-foreground">
                    <p className="mb-2">
                      <strong>{currentPlatformConfig?.label}</strong> modpacks don't support direct installation from this interface.
                    </p>
                    <p>
                      Visit the official {currentPlatformConfig?.label} launcher or website to download this modpack, then use the <strong>Import</strong> tab to add it.
                    </p>
                  </div>
                </div>
              </>
            ) : (
              <div className="flex-1 flex items-center justify-center text-muted-foreground">
                <p>Failed to load modpack details</p>
              </div>
            )}
          </>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground p-4">
            <Package className="h-12 w-12 mb-4 opacity-50" />
            <p className="text-center">Select a modpack to view details</p>
          </div>
        )}
      </div>
    </div>
  );
}
