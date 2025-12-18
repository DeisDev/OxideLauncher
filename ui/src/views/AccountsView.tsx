import { useEffect, useState, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  UserPlus,
  CheckCircle,
  Trash2,
  RefreshCw,
  Loader2,
  Copy,
  ExternalLink,
  AlertCircle,
  Palette,
} from "lucide-react";
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
import { Alert, AlertDescription } from "@/components/ui/alert";
import { openDialogWindow, WINDOW_LABELS } from "@/lib/windowManager";
import { AccountInfo, DeviceCodeInfo, AuthProgressEventType } from "@/types";

export function AccountsView() {
  const [accounts, setAccounts] = useState<AccountInfo[]>([]);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [showMsaDialog, setShowMsaDialog] = useState(false);
  const [newUsername, setNewUsername] = useState("");
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [refreshDialogOpen, setRefreshDialogOpen] = useState(false);
  const [selectedAccount, setSelectedAccount] = useState<string | null>(null);
  const [selectedAccountForRefresh, setSelectedAccountForRefresh] = useState<AccountInfo | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [refreshingAccount, setRefreshingAccount] = useState<string | null>(null);
  const [isMsaConfigured, setIsMsaConfigured] = useState(false);

  // Microsoft login state
  const [deviceCode, setDeviceCode] = useState<DeviceCodeInfo | null>(null);
  const [msaStatus, setMsaStatus] = useState<string>("");
  const [msaError, setMsaError] = useState<string | null>(null);
  const [isPolling, setIsPolling] = useState(false);
  const [codeCopied, setCodeCopied] = useState(false);
  const pollingIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const deviceCodeRef = useRef<string | null>(null);

  useEffect(() => {
    loadAccounts();
    checkMsaConfigured();

    // Listen for auth progress events
    const unlisten = listen<AuthProgressEventType>("auth_progress", (event) => {
      const data = event.payload;
      if (data.type === "StepStarted") {
        setMsaStatus(data.data.description);
      } else if (data.type === "PollingForAuth") {
        setMsaStatus(data.data.message);
      } else if (data.type === "StepCompleted") {
        setMsaStatus(`Completed: ${data.data.step}`);
      } else if (data.type === "Completed") {
        setMsaStatus(`Welcome, ${data.data.username}!`);
      } else if (data.type === "Failed") {
        setMsaError(`${data.data.step}: ${data.data.error}`);
      }
    });

    // Listen for account updates from skin management window
    const unlistenAccountUpdated = listen("account-updated", () => {
      loadAccounts();
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenAccountUpdated.then((fn) => fn());
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current);
      }
    };
  }, []);

  const loadAccounts = async () => {
    try {
      const data = await invoke<AccountInfo[]>("get_accounts");
      setAccounts(data);
    } catch (error) {
      console.error("Failed to load accounts:", error);
      setError(String(error));
    }
  };

  const checkMsaConfigured = async () => {
    try {
      const configured = await invoke<boolean>("is_microsoft_configured");
      setIsMsaConfigured(configured);
    } catch (error) {
      console.error("Failed to check MSA configuration:", error);
    }
  };

  const addOfflineAccount = async () => {
    if (!newUsername.trim()) return;

    setIsLoading(true);
    setError(null);

    try {
      await invoke("add_offline_account", { username: newUsername });
      setNewUsername("");
      setShowAddDialog(false);
      loadAccounts();
    } catch (error) {
      setError(String(error));
    } finally {
      setIsLoading(false);
    }
  };

  const startMicrosoftLogin = async () => {
    setIsLoading(true);
    setMsaError(null);
    setMsaStatus("Starting Microsoft login...");
    setDeviceCode(null);
    setCodeCopied(false);

    try {
      const code = await invoke<DeviceCodeInfo>("start_microsoft_login");
      console.log("Received device code from backend:", {
        device_code_length: code.device_code.length,
        device_code_preview: code.device_code.substring(0, 8),
        user_code: code.user_code,
      });
      setDeviceCode(code);
      deviceCodeRef.current = code.user_code;
      setMsaStatus("Please enter the code in your browser");

      // Start polling
      setIsPolling(true);
      startPolling(code);
    } catch (error) {
      setMsaError(String(error));
      setIsLoading(false);
    }
  };

  const startPolling = useCallback((code: DeviceCodeInfo) => {
    if (pollingIntervalRef.current) {
      clearInterval(pollingIntervalRef.current);
    }

    // Poll at the recommended interval (minimum 5 seconds)
    const interval = Math.max(code.interval, 5) * 1000;
    
    console.log("Starting polling with device_code:", {
      length: code.device_code.length,
      preview: code.device_code.substring(0, 8),
      interval: interval,
    });

    pollingIntervalRef.current = setInterval(async () => {
      console.log("Polling with deviceCode:", code.device_code.substring(0, 8));
      try {
        const result = await invoke<AccountInfo | null>("poll_microsoft_login", {
          deviceCode: code.device_code,
        });

        if (result) {
          // Success!
          if (pollingIntervalRef.current) {
            clearInterval(pollingIntervalRef.current);
            pollingIntervalRef.current = null;
          }
          setIsPolling(false);
          setIsLoading(false);
          setShowMsaDialog(false);
          setDeviceCode(null);
          setMsaStatus("");
          loadAccounts();
        }
      } catch (error) {
        // Check if it's a terminal error
        const errorStr = String(error);
        if (
          errorStr.includes("declined") ||
          errorStr.includes("expired") ||
          errorStr.includes("not found")
        ) {
          if (pollingIntervalRef.current) {
            clearInterval(pollingIntervalRef.current);
            pollingIntervalRef.current = null;
          }
          setMsaError(errorStr);
          setIsPolling(false);
          setIsLoading(false);
        }
      }
    }, interval);
  }, []);

  const cancelMicrosoftLogin = async () => {
    if (pollingIntervalRef.current) {
      clearInterval(pollingIntervalRef.current);
      pollingIntervalRef.current = null;
    }

    if (deviceCode) {
      try {
        await invoke("cancel_microsoft_login", { deviceCode: deviceCode.device_code });
      } catch (e) {
        // Ignore errors when canceling
      }
    }

    setIsPolling(false);
    setIsLoading(false);
    setDeviceCode(null);
    setMsaError(null);
    setMsaStatus("");
    setShowMsaDialog(false);
  };

  const copyCode = async () => {
    if (deviceCode) {
      await navigator.clipboard.writeText(deviceCode.user_code);
      setCodeCopied(true);
      setTimeout(() => setCodeCopied(false), 2000);
    }
  };

  const openVerificationUrl = () => {
    if (deviceCode) {
      window.open(deviceCode.verification_uri, "_blank");
    }
  };

  const openRefreshDialog = (account: AccountInfo) => {
    setSelectedAccountForRefresh(account);
    setRefreshDialogOpen(true);
  };

  const handleRefreshAccount = async () => {
    if (!selectedAccountForRefresh) return;
    
    setRefreshDialogOpen(false);
    setRefreshingAccount(selectedAccountForRefresh.id);
    setError(null);

    try {
      await invoke("refresh_account", { accountId: selectedAccountForRefresh.id });
      loadAccounts();
    } catch (error) {
      setError(String(error));
    } finally {
      setRefreshingAccount(null);
      setSelectedAccountForRefresh(null);
    }
  };

  const setActiveAccount = async (id: string) => {
    try {
      await invoke("set_active_account", { accountId: id });
      loadAccounts();
    } catch (error) {
      setError(String(error));
    }
  };

  const handleRemoveAccount = async () => {
    if (!selectedAccount) return;
    try {
      await invoke("remove_account", { accountId: selectedAccount });
      loadAccounts();
    } catch (error) {
      setError(String(error));
    } finally {
      setDeleteDialogOpen(false);
      setSelectedAccount(null);
    }
  };

  const openDeleteDialog = (accountId: string) => {
    setSelectedAccount(accountId);
    setDeleteDialogOpen(true);
  };

  const openSkinDialog = async (account: AccountInfo) => {
    await openDialogWindow(WINDOW_LABELS.SKIN_MANAGEMENT, {
      accountId: account.id,
      username: account.username,
      uuid: account.uuid,
      accountType: account.account_type,
    });
  };

  const getSkinAvatar = (account: AccountInfo) => {
    if (account.skin_url) {
      // Minecraft skin URLs point to the full skin, we need the face
      // The face is at position (8,8) with size 8x8 in the skin texture
      // For simplicity, we'll just show a colored avatar based on username
    }
    return null;
  };

  return (
    <div className="w-full">
      <div className="flex flex-wrap justify-between items-center gap-4 mb-8 pb-5 border-b border-border">
        <h1 className="text-3xl font-bold">Accounts</h1>
        <div className="flex flex-wrap gap-2">
          {/* Microsoft Login Button */}
          <Dialog open={showMsaDialog} onOpenChange={(open) => {
            if (!open && isPolling) {
              cancelMicrosoftLogin();
            } else {
              setShowMsaDialog(open);
            }
          }}>
            <DialogTrigger asChild>
              <Button
                variant="default"
                disabled={!isMsaConfigured}
                title={!isMsaConfigured ? "Microsoft Client ID not configured" : undefined}
              >
                <UserPlus className="mr-2 h-4 w-4" /> Add Microsoft Account
              </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-md">
              <DialogHeader>
                <DialogTitle>Sign in with Microsoft</DialogTitle>
                <DialogDescription>
                  {!deviceCode
                    ? "Click below to start the sign-in process."
                    : "Enter the code below at the Microsoft website to complete sign-in."}
                </DialogDescription>
              </DialogHeader>

              {msaError && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{msaError}</AlertDescription>
                </Alert>
              )}

              {!deviceCode ? (
                <div className="flex flex-col items-center gap-4 py-4">
                  <p className="text-sm text-muted-foreground text-center">
                    You'll be asked to sign in with your Microsoft account that owns
                    Minecraft Java Edition.
                  </p>
                  <Button onClick={startMicrosoftLogin} disabled={isLoading}>
                    {isLoading ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        Starting...
                      </>
                    ) : (
                      "Start Sign In"
                    )}
                  </Button>
                </div>
              ) : (
                <div className="flex flex-col items-center gap-4 py-4">
                  <div className="text-center">
                    <p className="text-sm text-muted-foreground mb-2">
                      Go to{" "}
                      <button
                        onClick={openVerificationUrl}
                        className="text-primary hover:underline inline-flex items-center gap-1"
                      >
                        {deviceCode.verification_uri}
                        <ExternalLink className="h-3 w-3" />
                      </button>
                    </p>
                    <p className="text-sm text-muted-foreground mb-4">
                      and enter this code:
                    </p>
                  </div>

                  <div className="flex items-center gap-2">
                    <code className="text-3xl font-mono font-bold tracking-wider bg-muted px-4 py-2 rounded-lg">
                      {deviceCode.user_code}
                    </code>
                    <Button
                      variant="outline"
                      size="icon"
                      onClick={copyCode}
                      title="Copy code"
                    >
                      {codeCopied ? (
                        <CheckCircle className="h-4 w-4 text-green-500" />
                      ) : (
                        <Copy className="h-4 w-4" />
                      )}
                    </Button>
                  </div>

                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    <span>{msaStatus || "Waiting for authentication..."}</span>
                  </div>

                  <p className="text-xs text-muted-foreground">
                    Code expires in {Math.floor(deviceCode.expires_in / 60)} minutes
                  </p>
                </div>
              )}

              <DialogFooter>
                <Button variant="secondary" onClick={cancelMicrosoftLogin}>
                  Cancel
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>

          {/* Offline Account Button */}
          <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
            <DialogTrigger asChild>
              <Button variant="outline">
                <UserPlus className="mr-2 h-4 w-4" /> Add Offline Account
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Add Offline Account</DialogTitle>
                <DialogDescription>
                  Create an offline account with a custom username. This won't let you
                  play on online servers.
                </DialogDescription>
              </DialogHeader>
              {error && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}
              <div className="py-4">
                <Label htmlFor="username">Username</Label>
                <Input
                  id="username"
                  value={newUsername}
                  onChange={(e) => setNewUsername(e.target.value)}
                  placeholder="Enter username (3-16 characters)"
                  onKeyDown={(e) => {
                    if (e.key === "Enter") addOfflineAccount();
                  }}
                />
                <p className="text-xs text-muted-foreground mt-1">
                  Only letters, numbers, and underscores allowed
                </p>
              </div>
              <DialogFooter>
                <Button variant="secondary" onClick={() => setShowAddDialog(false)}>
                  Cancel
                </Button>
                <Button onClick={addOfflineAccount} disabled={isLoading}>
                  {isLoading ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      Adding...
                    </>
                  ) : (
                    "Add Account"
                  )}
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
      </div>

      {/* MSA not configured warning */}
      {!isMsaConfigured && (
        <Alert className="mb-4">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            Microsoft login is not configured. Add your Microsoft Azure Client ID in Settings
            to enable Microsoft account login.
          </AlertDescription>
        </Alert>
      )}

      {error && !showAddDialog && (
        <Alert variant="destructive" className="mb-4">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {accounts.length === 0 ? (
        <div className="empty-state text-center py-12">
          <p className="text-muted-foreground">
            No accounts found. Add an account to get started!
          </p>
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
                  <div className="h-12 w-12 rounded-full bg-muted flex items-center justify-center text-lg font-semibold overflow-hidden">
                    {getSkinAvatar(account) || account.username.charAt(0).toUpperCase()}
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <h3 className="font-semibold text-lg">{account.username}</h3>
                      {account.is_active && (
                        <Badge className="bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30 hover:bg-emerald-500/20">Active</Badge>
                      )}
                      {account.account_type === "Microsoft" && !account.is_valid && (
                        <Badge variant="destructive">Expired</Badge>
                      )}
                      {account.account_type === "Microsoft" &&
                        account.is_valid &&
                        account.needs_refresh && (
                          <Badge className="bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/30">Needs Refresh</Badge>
                        )}
                    </div>
                    <p className="text-sm">
                      <span className={account.account_type === "Microsoft" ? "text-blue-600 dark:text-blue-400" : "text-muted-foreground"}>
                        {account.account_type}
                      </span>
                      {account.uuid && (
                        <span className="text-xs ml-2 text-muted-foreground opacity-50">
                          {account.uuid.substring(0, 8)}...
                        </span>
                      )}
                    </p>
                  </div>
                </div>
                <div className="flex gap-2">
                  {account.account_type === "Microsoft" && (
                    <>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => openSkinDialog(account)}
                        title="Manage skin"
                      >
                        <Palette className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => openRefreshDialog(account)}
                        disabled={refreshingAccount === account.id}
                        title="Refresh account tokens"
                      >
                        {refreshingAccount === account.id ? (
                          <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                          <RefreshCw className="h-4 w-4" />
                        )}
                      </Button>
                    </>
                  )}
                  {!account.is_active && (
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => setActiveAccount(account.id)}
                    >
                      <CheckCircle className="mr-2 h-4 w-4" /> Set Active
                    </Button>
                  )}
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={() => openDeleteDialog(account.id)}
                  >
                    <Trash2 className="h-4 w-4" />
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

      {/* Refresh Token Confirmation */}
      <AlertDialog open={refreshDialogOpen} onOpenChange={setRefreshDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Refresh Account Tokens?</AlertDialogTitle>
            <AlertDialogDescription>
              This will re-authenticate with Microsoft servers to refresh the access tokens for{" "}
              <span className="font-semibold">{selectedAccountForRefresh?.username}</span>.
              This is needed if your tokens have expired.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleRefreshAccount}>
              Refresh Tokens
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
