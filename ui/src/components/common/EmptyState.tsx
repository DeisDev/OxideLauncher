/**
 * Reusable Empty State Component
 * 
 * A standardized empty state indicator for lists and content areas.
 */

import { cn } from "@/lib/utils";
import { LucideIcon } from "lucide-react";

interface EmptyStateProps {
  message: string;
  description?: string;
  icon?: LucideIcon;
  className?: string;
  action?: React.ReactNode;
}

export function EmptyState({
  message,
  description,
  icon: Icon,
  className,
  action,
}: EmptyStateProps) {
  return (
    <div className={cn("flex flex-col items-center justify-center py-12 text-center", className)}>
      {Icon && (
        <Icon className="h-12 w-12 text-muted-foreground mb-4" />
      )}
      <p className="text-lg font-medium text-foreground">{message}</p>
      {description && (
        <p className="text-sm text-muted-foreground mt-1 max-w-md">{description}</p>
      )}
      {action && <div className="mt-4">{action}</div>}
    </div>
  );
}

export default EmptyState;
