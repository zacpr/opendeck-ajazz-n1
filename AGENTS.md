# OpenDeck Ajazz N1 Plugin

## Project Overview

This is an **OpenDeck plugin** that provides hardware support for the Ajazz N1 Stream Controller and various compatible devices (AKP153 series, Mirabox HSV293S, etc.). The plugin bridges USB HID devices with the OpenDeck software, enabling button displays, input handling, and encoder support.

**Key characteristics:**
- Written in Rust using async/await patterns
- Cross-platform: Linux (primary), Windows, macOS
- Plugin architecture based on `openaction` SDK
- Device communication via `mirajazz` crate
- Hot-plug support with device discovery and lifecycle management

## Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (Edition 2024, requires 1.87+) |
| Async Runtime | Tokio |
| HID Communication | `async-hid` (0.4.4), `mirajazz` (0.9.0) |
| OpenDeck SDK | `openaction` (1.1.5) |
| Image Processing | `image` crate (JPEG, BMP support) |
| Build Tool | `just` (command runner) |
| Cross-compilation | Docker (for macOS) |

### Key Dependencies

```toml
async-hid = "0.4.4"      # HID device communication
data-url = "0.3.1"       # Parse data URLs from OpenDeck
futures-lite = "2.6.0"   # Lightweight async utilities
image = "0.25.6"         # Image processing for button displays
mirajazz = "0.9.0"       # Ajazz/Mirabox device protocol
openaction = "1.1.5"     # OpenDeck plugin SDK
simplelog = "0.12.2"     # Logging
tokio = "1.44.2"         # Async runtime
tokio-util = "0.7.15"    # Additional Tokio utilities
```

## Project Structure

```
├── Cargo.toml              # Rust project configuration
├── Cargo.lock              # Dependency lock file
├── justfile                # Build automation commands
├── manifest.json           # OpenDeck plugin manifest
├── 40-opendeck-ajazz-n1.rules  # Linux udev rules for USB access
├── src/
│   ├── main.rs             # Plugin entry point, OpenDeck handlers
│   ├── device.rs           # Device connection, keepalive, image handling
│   ├── watcher.rs          # USB device discovery and hot-plug
│   ├── inputs.rs           # Input event mapping (device → OpenDeck)
│   ├── mappings.rs         # Device identification, layouts, image formats
│   └── bin/                # Debug utilities
│       ├── test_detect.rs  # Device detection test
│       ├── debug_inputs.rs # Raw input debugging
│       ├── debug_raw.rs    # Raw HID debugging
│       ├── debug_raw_hid.rs
│       ├── map_buttons.rs  # Button mapping utility
│       └── simple_read.rs  # Simple read test
├── assets/
│   ├── icon.png            # Plugin icon
│   └── icon.svg            # Source icon
└── .github/workflows/
    └── build.yml           # CI/CD for cross-platform builds
```

## Module Architecture

### Core Modules

**`main.rs`** - Plugin entry point
- Implements `GlobalEventHandler` and `ActionEventHandler` from `openaction`
- Handles OpenDeck events: `plugin_ready`, `set_image`, `set_brightness`
- Global state management: `DEVICES`, `TOKENS`, `TRACKER`
- Signal handling for graceful shutdown (SIGTERM on Linux/macOS)

**`device.rs`** - Device lifecycle management
- `device_task()`: Main device handling loop
- `connect()`: Establishes HID connection
- `device_events_task()`: Reads button/encoder events from device
- `keepalive_task()`: Sends periodic keepalive (10s interval)
- `handle_set_image()`: Processes JPEG images from OpenDeck
- `handle_error()`: Error recovery and cleanup

**`watcher.rs`** - Device discovery
- `watcher_task()`: Main watcher loop
- Scans for devices matching known VID/PID pairs
- Handles `DeviceLifecycleEvent::Connected` and `Disconnected`
- Spawns device tasks for newly connected devices

**`inputs.rs`** - Input mapping
- `process_input_n1()`: N1-specific input handling
- `process_input_akp153()`: AKP153 input handling
- `opendeck_to_device()`: Converts OpenDeck key index to device key index
- `device_to_opendeck_n1()`: Reverse mapping for N1

**`mappings.rs`** - Device definitions
- `Kind` enum: All supported device types (N1, AKP153 variants, etc.)
- VID/PID constants for device identification
- `DeviceQuery` definitions for HID discovery
- Image format specifications per device/key
- Layout definitions (rows, columns, encoder count)

### Device Support

| Device | VID | PID | Protocol | Layout |
|--------|-----|-----|----------|--------|
| Ajazz N1 | 0x0300 | 0x3007 | v3 | 6×3 grid, 1 encoder |
| Ajazz AKP153E | 0x0300 | 0x1010 | v1 | 3×6 grid |
| Ajazz AKP153E (rev.2) | 0x0300 | 0x3010 | v3 | 3×6 grid |
| Ajazz AKP153R | 0x0300 | 0x1020 | v1 | 3×6 grid |
| Mirabox HSV293S | 0x5548 | 0x6670 | v1 | 3×6 grid |
| Mirabox HSV293SV3 | 0x6603 | 0x1014 | v3 | 3×6 grid |

## Build Commands

All builds are orchestrated via `just` (see `justfile`):

```bash
# Install prerequisites (Docker image for macOS cross-compilation)
just prepare

# Build for current platform (Linux release)
just build-linux

# Cross-compile for Windows
just build-win

# Cross-compile for macOS (requires Docker)
just build-mac

# Collect plugin files to build/ directory
just collect

# Full package build (Linux + collect + zip)
just package

# Clean build artifacts
just clean

# Version bumping (interactive)
just bump [version]
just tag [version]
```

