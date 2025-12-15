import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
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

function App() {
  return (
    <ConfigProvider>
      <ThemeProvider>
        <Router>
          <Layout>
            <Routes>
              <Route path="/" element={<InstancesView />} />
              <Route path="/instance/:id" element={<InstanceDetailsView />} />
              <Route path="/create-instance" element={<CreateInstanceView />} />
              <Route path="/news" element={<NewsView />} />
              <Route path="/accounts" element={<AccountsView />} />
              <Route path="/settings" element={<SettingsView />} />
              <Route path="/update" element={<UpdateView />} />
            </Routes>
          </Layout>
        </Router>
      </ThemeProvider>
    </ConfigProvider>
  );
}

export default App;
