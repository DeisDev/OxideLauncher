import { ReactNode } from "react";
import { Link, useLocation } from "react-router-dom";
import { Gamepad2, User, Settings } from "lucide-react";
import { cn } from "@/lib/utils";

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const location = useLocation();

  const navItems = [
    { path: "/", icon: Gamepad2, label: "Instances" },
    { path: "/accounts", icon: User, label: "Accounts" },
    { path: "/settings", icon: Settings, label: "Settings" },
  ];

  return (
    <div className="flex h-screen w-screen overflow-hidden">
      <nav className="w-56 bg-card border-r border-border flex flex-col flex-shrink-0">
        <div className="p-6 border-b border-border">
          <h2 className="text-xl font-bold bg-gradient-to-r from-primary to-green-400 bg-clip-text text-transparent">
            OxideLauncher
          </h2>
        </div>
        <ul className="flex-1 p-3 space-y-1">
          {navItems.map((item) => (
            <li key={item.path}>
              <Link
                to={item.path}
                className={cn(
                  "flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-all",
                  location.pathname === item.path
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                )}
              >
                <item.icon className="h-5 w-5" />
                {item.label}
              </Link>
            </li>
          ))}
        </ul>
      </nav>
      <main className="flex-1 overflow-auto">
        <div key={location.pathname} className="page-transition view-container h-full p-8">
          {children}
        </div>
      </main>
    </div>
  );
}