### Manual Cargo Commands

```bash
# Development build
cargo build

# Run plugin
cargo run

# Run debug utilities
cargo run --bin test_detect
cargo run --bin debug_inputs
cargo run --bin map_buttons

# Release build for Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Cross-compile for Windows
cargo build --release --target x86_64-pc-windows-gnu
```

## Development Workflow

### Running Debug Utilities

The project includes several debug binaries for development:

```bash
# Test if device is detected
cargo run --bin test_detect

# Debug raw inputs with detailed logging
cargo run --bin debug_inputs

# Simple button mapper for testing mappings
cargo run --bin map_buttons

# Raw HID debugging
cargo run --bin debug_raw
cargo run --bin debug_raw_hid

# Simple read test
cargo run --bin simple_read
```

### Linux Development Setup

1. **Install udev rules** (required for USB access without root):
   ```bash
   sudo cp 40-opendeck-ajazz-n1.rules /etc/udev/rules.d/
   sudo udevadm control --reload-rules
   sudo udevadm trigger
   ```

2. **Add user to plugdev group**:
   ```bash
   sudo usermod -a -G plugdev $USER
   # Log out and back in for changes to take effect
   ```

3. **Verify device detection**:
   ```bash
   lsusb | grep -i ajazz
   cargo run --bin test_detect
   ```

## Code Style Guidelines

- **Rust Edition**: 2024 (requires Rust 1.87+)
- **Async/Await**: Used throughout for concurrent operations
- **Error Handling**: Use `?` operator; `MirajazzError` for device errors
- **Logging**: Use `log` crate macros (`log::info!`, `log::debug!`, `log::error!`)
- **Global State**: Managed via `LazyLock` + async locks (`RwLock`, `Mutex`)

### Naming Conventions

- Modules: `snake_case` (e.g., `device.rs`, `mappings.rs`)
- Types/Enums: `PascalCase` (e.g., `DeviceStateUpdate`, `Kind`)
- Functions/Variables: `snake_case` (e.g., `opendeck_to_device`, `handle_error`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `N1_PID`, `DEVICE_NAMESPACE`)

### Error Handling Patterns

```rust
// In device operations, return MirajazzError
pub async fn connect(candidate: &CandidateDevice) -> Result<Device, MirajazzError> {
    Device::connect(&candidate.dev, ...).await
}

// In main/error handlers, use handle_error for cleanup
if let Err(err) = result {
    handle_error(&device_id, err).await;
}
```

## Testing Strategy

**Note**: There is no formal test suite (no `tests/` directory). Testing is done via:

1. **Debug binaries** in `src/bin/` for manual testing
2. **CI/CD builds** verify compilation across platforms
3. **Manual integration testing** with physical hardware

### Debug Utilities Usage

- `test_detect`: Verify device is detectable via USB/HID
- `debug_inputs`: Verify input mapping is correct
- `map_buttons`: Interactive button mapping verification

## Deployment Process

### Plugin Package Structure

```
opendeck-ajazz-n1.sdPlugin/  (zip file)
└── net.ashurtech.plugins.opendeck-ajazz-n1.sdPlugin/
    ├── manifest.json              # Plugin metadata
    ├── icon.png                   # Plugin icon
    ├── opendeck-ajazz-n1-linux    # Linux binary
    ├── opendeck-ajazz-n1-win.exe  # Windows binary
    └── opendeck-ajazz-n1-macos    # macOS binary
```

### Release Process

1. **Local version bump**:
   ```bash
   just bump  # Uses git-cliff to determine next version
   just tag   # Creates git tag and updates CHANGELOG.md
   ```

2. **GitHub Actions** automatically:
   - Builds for Linux, Windows, macOS on tag push
   - Creates release with packaged plugin
   - Generates release notes

### Installation (End Users)

1. Download release from GitHub
2. In OpenDeck: Plugins → Install from file
3. Linux only: Install udev rules (see Linux Development Setup)
4. Unplug/replug device, restart OpenDeck

## Security Considerations

- **USB Permissions**: Plugin requires USB HID access; udev rules grant access to specific VID/PID pairs
- **Serial Numbers**: Some v1 devices share the same serial number; the plugin uses custom suffixes to differentiate
- **Image Processing**: Only JPEG images are accepted from OpenDeck; data URLs are validated before processing

## Key Implementation Details

### N1 Device Specifics

- **Software Mode**: N1 requires mode 3 (software control) to be set on connection
- **Layout**: 6 rows × 3 columns (18 display keys: 15 main buttons + 3 top LCDs)
- **Encoder**: Single encoder (inputs 50/51 for twist, treated as button)
- **Image Sizes**: Top LCDs (64×64), Main buttons (96×96)
- **Timing**: Small delays (20-50ms) after certain operations for device stability

### Protocol Versions

- **v1**: Older devices (AKP153, AKP153E, AKP153R)
- **v2/v3**: Newer devices with proper serial numbers (AKP153E rev.2, HSV293SV3, N1)

### Device ID Format

```
// v2/v3 devices: N1-<serial_number>
N1-123456789ABC

// v1 devices: N1-355499441494-<suffix>
N1-355499441494-N1
N1-355499441494-153
```

## Related Projects

- **Upstream**: Forked from [opendeck-akp153](https://github.com/4ndv/opendeck-akp153)
- **Device Protocol**: Based on [elgato-streamdeck](https://github.com/streamduck-org/elgato-streamdeck) crate
- **OpenDeck**: [opendeck](https://github.com/ninjadev64/OpenDeck) - Stream Deck alternative
