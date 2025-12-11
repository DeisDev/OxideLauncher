import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { UserPlus, CheckCircle, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

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
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [selectedAccount, setSelectedAccount] = useState<string | null>(null);

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

  const handleRemoveAccount = async () => {
    if (!selectedAccount) return;
    try {
      await invoke("remove_account", { accountId: selectedAccount });
      loadAccounts();
    } catch (error) {
      console.error("Failed to remove account:", error);
    } finally {
      setDeleteDialogOpen(false);
      setSelectedAccount(null);
    }
  };

  const openDeleteDialog = (accountId: string) => {
    setSelectedAccount(accountId);
    setDeleteDialogOpen(true);
  };

  return (
    <div className="max-w-4xl mx-auto">
      <div className="flex justify-between items-center mb-8 pb-5 border-b border-border">
        <h1 className="text-3xl font-bold">Accounts</h1>
        <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
          <DialogTrigger asChild>
            <Button>
              <UserPlus className="mr-2 h-4 w-4" /> Add Offline Account
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Add Offline Account</DialogTitle>
              <DialogDescription>
                Create an offline account with a custom username.
              </DialogDescription>
            </DialogHeader>
            <div className="py-4">
              <Label htmlFor="username">Username</Label>
              <Input
                id="username"
                value={newUsername}
                onChange={(e) => setNewUsername(e.target.value)}
                placeholder="Enter username"
                onKeyDown={(e) => {
                  if (e.key === "Enter") addOfflineAccount();
                }}
              />
            </div>
            <DialogFooter>
              <Button variant="secondary" onClick={() => setShowAddDialog(false)}>
                Cancel
              </Button>
              <Button onClick={addOfflineAccount}>Add Account</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {accounts.length === 0 ? (
        <div className="empty-state">
          <p>No accounts found. Add an offline account to get started!</p>
        </div>
      ) : (
        <div className="space-y-4">
          {accounts.map((account) => (
            <Card
              key={account.id}
              className={`transition-all ${
                account.is_active ? "border-primary ring-1 ring-primary" : ""
              }`}
            >
              <CardContent className="flex items-center justify-between p-6">
                <div className="flex items-center gap-4">
                  <div className="h-12 w-12 rounded-full bg-muted flex items-center justify-center text-lg font-semibold">
                    {account.username.charAt(0).toUpperCase()}
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <h3 className="font-semibold text-lg">{account.username}</h3>
                      {account.is_active && (
                        <Badge variant="default">Active</Badge>
                      )}
                    </div>
                    <p className="text-sm text-muted-foreground">
                      {account.account_type}
                    </p>
                  </div>
                </div>
                <div className="flex gap-2">
                  {!account.is_active && (
                    <Button
                      variant="secondary"
                      onClick={() => setActiveAccount(account.id)}
                    >
                      <CheckCircle className="mr-2 h-4 w-4" /> Set Active
                    </Button>
                  )}
                  <Button
                    variant="destructive"
                    onClick={() => openDeleteDialog(account.id)}
                  >
                    <Trash2 className="mr-2 h-4 w-4" /> Remove
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Delete Confirmation */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Remove Account?</AlertDialogTitle>
            <AlertDialogDescription>
              This will remove the account from your launcher. You can add it back
              later if needed.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleRemoveAccount}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Remove
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
