// News feed view component displaying launcher and Minecraft updates
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

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Newspaper, ExternalLink, Calendar, RefreshCw, User, AlertCircle } from "lucide-react";
import { Button } from "@/components/ui/button";

// Type for news categories
type NewsCategory = "minecraft" | "launcher" | "mods" | "community";

interface NewsItem {
  id: string;
  slug: string;
  title: string;
  summary: string;
  date: string;
  category: NewsCategory;
  author: string;
}

const CATEGORY_COLORS: Record<NewsCategory, string> = {
  minecraft: "bg-green-500/10 text-green-500 border-green-500/20",
  launcher: "bg-orange-500/10 text-orange-500 border-orange-500/20",
  mods: "bg-purple-500/10 text-purple-500 border-purple-500/20",
  community: "bg-blue-500/10 text-blue-500 border-blue-500/20",
};

const CATEGORY_LABELS: Record<NewsCategory, string> = {
  minecraft: "Minecraft",
  launcher: "Launcher",
  mods: "Mods",
  community: "Community",
};

function formatDate(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "Yesterday";
  if (diffDays < 7) return `${diffDays} days ago`;
  
  return date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: date.getFullYear() !== now.getFullYear() ? "numeric" : undefined,
  });
}

export function NewsView() {
  const [news, setNews] = useState<NewsItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [skeletonCount, setSkeletonCount] = useState(1); // Default to 1 skeleton

  useEffect(() => {
    loadNews();
  }, []);

  const loadNews = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const articles = await invoke<NewsItem[]>("get_news");
      setNews(articles);
      // Update skeleton count for next refresh (at least 1, cap at 5)
      setSkeletonCount(Math.max(1, Math.min(articles.length, 5)));
    } catch (err) {
      console.error("Failed to load news:", err);
      setError(err instanceof Error ? err.message : String(err));
      setNews([]);
    } finally {
      setLoading(false);
    }
  };

  const openArticle = async (slug: string) => {
    try {
      // Website uses hash routing: /#/news/{slug}
      await invoke("open_external_url", { url: `https://oxidelauncher.org/#/news/${slug}` });
    } catch (err) {
      console.error("Failed to open article:", err);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg bg-primary/10">
            <Newspaper className="h-6 w-6 text-primary" />
          </div>
          <div>
            <h1 className="text-2xl font-bold">News</h1>
            <p className="text-muted-foreground">Stay updated with the latest news and announcements</p>
          </div>
        </div>
        <Button variant="outline" size="sm" onClick={loadNews} disabled={loading}>
          <RefreshCw className={`mr-2 h-4 w-4 ${loading ? "animate-spin" : ""}`} />
          Refresh
        </Button>
      </div>

      {/* News Feed */}
      {loading ? (
        <div className="grid gap-4">
          {Array.from({ length: skeletonCount }, (_, i) => i + 1).map((i) => (
            <Card key={i} className="animate-pulse">
              <CardHeader className="pb-3">
                <div className="flex items-start justify-between gap-4">
                  <div className="space-y-2 flex-1">
                    <div className="h-5 bg-muted rounded w-3/4" />
                    <div className="flex items-center gap-3">
                      <div className="h-3.5 bg-muted rounded w-20" />
                      <div className="h-3.5 bg-muted rounded w-16" />
                    </div>
                  </div>
                  <div className="h-5 bg-muted rounded w-20" />
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  <div className="h-4 bg-muted rounded w-full" />
                  <div className="h-4 bg-muted rounded w-4/5" />
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : error ? (
        <Card className="border-destructive/50">
          <CardContent className="py-8 text-center">
            <AlertCircle className="h-10 w-10 text-destructive mx-auto mb-4" />
            <p className="text-destructive font-medium mb-2">Failed to load news</p>
            <p className="text-muted-foreground text-sm mb-4">{error}</p>
            <Button variant="outline" size="sm" onClick={loadNews}>
              <RefreshCw className="mr-2 h-4 w-4" />
              Try again
            </Button>
          </CardContent>
        </Card>
      ) : news.length === 0 ? (
        <Card className="border-dashed border-muted-foreground/30">
          <CardContent className="py-8 text-center">
            <Newspaper className="h-10 w-10 text-muted-foreground mx-auto mb-4" />
            <p className="text-muted-foreground text-sm">
              No news articles available at this time.
              <br />
              Check back later for updates!
            </p>
          </CardContent>
        </Card>
      ) : (
        <ScrollArea className="h-[calc(100vh-200px)]">
          <div className="grid gap-4 pr-4">
            {news.map((item) => (
              <Card 
                key={item.id} 
                className="hover:border-primary/50 transition-colors cursor-pointer"
                onClick={() => openArticle(item.slug)}
              >
                <CardHeader className="pb-3">
                  <div className="flex items-start justify-between gap-4">
                    <div className="space-y-1">
                      <CardTitle className="text-lg leading-tight">{item.title}</CardTitle>
                      <div className="flex items-center gap-3 text-sm text-muted-foreground">
                        <div className="flex items-center gap-1.5">
                          <Calendar className="h-3.5 w-3.5" />
                          <span>{formatDate(item.date)}</span>
                        </div>
                        <div className="flex items-center gap-1.5">
                          <User className="h-3.5 w-3.5" />
                          <span>{item.author}</span>
                        </div>
                      </div>
                    </div>
                    <Badge 
                      variant="outline" 
                      className={CATEGORY_COLORS[item.category] || CATEGORY_COLORS.community}
                    >
                      {CATEGORY_LABELS[item.category] || item.category}
                    </Badge>
                  </div>
                </CardHeader>
                <CardContent>
                  <CardDescription className="text-sm leading-relaxed">
                    {item.summary}
                  </CardDescription>
                  <div className="inline-flex items-center gap-1 mt-3 text-sm text-primary hover:underline">
                    Read more
                    <ExternalLink className="h-3.5 w-3.5" />
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </ScrollArea>
      )}
    </div>
  );
}
