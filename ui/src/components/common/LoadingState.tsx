/**
 * Reusable Loading State Component
 * 
 * A standardized loading indicator for use throughout the app.
 * Reduces code duplication from repeated loading patterns.
 */

import { RefreshCw } from "lucide-react";
import { cn } from "@/lib/utils";

interface LoadingStateProps {
  message?: string;
  className?: string;
  size?: "sm" | "md" | "lg";
}

const sizeClasses = {
  sm: "h-4 w-4",
  md: "h-6 w-6",
  lg: "h-8 w-8",
};

const textSizeClasses = {
  sm: "text-sm",
  md: "text-base",
  lg: "text-lg",
};

export function LoadingState({
  message = "Loading...",
  className,
  size = "md",
}: LoadingStateProps) {
  return (
    <div className={cn("flex items-center justify-center py-8", className)}>
      <RefreshCw className={cn("animate-spin text-muted-foreground", sizeClasses[size])} />
      <span className={cn("ml-2 text-muted-foreground", textSizeClasses[size])}>
        {message}
      </span>
    </div>
  );
}

export default LoadingState;
