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
- **Dial Support** (Encoder 0):
  - ✅ **Dial Press**: Works - assign actions to dial down/up events
  - ⚠️ **Dial Rotation**: Experimental - may not trigger actions reliably
- **Face Buttons**: Experimental - currently ignored
- Software mode control for full device management

### Using the Dial (Encoder)

OpenDeck's default actions don't all support encoders. Use **Multi-Action** or **Run Command** for dial actions.

#### Important Dial Behavior Notes

- **Scene Swap**: Rotating the dial while it is depressed is captured by the device firmware to swap between 'scenes'. This behavior cannot be overridden by the plugin.
- **Dial Press**: Works reliably - assign actions to encoder 0 press/release
- **Dial Rotation**: Returns `-1` (CCW) or `1` (CW) — currently experimental and may not work reliably in the GUI
- **Face Buttons**: The two small buttons above the dial are currently experimental and disabled

> **Note on rotation values**: The dial returns exactly `-1` (counter-clockwise) or `1` (clockwise) — **not** `+1`. This distinction matters because many commands interpret bare numbers differently than signed numbers. For example, `pactl set-sink-volume @DEFAULT_SINK@ 1` sets volume to 1% (absolute), while `pactl set-sink-volume @DEFAULT_SINK@ +1` increments by 1%.

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
