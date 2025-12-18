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
