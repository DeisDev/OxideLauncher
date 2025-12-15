import { useState, useEffect } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Newspaper, ExternalLink, Calendar, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";

interface NewsItem {
  id: string;
  title: string;
  description: string;
  date: string;
  category: "minecraft" | "launcher" | "mods" | "community";
  url?: string;
  image?: string;
}

// Placeholder news data until real API integration
const PLACEHOLDER_NEWS: NewsItem[] = [
  {
    id: "1",
    title: "Welcome to Oxide Launcher!",
    description: "Oxide Launcher is a modern, fast, and feature-rich Minecraft launcher built with Rust and React. We're constantly working on new features and improvements.",
    date: new Date().toISOString(),
    category: "launcher",
  },
  {
    id: "2", 
    title: "Multi-Instance Support",
    description: "Create and manage multiple Minecraft instances with different versions, mods, and configurations. Each instance is isolated and can be customized independently.",
    date: new Date(Date.now() - 86400000).toISOString(),
    category: "launcher",
  },
  {
    id: "3",
    title: "Modloader Support Coming Soon",
    description: "Full support for Forge, Fabric, Quilt, and NeoForge modloaders is in development. Stay tuned for updates!",
    date: new Date(Date.now() - 172800000).toISOString(),
    category: "mods",
  },
  {
    id: "4",
    title: "Blazingly Fast Downloads 🦀",
    description: "Oxide Launcher leverages Rust's async runtime and parallel downloads for lightning-fast game downloads. Memory safe AND fast!",
    date: new Date(Date.now() - 259200000).toISOString(),
    category: "launcher",
  },
];

const CATEGORY_COLORS: Record<NewsItem["category"], string> = {
  minecraft: "bg-green-500/10 text-green-500 border-green-500/20",
  launcher: "bg-orange-500/10 text-orange-500 border-orange-500/20",
  mods: "bg-purple-500/10 text-purple-500 border-purple-500/20",
  community: "bg-blue-500/10 text-blue-500 border-blue-500/20",
};

const CATEGORY_LABELS: Record<NewsItem["category"], string> = {
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

  useEffect(() => {
    loadNews();
  }, []);

  const loadNews = async () => {
    setLoading(true);
    // TODO: Replace with actual API call when news service is implemented
    // For now, use placeholder data
    await new Promise(resolve => setTimeout(resolve, 500)); // Simulate network delay
    setNews(PLACEHOLDER_NEWS);
    setLoading(false);
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
          {[1, 2, 3].map((i) => (
            <Card key={i}>
              <CardHeader>
                <div className="skeleton h-6 w-3/4 rounded" />
                <div className="skeleton h-4 w-24 rounded mt-2" />
              </CardHeader>
              <CardContent>
                <div className="skeleton h-4 w-full rounded mb-2" />
                <div className="skeleton h-4 w-4/5 rounded" />
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <ScrollArea className="h-[calc(100vh-200px)]">
          <div className="grid gap-4 pr-4">
            {news.map((item) => (
              <Card key={item.id} className="hover:border-primary/50 transition-colors">
                <CardHeader className="pb-3">
                  <div className="flex items-start justify-between gap-4">
                    <div className="space-y-1">
                      <CardTitle className="text-lg leading-tight">{item.title}</CardTitle>
                      <div className="flex items-center gap-2 text-sm text-muted-foreground">
                        <Calendar className="h-3.5 w-3.5" />
                        <span>{formatDate(item.date)}</span>
                      </div>
                    </div>
                    <Badge 
                      variant="outline" 
                      className={CATEGORY_COLORS[item.category]}
                    >
                      {CATEGORY_LABELS[item.category]}
                    </Badge>
                  </div>
                </CardHeader>
                <CardContent>
                  <CardDescription className="text-sm leading-relaxed">
                    {item.description}
                  </CardDescription>
                  {item.url && (
                    <a 
                      href={item.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="inline-flex items-center gap-1 mt-3 text-sm text-primary hover:underline"
                    >
                      Read more
                      <ExternalLink className="h-3.5 w-3.5" />
                    </a>
                  )}
                </CardContent>
              </Card>
            ))}

            {/* Placeholder notice */}
            <Card className="border-dashed border-muted-foreground/30">
              <CardContent className="py-8 text-center">
                <p className="text-muted-foreground text-sm">
                  📰 News feed is currently showing placeholder content.
                  <br />
                  Real news integration coming soon!
                </p>
              </CardContent>
            </Card>
          </div>
        </ScrollArea>
      )}
    </div>
  );
}
