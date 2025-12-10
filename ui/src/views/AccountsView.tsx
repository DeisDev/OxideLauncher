import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faUserPlus, faCheckCircle, faTrash } from "@fortawesome/free-solid-svg-icons";
import "./AccountsView.css";

interface AccountInfo {
  id: string;
  username: string;
  account_type: string;
  is_active: boolean;
}

export function AccountsView() {
  const [accounts, setAccounts] = useState<AccountInfo[]>([]);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [newUsername, setNewUsername] = useState("");

  useEffect(() => {
    loadAccounts();
  }, []);

  const loadAccounts = async () => {
    try {
      const data = await invoke<AccountInfo[]>("get_accounts");
      setAccounts(data);
    } catch (error) {
      console.error("Failed to load accounts:", error);
    }
  };

  const addOfflineAccount = async () => {
    if (!newUsername.trim()) return;

    try {
      await invoke("add_offline_account", { username: newUsername });
      setNewUsername("");
      setShowAddDialog(false);
      loadAccounts();
    } catch (error) {
      console.error("Failed to add account:", error);
    }
  };

  const setActiveAccount = async (id: string) => {
    try {
      await invoke("set_active_account", { accountId: id });
      loadAccounts();
    } catch (error) {
      console.error("Failed to set active account:", error);
    }
  };

  const removeAccount = async (id: string) => {
    if (confirm("Are you sure you want to remove this account?")) {
      try {
        await invoke("remove_account", { accountId: id });
        loadAccounts();
      } catch (error) {
        console.error("Failed to remove account:", error);
      }
    }
  };

  return (
    <div className="accounts-view">
      <div className="header">
        <h1>Accounts</h1>
        <button onClick={() => setShowAddDialog(true)} className="btn-success">
          <FontAwesomeIcon icon={faUserPlus} /> Add Offline Account
        </button>
      </div>

      {showAddDialog && (
        <div className="dialog-overlay" onClick={() => setShowAddDialog(false)}>
          <div className="dialog card" onClick={(e) => e.stopPropagation()}>
            <h2>Add Offline Account</h2>
            <input
              type="text"
              value={newUsername}
              onChange={(e) => setNewUsername(e.target.value)}
              placeholder="Enter username"
              autoFocus
            />
            <div className="dialog-actions">
              <button onClick={() => setShowAddDialog(false)} className="btn-secondary">
                Cancel
              </button>
              <button onClick={addOfflineAccount} className="btn-success">
                Add
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="accounts-list">
        {accounts.length === 0 ? (
          <div className="empty-state">
            <p>No accounts found. Add an offline account to get started!</p>
          </div>
        ) : (
          accounts.map((account) => (
            <div key={account.id} className={`account-card card ${account.is_active ? "active" : ""}`}>
              <div className="account-info">
                <h3>{account.username}</h3>
                <p className="account-type">{account.account_type}</p>
                {account.is_active && <span className="active-badge">Active</span>}
              </div>
              <div className="account-actions">
                {!account.is_active && (
                  <button onClick={() => setActiveAccount(account.id)}>
                    <FontAwesomeIcon icon={faCheckCircle} /> Set Active
                  </button>
                )}
                <button onClick={() => removeAccount(account.id)} className="btn-danger">
                  <FontAwesomeIcon icon={faTrash} /> Remove
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
