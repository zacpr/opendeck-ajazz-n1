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
- **Dial Press**: Working - the dial can be used as a button (encoder 0)
- Software mode control for full device management

### Encoder / Dial Support (Work in Progress)

**Status**: The dial press function works and can trigger actions. Dial rotation and the face buttons above the dial are a work in progress.

**The Challenge**: The Ajazz N1 device firmware aggressively captures certain input combinations for its own "scene switching" functionality:
- Rotating the dial while depressed triggers firmware scene switching
- The face buttons (inputs 30/31) may also be intercepted by the firmware

While we can detect these inputs at the USB level, the device firmware may intercept or override them before they reach the host. This makes reliable encoder rotation and face button support difficult or potentially impossible to implement fully.

We're continuing to investigate workarounds, but full encoder/face button configuration may be limited by the device firmware itself.

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
