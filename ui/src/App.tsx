import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { Layout } from "./components/Layout";
import { InstancesView } from "./views/InstancesView";
import { InstanceDetailsView } from "./views/InstanceDetailsView";
import { InstanceSettingsView } from "./views/InstanceSettingsView";
import { AccountsView } from "./views/AccountsView";
import { SettingsView } from "./views/SettingsView";
import { CreateInstanceView } from "./views/CreateInstanceView";

function App() {
  return (
    <Router>
      <Layout>
        <Routes>
          <Route path="/" element={<InstancesView />} />
          <Route path="/instance/:id" element={<InstanceDetailsView />} />
          <Route path="/instance/:id/settings" element={<InstanceSettingsView />} />
          <Route path="/create-instance" element={<CreateInstanceView />} />
          <Route path="/accounts" element={<AccountsView />} />
          <Route path="/settings" element={<SettingsView />} />
        </Routes>
      </Layout>
    </Router>
  );
}

export default App;
