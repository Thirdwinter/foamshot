# foam_shot

A lightweight screenshot utility based on the Wayland screen capture protocol (`wlroots` extension protocol).

> ⚠️ **Development Status**: Currently in early experimental phase. APIs and features may undergo significant changes.

---

## Features

- 🔍 **Area Selection Capture**: Interactive screen region selection with PNG/JPEG output
- 🔍 **toggle screen freeze**: Press key `f` before clicking the mouse to toggle screen freeze
- 🔍 **compositor**: foamshot can working in `hyprland`, `niri`, `wayfire(TODO: can not change cursor shape)`

---

## Installation
`arch aur`: paru -S foamshot-bin
or build by cargo.

---

## Usage

```
Usage: foamshot [OPTIONS]

Options:
  -p, --path <PATH>  The directory path where the output file is located. The default is the XDG user image path
  -n, --name <NAME>  Output file name, supports time formatting placeholders (such as %Y, %m, %d, %H, %M, %S) [default: foamshot-2025-04-12-20-44-35.png]
      --show-cursor  Whether to display the mouse when taking a screenshot. The default value is false
      --no-copy      Whether to automatically copy the screenshot to the clipboard, requires wl-copy, default is false
      --no-freeze    Whether to freeze the screen before taking a screenshot, the default is false
      --full-screen  Whether to skip automatic full screen capture in interactive mode, the default value is false
      --edit         Whether to enter edit mode after taking a screenshot, the default is false
      --no-notify    disable desktop notify, the default is false
  -h, --help         Print help
  -V, --version      Print version
```
* When the screen is waiting for the mouse to be pressed, press the `a` key to quickly capture the full screen, press the `f` key to toggle freeze state.
* In edit mode, you can resize selectbox, press the `s` key to apply and saved.
* In hyprland,you can  `bind = $mainMod, A, exec, foamshot -p $HOME/Pictures/Screenshots/ -n foam_shot-%Y-%m-%d_%H-%M-%S.png`

---
## Roadmap
- [x] **Multi-monitor coordinated capture**
- [ ] **Cross-compositor compatibility layer**
- [x] **CLI parameters**
- [ ] **Quick-edit mode**
- [ ] **pin**
