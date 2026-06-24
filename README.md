![Plugin Icon](assets/icon.png)

# OpenDeck Ajazz N1 Plugin

**Fork of [opendeck-akp153](https://github.com/4ndv/opendeck-akp153)**
Many thanks to the original author for the work done on this plugin and everyone else involved in opendeck and the ecosystem.

An unofficial plugin for Ajazz N1 devices. This fork provides dedicated support for Ajazz and Mirabox N1 devices.

## OpenDeck version

Requires OpenDeck 2.5.0 or newer

Built with [openaction](https://crates.io/crates/openaction) 2.5.0

## Supported devices

- Ajazz N1 (0300:3007)
- Mirabox N1 (6603:1000)

## Features

- Full support for all 15 main buttons + 3 top LCD buttons
- **Dial**: Working - press (encoder 0) and rotation (CW/CCW) both trigger actions
- **Face buttons**: Working - the two buttons above the dial (inputs 30/31) are mapped onto the top-left and top-middle display slots (OpenDeck keys 0 and 1)
- Software mode control for full device management

### Encoder / Dial & Face Button Support

**Status**: Working. The dial press, dial rotation (clockwise and counter-clockwise), and both face buttons are detected and forwarded to OpenDeck.

The two face buttons have no display of their own, so they are routed onto the top display slots (keys 0 and 1): you get a configurable icon on the display plus a real button press from the physical face button. The top-right display (key 2) remains display-only.

> Note: rotating the dial *while it is pressed* is captured by the device firmware for its own scene-switching and does not reach the host. Normal rotation and press work as expected.

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
