# Framework Toolbox

![image](https://user-images.githubusercontent.com/29749622/205031263-4783396d-02e0-4996-bc5a-693db567e131.png)

Quick and dirty GUI for utilities of the Framework Laptop

## Installation

Note: **I am new to rust, please look through the code and taste the spaghetti
before installation**

```sh
cargo install --git "https://github.com/taotien/framework_toolbox.git"
```

Add cargo's bin folder to your desktop environment's PATH.

```sh
# do as root
desktop-file-install fwtb.desktop
```

### If you want just the auto-brightness

Clone the repo

```sh
cargo build --release fwtb-ab
```

Copy the binary from ./target/{arch}/release

## Dependencies

- rust
- ectool (DHowett/fw-ectool)
- Polkit/pkexec
- brightnessctl

## TODO/Need help

- apply all settings on start, not just battery charge
- check if ectool needs specific options on other distros/allow users to manually change arguments
- cleanup unecessary unwraps and expects
- remove dependency on brightnessctl
- Windows support
- package binaries
- text input values for sliders
- tray icon => waiting for iced
- use dbus for brightness stuff?
- somehow capture F7/F8 presses to jank re-enable manual brightness while using
  ambient light sensor
- keyboard remapping
- find better way of communication with "daemon"
- fan curves based on temperature
