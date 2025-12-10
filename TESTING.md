# OxideLauncher Testing Guide

## Prerequisites

- ✅ Java 21 installed (verified: OpenJDK 21.0.6)
- ✅ Backend compiled successfully (warnings only, no errors)
- ✅ Frontend built successfully

## How to Test the Complete Flow

### 1. Launch the Application

```powershell
cd "e:\VSCODE Projects\OxideLauncher\app"
cargo tauri dev
```

Or run the release build:
```powershell
.\target\release\oxide-launcher.exe
```

### 2. Create a New Instance

1. Click the **"Create Instance"** button on the main screen
2. Fill in the details:
   - **Name**: Test Instance (or any name you prefer)
   - **Minecraft Version**: Select from dropdown (e.g., 1.21.1, 1.20.1)
   - **Mod Loader**: Choose one (Fabric, Forge, NeoForge, Quilt, or None)
3. Click **"Create"**
4. Wait for the instance to appear in your list

**What happens in the background:**
- Instance directory structure is created
- Background task downloads:
  - Version manifest
  - Version data JSON
  - Client JAR (~20-50MB)
  - Libraries (~100-300MB depending on version)
  - Native libraries (extracted to natives folder)
  - Asset index JSON
  - Assets (sounds, textures, etc. ~200-500MB)
- Total download time: 2-5 minutes depending on internet speed
- You can monitor progress in the terminal/console output

### 3. Search and Install Mods

1. Click on your instance to open the **Instance Details View**
2. Click the **"Mods"** tab in the sidebar
3. In the search bar, enter a mod name (e.g., "jei", "sodium", "iris")
4. Click **"Search"**
5. Browse the results and click **"Install"** on any mod you want
6. The mod will be downloaded to your instance's mods folder
7. You'll see it appear in the **"Installed Mods"** list below

**Mod Management Features:**
- **Toggle**: Click the toggle icon to enable/disable a mod (adds `.disabled` extension)
- **Delete**: Click the trash icon to remove a mod

### 4. Launch Minecraft

1. Go back to the main instances view
2. Find your instance and click **"Launch"** or click the play button
3. The game will start launching

**What happens:**
- Java process is spawned with proper arguments
- Classpath is built from all libraries
- Native libraries path is set
- Offline authentication session is created (username: "Player")
- Game arguments are constructed
- Minecraft starts

### 5. Monitor Console Logs

1. While the game is launching/running, go to **Instance Details → Minecraft Log tab**
2. You should see real-time console output including:
   - `[main/INFO]: Loading Minecraft...`
   - `[main/INFO]: Starting game version X.XX.X`
   - `[Render thread/INFO]: Backend library: LWJGL version X.X.X`
   - Mod loading messages (if using mod loader)
   - Any errors or warnings

**Log Viewer Features:**
- **Auto-scroll**: Automatically scrolls to bottom as new logs arrive
- **Wrap lines**: Toggle line wrapping for long log lines
- **Search**: Filter logs by keyword
- **Copy**: Copy all logs to clipboard
- **Clear**: Clear the log viewer (doesn't affect actual game)

### 6. Verify Game Launched Successfully

**Expected Results:**
- ✅ Minecraft window opens
- ✅ Main menu appears
- ✅ Console logs show successful initialization
- ✅ No crashes or critical errors
- ✅ Mods are loaded (if installed and mod loader is used)

**Offline Mode Notice:**
- Since we don't have authentication yet, you're playing in offline mode
- Username will be "Player"
- You can play singleplayer worlds
- Multiplayer servers that require online authentication won't work

## Troubleshooting

### Instance Not Launching

**Check:**
1. Java is installed: `java -version`
2. Download completed successfully (check terminal output)
3. Console logs for errors (Instance Details → Minecraft Log)

**Common Issues:**
- **"No Java installation found"**: Install Java 17+ from Adoptium or Oracle
- **Missing files**: Delete instance and recreate to re-download files
- **Port conflicts**: Close other Minecraft instances

### Mods Not Loading

**Check:**
1. Correct mod loader is selected (Fabric mods need Fabric, Forge mods need Forge)
2. Mod is compatible with your Minecraft version
3. Mod is enabled (not showing strikethrough in installed mods list)
4. No mod conflicts (check console logs for errors)

### Downloads Failing

**Check:**
1. Internet connection is stable
2. Firewall isn't blocking the application
3. Sufficient disk space (~1-2GB per instance)
4. Check terminal output for download error messages

## Test Checklist

- [ ] Create instance with Vanilla Minecraft
- [ ] Create instance with Fabric mod loader
- [ ] Observe downloads in console/terminal
- [ ] Search for mods (e.g., "sodium")
- [ ] Install a mod
- [ ] Toggle mod enabled/disabled
- [ ] Launch instance
- [ ] View real-time console logs
- [ ] Verify game window opens
- [ ] Check mods are loaded (if applicable)
- [ ] Play for a few minutes to ensure stability

## Known Limitations (Current Version)

- ❌ No authentication system (offline mode only)
- ❌ No multiplayer with online-mode servers
- ❌ No mod version selection (downloads latest compatible)
- ❌ No progress bars for downloads (terminal only)
- ❌ No automatic Java installation
- ❌ Basic error handling (alerts instead of graceful recovery)

## Next Steps for Full Implementation

1. Implement Microsoft/Mojang authentication
2. Add download progress UI
3. Add Java runtime detection and installation
4. Improve error handling with user-friendly messages
5. Add mod dependency resolution
6. Support CurseForge mods (currently Modrinth only)
7. Add resource pack and shader pack management
8. Implement world backup/restore
9. Add screenshot viewer
10. Implement launch wrapper for better process control
