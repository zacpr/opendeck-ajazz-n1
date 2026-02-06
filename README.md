![Plugin Icon](assets/icon.png)

# OpenDeck Ajazz N1 Plugin

**Fork of [opendeck-akp153](https://github.com/4ndv/opendeck-akp153)**
Many thanks to the original author for the work done on this plugin and everyone else involved in opendeck and the ecosystem.

An unofficial plugin for Ajazz N1 devices. This fork provides dedicated support for the Ajazz N1 only.

## OpenDeck version

Requires OpenDeck 2.5.0 or newer

Built with [openaction](https://crates.io/crates/openaction) 2.5.0

## Supported devices

- Ajazz N1 (0300:3007)

## Features

- Full support for all 15 main buttons + 3 top LCD buttons
- **3 Virtual Encoders** for maximum flexibility:
  - **Encoder 0**: Left face button (press only)
  - **Encoder 1**: Right face button (press only)
  - **Encoder 2**: Dial (press + rotate with -1/+1 values)
- Software mode control for full device management

### Using Encoders

OpenDeck's default actions don't all support encoders. Use **Multi-Action** or **Run Command** which supports:
- **Dial Down/Up**: Trigger on press/release
- **Dial Rotate**: Use `%d` in your command - substitutes with tick count (`-1` for CCW, `1` for CW — note: `1` not `+1`)

#### Important Dial Behavior Notes

- **Scene Swap**: Rotating the dial while it is depressed is captured by the device firmware to swap between 'scenes'. This behavior cannot be overridden by the plugin.
- **Top Row Buttons**: The two small buttons in the top row (left and right of the dial) can have actions assigned to them independently.
- **Dial Actions**: Actions can be attached to:
  - **Dial Press Down**: Triggered when the dial is pressed
  - **Dial Release**: Triggered when the dial is released
  - **Dial Rotate**: Returns exactly `-1` (counter-clockwise) or `1` (clockwise) — note this is `1` not `+1`

> **Important**: The dial returns exactly `-1` (counter-clockwise) or `1` (clockwise) — **not** `+1`. This distinction matters because many commands interpret bare numbers differently than signed numbers. For example, `pactl set-sink-volume @DEFAULT_SINK@ 1` sets volume to 1% (absolute), while `pactl set-sink-volume @DEFAULT_SINK@ +1` increments by 1%. Use the provided `volume.sh` script or add the `+` sign in your command to ensure proper increment/decrement behavior.

#### Quick Start: Volume Control

A volume control script is included. In OpenDeck for **Encoder 2** (the dial):

1. **Dial Rotate**: Set command to:
   ```
   /path/to/plugin/scripts/volume.sh %d
   ```
   (Replace `/path/to/plugin` with where you extracted the plugin)

2. Make sure the script is executable:
   ```bash
   chmod +x /path/to/plugin/scripts/volume.sh
   ```

The script will automatically adjust volume by 5% per tick in the correct direction.

#### Manual Volume Control

Or configure separate actions without the script:
- **Dial Rotate (Clockwise)**: `pactl set-sink-volume @DEFAULT_SINK@ +5%`
- **Dial Rotate (Counter-clockwise)**: `pactl set-sink-volume @DEFAULT_SINK@ -5%`

## Platform support

- Linux: Developed on Linux, and I use this one, so I assume I'll catch the bugs.
- Mac & Windows: No testing has been performed but it should work. Happy to accept PRs for fixes but I don't have the means or inclination to test these.

## Installation

1. Download an archive from [releases](https://github.com/zacpr/opendeck-ajazz-n1/releases)
2. In OpenDeck: Plugins -> Install from file
3. Linux: Download [udev rules](./40-opendeck-ajazz-n1.rules) and install them by copying into `/etc/udev/rules.d/` and running `sudo udevadm control --reload-rules`
4. Unplug and plug again the device, restart OpenDeck

## Building

### Prerequisites

You'll need:

- A Linux OS of some sort
- Rust 1.87 and up with `x86_64-unknown-linux-gnu` and `x86_64-pc-windows-gnu` targets installed
- Docker
- [just](https://just.systems)

### Preparing environment

```sh
$ just prepare
```

This will build docker image for macOS crosscompilation

### Building a release package

```sh
$ just package
```

## Acknowledgments

This plugin is heavily based on work by contributors of [elgato-streamdeck](https://github.com/streamduck-org/elgato-streamdeck) crate
