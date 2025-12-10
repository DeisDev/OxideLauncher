# OxideLauncher - Tauri + Bun Architecture

## Overview

OxideLauncher has been refactored from Iced to use **Tauri + Bun + React**, providing a web-based frontend with a Rust backend.

## Architecture

### Backend (Rust + Tauri)
- **Location**: `app/`
- **Framework**: Tauri 2.1
- **Purpose**: Core logic, file system operations, Minecraft launching, account management

### Frontend (React + TypeScript)
- **Location**: `ui/`
- **Runtime**: Bun
- **Bundler**: Vite
- **Framework**: React 18 with React Router
- **Purpose**: User interface, rendering views, state management

## Project Structure

```
OxideLauncher/
├── app/                          # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs              # Tauri entry point
│   │   ├── commands.rs          # Tauri command handlers (API)
│   │   └── core/                # Business logic
│   │       ├── accounts/        # Account management
│   │       ├── config/          # Configuration
│   │   ├── instance/        # Instance management
│   │       ├── minecraft/       # Minecraft version/assets
│   │       └── ...
│   ├── Cargo.toml
│   ├── build.rs                 # Tauri build script
│   └── tauri.conf.json          # Tauri configuration
│
└── ui/                           # React frontend
    ├── src/
    │   ├── main.tsx             # React entry point
    │   ├── App.tsx              # App router
    │   ├── components/          # Reusable components
    │   │   └── Layout.tsx       # Main layout with sidebar
    │   ├── views/               # Page views
    │   │   ├── InstancesView.tsx
    │   │   ├── AccountsView.tsx
    │   │   ├── SettingsView.tsx
    │   │   └── CreateInstanceView.tsx
    │   └── styles/              # CSS stylesheets
    ├── package.json
    ├── vite.config.ts
    └── tsconfig.json
```

## Development

### Prerequisites
- Rust 1.70+ (for Tauri backend)
- Bun 1.0+ (for frontend)
- Node.js (optional, Bun is preferred)

### Setup

1. **Install Rust dependencies**:
```bash
cd app
cargo build
```

2. **Install frontend dependencies**:
```bash
cd ui
bun install
```

### Running in Development

From the `ui` directory:
```bash
bun run dev
```

This will:
1. Start Vite dev server on `http://localhost:5173`
2. Tauri will automatically launch the app window
3. Hot reload is enabled for both frontend and backend

### Building for Production

From the `ui` directory:
```bash
bun run build
bun run tauri build
```

This creates:
- Windows: `.exe` installer in `app/target/release/bundle/`
- macOS: `.app` bundle and `.dmg`
- Linux: `.AppImage` and `.deb`

## API (Tauri Commands)

### Instances
- `get_instances()` - Get all instances
- `get_instance_details(instance_id)` - Get details for specific instance
- `create_instance(request)` - Create new instance
- `delete_instance(instance_id)` - Delete instance
- `launch_instance(instance_id)` - Launch Minecraft

### Accounts
- `get_accounts()` - Get all accounts
- `add_offline_account(username)` - Add offline account
- `set_active_account(account_id)` - Set active account
- `remove_account(account_id)` - Remove account

### Configuration
- `get_config()` - Get current configuration
- `update_config(config)` - Save configuration

### Versions
- `get_minecraft_versions()` - Get available Minecraft versions

## Frontend Stack

- **React 18**: UI framework
- **TypeScript**: Type safety
- **React Router**: Client-side routing
- **Vite**: Fast build tool optimized for Tauri
- **Bun**: Fast JavaScript runtime and package manager

## Backend Stack

- **Tauri**: Desktop app framework
- **Tokio**: Async runtime
- **Serde**: Serialization
- **Reqwest**: HTTP client
- **Chrono**: Date/time handling

## Why Tauri + Bun?

### Advantages over Iced:
1. **Flexibility**: HTML/CSS/JS for UI = more design freedom
2. **Ecosystem**: Access to npm ecosystem and UI libraries
3. **Development Speed**: Hot reload, better tooling
4. **Performance**: Bun is extremely fast for development
5. **Modern**: Web technologies are constantly improving
6. **Cross-platform**: Better macOS/Linux support

### Why Bun specifically?
- **Speed**: 3-4x faster than npm/yarn
- **TypeScript**: Native TypeScript support
- **Bundle Size**: Smaller bundles
- **Modern**: Built for modern JavaScript

## Cross-Compilation

To build for other platforms:

```bash
# Windows → Linux
bun run tauri build --target x86_64-unknown-linux-gnu

# Windows → macOS  
bun run tauri build --target x86_64-apple-darwin

# Linux → Windows
bun run tauri build --target x86_64-pc-windows-msvc
```

Note: You'll need the appropriate toolchains installed for cross-compilation.

## License

GPL-3.0-only
