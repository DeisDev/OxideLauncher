# OxideLaunch

A Java wrapper JAR for launching Minecraft with proper support for all versions and modloaders.

## Purpose

OxideLaunch handles the complexities of launching different Minecraft versions and modloaders by providing a unified interface. It supports:

- **Modern versions** (1.13+) - Vanilla, Forge, NeoForge, Fabric, Quilt
- **Tweaker-based modloaders** (1.6-1.12.2) - Forge, LiteLoader
- **Legacy versions** (Alpha, Beta, pre-1.6) - Applet support, game directory injection

## Building

### Prerequisites
- Java 8 or later (for building)
- Java 8+ runtime (for running)

### Build Command
```bash
.\build.bat
```

This creates `build/OxideLaunch.jar`.

### Deployment
Copy the JAR to your OxideLauncher data directory:
```
%APPDATA%\OxideLauncher\bin\OxideLaunch.jar
```

## Usage

The JAR is automatically used by OxideLauncher for non-standard launcher types. Manual usage:

```bash
java -cp "OxideLaunch.jar;minecraft.jar;libraries/*" dev.oxide.launch.OxideLaunch [options] -- [game args]
```

### Options

| Option | Description |
|--------|-------------|
| `--launcher <type>` | Launcher type: `standard`, `tweaker`, or `legacy` |
| `--mainClass <class>` | The main class to launch |
| `--gameDir <path>` | The game directory path |
| `--assetsDir <path>` | The assets directory path |
| `--tweakClass <class>` | Add a tweaker class (repeatable) |
| `--width <pixels>` | Window width |
| `--height <pixels>` | Window height |
| `--` | Separator between wrapper args and game args |

### Launcher Types

1. **standard** - Direct reflection to main class. Used for modern Forge, NeoForge, Fabric, Quilt.

2. **tweaker** - Handles LaunchWrapper-based modloaders. Passes `--tweakClass` arguments correctly to `net.minecraft.launchwrapper.Launch`.

3. **legacy** - Supports applet-based launch and game directory field injection for very old Minecraft versions.

## Example Commands

### Modern Forge/Fabric
```bash
java -cp "OxideLaunch.jar;..." dev.oxide.launch.OxideLaunch \
  --launcher standard \
  --mainClass net.fabricmc.loader.impl.launch.knot.KnotClient \
  --gameDir "C:\Users\...\instances\MyInstance" \
  -- --username Player --accessToken ...
```

### Legacy Forge (1.7.10-1.12.2)
```bash
java -cp "OxideLaunch.jar;..." dev.oxide.launch.OxideLaunch \
  --launcher tweaker \
  --mainClass net.minecraft.launchwrapper.Launch \
  --gameDir "..." \
  --tweakClass cpw.mods.fml.common.launcher.FMLTweaker \
  -- --username Player ...
```

## License

GPL-3.0 - See LICENSE in the repository root.
