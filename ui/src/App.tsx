// Main application component with routing configuration.
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

import { HashRouter as Router, Routes, Route, Outlet } from "react-router-dom";
import { Layout } from "./components/Layout";
import { ThemeProvider } from "./hooks/useTheme";
import { ConfigProvider } from "./hooks/useConfig";
import { InstancesView } from "./views/InstancesView";
import { InstanceDetailsView } from "./views/InstanceDetailsView";
import { AccountsView } from "./views/AccountsView";
import { SettingsView } from "./views/SettingsView";
import { CreateInstanceView } from "./views/CreateInstanceView";
import { NewsView } from "./views/NewsView";
import { UpdateView } from "./views/UpdateView";
import { ModpackBrowserPage, ModBrowserPage, ResourceBrowserPage, SkinManagementPage } from "./views/dialogs";

// Layout wrapper component that renders children via Outlet
function MainLayout() {
  return (
    <Layout>
      <Outlet />
    </Layout>
  );
}

function App() {
  return (
    <ConfigProvider>
      <ThemeProvider>
        <Router>
          <Routes>
            {/* Dialog pages - rendered without main Layout */}
            <Route path="/dialog/modpack-browser" element={<ModpackBrowserPage />} />
            <Route path="/dialog/mod-browser" element={<ModBrowserPage />} />
            <Route path="/dialog/resource-browser" element={<ResourceBrowserPage />} />
            <Route path="/dialog/skin-management" element={<SkinManagementPage />} />
            
            {/* Main app routes with Layout */}
            <Route element={<MainLayout />}>
              <Route path="/" element={<InstancesView />} />
              <Route path="/instance/:id" element={<InstanceDetailsView />} />
              <Route path="/create-instance" element={<CreateInstanceView />} />
              <Route path="/news" element={<NewsView />} />
              <Route path="/accounts" element={<AccountsView />} />
              <Route path="/settings" element={<SettingsView />} />
              <Route path="/update" element={<UpdateView />} />
            </Route>
          </Routes>
        </Router>
      </ThemeProvider>
    </ConfigProvider>
  );
}

export default App;
