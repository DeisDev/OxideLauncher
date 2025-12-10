import { ReactNode } from "react";
import { Link, useLocation } from "react-router-dom";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faGamepad, faUser, faCog } from "@fortawesome/free-solid-svg-icons";
import "./Layout.css";

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const location = useLocation();

  const isActive = (path: string) => {
    return location.pathname === path ? "active" : "";
  };

  return (
    <div className="layout">
      <nav className="sidebar">
        <div className="logo">
          <h2>OxideLauncher</h2>
        </div>
        <ul className="nav-links">
          <li>
            <Link to="/" className={isActive("/")}>
              <FontAwesomeIcon icon={faGamepad} className="icon" />
              Instances
            </Link>
          </li>
          <li>
            <Link to="/accounts" className={isActive("/accounts")}>
              <FontAwesomeIcon icon={faUser} className="icon" />
              Accounts
            </Link>
          </li>
          <li>
            <Link to="/settings" className={isActive("/settings")}>
              <FontAwesomeIcon icon={faCog} className="icon" />
              Settings
            </Link>
          </li>
        </ul>
      </nav>
      <main className="content">{children}</main>
    </div>
  );
}
