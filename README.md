# Framework Toolbox

Quick and dirty GUI for utilities of the Framework Laptop

## Installation

Note: **I am new to rust, please look through the code and taste the spaghetti
before installation**

```sh
cargo install --git "https://github.com/taotien/framework_toolbox.git"
```

## Dependencies

- rust
- ectool (DHowett/fw-ectool)
- iio-sensor-proxy
- Polkit/pkexec

## TODO/Need help

- Windows support
- package binaries
- don't rely on hardcoded paths
- run without shell, .desktop file?
- config, esp. for ectool and other dependency paths
- text input values for sliders
- make purty
- tray icon
- check for iio-sensor-proxy
- somehow capture F7/F8 presses to jank re-enable manual brightness while using
  ambient light sensor
- change backlight slider to be offset for auto when it's implemented
- power and side led control
- keyboard remapping
- find better way of communication with "daemon"
- fan curves based on temperature
